// Function to create dataset for time series plots.

use crate::scraper::{Scraper, ScrapedData};
use crate::time_series_plot::TimeSeriesData;
use crate::time_series_plot::SinglePoint;
use crate::helpers_ts;

// Function to create the data sets to plot.
pub fn create_time_series_datasets(scraper: &Scraper, selected_trip: &str) -> Vec<TimeSeriesData> {
    // Create datasets of plots.
    let mut datasets = Vec::new();
    
    // Get all points for the selected trip.
    let trip_data: Vec<&ScrapedData> = scraper.scrapings.iter()
        .filter(|scraped| scraped.trip_num == *selected_trip)
        .collect();
    
    // Abort if no data in the trip.
    if trip_data.is_empty() {
        return datasets;
    }
    
    // Find the overall trip start and end times.
    let trip_start_time = trip_data.iter()
        .map(|data| data.unix_time)
        .min()
        .unwrap_or(0);
    
    let trip_end_time = trip_data.iter()
        .map(|data| data.unix_time)
        .max()
        .unwrap_or(0);

    // Battery voltage time series.
    let battery_points: Vec<SinglePoint> = trip_data.iter()
    .filter_map(|data| {
        // Look for "Battery voltage" in the ev_detail vector.
        // This is in all events, even the logic ones.
        data.ev_detail.iter()
            .find(|(tag, _)| tag == "Battery voltage")
            .and_then(|(_, value)| {
                // Parse the f16 string value to f32.
                value.parse::<f32>().ok()
            })
            .map(|voltage| SinglePoint {
                unix_time: data.unix_time,
                point_value: voltage,
            })
    })
    .collect();

    if !battery_points.is_empty() {
        datasets.push(TimeSeriesData {
            data_type: "Analog".to_string(),
            series_name: "Battery".to_string(),
            units: "V".to_string(),
            levels: Vec::new(),
            time_series_points: battery_points,
            multi_traces: Vec::new(),
        });
    }

    // Speed time series.
    // This is in all events, even the logic ones if they contain gps data.
    let speed_points: Vec<SinglePoint> = trip_data.iter()
        .map(|data| SinglePoint {
            unix_time: data.unix_time,
            point_value: data.gps_speed as f32,
        })
        .collect();

    if !speed_points.is_empty() {
        datasets.push(TimeSeriesData {
            data_type: "Analog".to_string(),
            series_name: "Speed".to_string(),
            units: "kph".to_string(),
            levels: Vec::new(),
            time_series_points: speed_points,
            multi_traces: Vec::new(),
        });
    }

    // The impulse is an instantaneous event marker.
    let xsidlestart_points: Vec<SinglePoint> = trip_data.iter()
        .filter(|data| data.event_type == "XSIDLESTART")
        .map(|data| SinglePoint {
            unix_time: data.unix_time,
            point_value: 1.0,
        })
        .collect();

    // The digital pulse shows the active duration.
    let mut xsidle_pulse_points: Vec<SinglePoint> = Vec::new();
    
    // Process XSIDLE events (similar to UNBUCKLED pulse creation)
    for data in trip_data.iter().filter(|d| d.event_type == "XSIDLE") {
        if let Some(duration) = data.ev_detail.iter()
            .find(|(tag, _)| tag == "Max idle")
            .and_then(|(_, value)| value.parse::<u64>().ok())
        {
            let event_end_time = data.unix_time;
            let event_start_time = if event_end_time >= duration {
                event_end_time - duration
            } else {
                trip_start_time
            };

            // Create pulse at full scale for visual separation above the impulse.
            xsidle_pulse_points.push(SinglePoint {
                unix_time: event_start_time,
                point_value: 0.0,
            });
            xsidle_pulse_points.push(SinglePoint {
                unix_time: event_start_time,
                point_value: 1.0,
            });
            xsidle_pulse_points.push(SinglePoint {
                unix_time: event_end_time,
                point_value: 1.0,
            });
            xsidle_pulse_points.push(SinglePoint {
                unix_time: event_end_time,
                point_value: 0.0,
            });
        }
    }

    // Add trip start and end baselines for the pulse trace.
    let mut xsidle_trace_points = Vec::new();
    xsidle_trace_points.push(SinglePoint {
        unix_time: trip_start_time,
        point_value: 0.0,
    });
    xsidle_trace_points.extend(xsidle_pulse_points);
    xsidle_trace_points.push(SinglePoint {
        unix_time: trip_end_time,
        point_value: 0.0,
    });

    // Only create dataset if there's at least one trace with events.
    if !xsidlestart_points.is_empty() || xsidle_trace_points.len() > 2 {
        datasets.push(TimeSeriesData {
            data_type: "ImpulseDigitalCombo".to_string(),
            series_name: "EXCESS_IDLE".to_string(),
            units: "Active".to_string(),
            levels: vec!["Start".to_string(), "Active".to_string()],
            time_series_points: Vec::new(),
            // multi_traces: vec![xsidlestart_points, xsidle_trace_points],
            multi_traces: vec![xsidle_trace_points, xsidlestart_points],
        });
    }

    // Process each unique event type once to create combined datasets.
    // That is a combined dataset for each type of event.
    let unique_event_types: std::collections::BTreeSet<String> = trip_data.iter()
        .map(|data| data.event_type.clone())
        .collect();

    for event_type in unique_event_types {
        match event_type.as_str() {
            "ENGINETEMP" => {
                // Get all points for this event type in the selected trip.
                let ev_points: Vec<SinglePoint> = trip_data.iter()
                    // Filter by event type.
                    .filter(|data| data.event_type == event_type)
                    .filter_map(|data| {
                        // Look for event duration in the ev_detail vector.
                        data.ev_detail.iter()
                            .find(|(tag, _)| tag == "Duration")
                            .and_then(|(_, value)| {
                                // Parse the integer string value to f32.
                                value.parse::<f32>().ok()
                            })
                            .map(|event_point| SinglePoint {
                                unix_time: data.unix_time,
                                point_value: event_point,
                            })
                    })
                    .collect();
                
                if !ev_points.is_empty() {
                    // Convert single points to pulse data.
                    let pulse_points = helpers_ts::convert_to_pulse_data(&ev_points, trip_start_time, trip_end_time, "Digital");
    
                    // Push the digital time series events to list of datasets.
                    datasets.push(TimeSeriesData {
                        data_type: "Digital".to_string(),
                        series_name: event_type.clone(),
                        units: "Active".to_string(),
                        levels: Vec::new(),
                        time_series_points: pulse_points,
                        multi_traces: Vec::new(),
                    });
                }
            }
            "LOWCOOLANT" => {
                // Get all points for this event type in the selected trip.
                let ev_points: Vec<SinglePoint> = trip_data.iter()
                    // Filter by event type.
                    .filter(|data| data.event_type == event_type)
                    .filter_map(|data| {
                        // Look for event duration in the ev_detail vector.
                        data.ev_detail.iter()
                            .find(|(tag, _)| tag == "Duration")
                            .and_then(|(_, value)| {
                                // Parse the integer string value to f32.
                                value.parse::<f32>().ok()
                            })
                            .map(|event_point| SinglePoint {
                                unix_time: data.unix_time,
                                point_value: event_point,
                            })
                    })
                    .collect();
                
                if !ev_points.is_empty() {
                    // Convert single points to pulse data.
                    let pulse_points = helpers_ts::convert_to_pulse_data(&ev_points, trip_start_time, trip_end_time, "Digital");    
    
                    // Push the digital time series events to list of datasets.
                    datasets.push(TimeSeriesData {
                        data_type: "Digital".to_string(),
                        series_name: event_type.clone(),
                        units: "Active".to_string(),
                        levels: Vec::new(),
                        time_series_points: pulse_points,
                        multi_traces: Vec::new(),
                    });
                }
            }
            "OILPRESSURE" => {
                // Get all points for this event type in the selected trip.
                let ev_points: Vec<SinglePoint> = trip_data.iter()
                    // Filter by event type.
                    .filter(|data| data.event_type == event_type)
                    .filter_map(|data| {
                        // Look for event duration in the ev_detail vector.
                        data.ev_detail.iter()
                            .find(|(tag, _)| tag == "Duration")
                            .and_then(|(_, value)| {
                                // Parse the integer string value to f32.
                                value.parse::<f32>().ok()
                            })
                            .map(|event_point| SinglePoint {
                                unix_time: data.unix_time,
                                point_value: event_point,
                            })
                    })
                    .collect();
                
                if !ev_points.is_empty() {
                    // Convert single points to pulse data.
                    let pulse_points = helpers_ts::convert_to_pulse_data(&ev_points, trip_start_time, trip_end_time, "Digital");    
    
                    // Push the digital time series events to list of datasets.
                    datasets.push(TimeSeriesData {
                        data_type: "Digital".to_string(),
                        series_name: event_type.clone(),
                        units: "Active".to_string(),
                        levels: Vec::new(),
                        time_series_points: pulse_points,
                        multi_traces: Vec::new(),
                    });
                }
            }
            "OVERSPEED" => {
                // Get all points for this event type in the selected trip.
                let ev_points: Vec<SinglePoint> = trip_data.iter()
                    // Filter by event type
                    .filter(|data| data.event_type == event_type)
                    .filter_map(|data| {
                        // Look for event duration in the ev_detail vector.
                        data.ev_detail.iter()
                            .find(|(tag, _)| tag == "Duration")
                            .and_then(|(_, value)| {
                                // Parse the integer string value to f32.
                                value.parse::<f32>().ok()
                            })
                            .map(|event_point| SinglePoint {
                                unix_time: data.unix_time,
                                point_value: event_point,
                            })
                    })
                    .collect();
                
                if !ev_points.is_empty() {
                    // Convert single points to pulse data.
                    let pulse_points = helpers_ts::convert_to_pulse_data(&ev_points, trip_start_time, trip_end_time, "Digital");    
    
                    // Push the digital time series events to list of datasets.
                    datasets.push(TimeSeriesData {
                        data_type: "Digital".to_string(),
                        series_name: event_type.clone(),
                        units: "Active".to_string(),
                        levels: Vec::new(),
                        time_series_points: pulse_points,
                        multi_traces: Vec::new(),
                    });
                }
            }
            "ZONEOVERSPEED" => {
                // Get all points for this event type in the selected trip.
                let ev_points: Vec<SinglePoint> = trip_data.iter()
                    // Filter by event type
                    .filter(|data| data.event_type == event_type)
                    .filter_map(|data| {
                        // Look for event duration in the ev_detail vector.
                        data.ev_detail.iter()
                            .find(|(tag, _)| tag == "Duration")
                            .and_then(|(_, value)| {
                                // Parse the integer string value to f32.
                                value.parse::<f32>().ok()
                            })
                            .map(|event_point| SinglePoint {
                                unix_time: data.unix_time,
                                point_value: event_point,
                            })
                    })
                    .collect();
                
                if !ev_points.is_empty() {
                    // Convert single points to pulse data.
                    let pulse_points = helpers_ts::convert_to_pulse_data(&ev_points, trip_start_time, trip_end_time, "Digital");    
    
                    // Push the digital time series events to list of datasets.
                    datasets.push(TimeSeriesData {
                        data_type: "Digital".to_string(),
                        series_name: event_type.clone(),
                        units: "Active".to_string(),
                        levels: Vec::new(),
                        time_series_points: pulse_points,
                        multi_traces: Vec::new(),
                    });
                }
            }
            "IMPACT" => {
                // Get all points for this event type in the selected trip.
                let ev_points: Vec<SinglePoint> = trip_data.iter()
                    // Filter by event type.
                    .filter(|data| data.event_type == event_type)
                    .filter_map(|data| {
                        // Look for event severity in the ev_detail vector.
                        data.ev_detail.iter()
                            .find(|(tag, _)| tag == "Severity")
                            .and_then(|(_, value)| {
                                // Translate severity strings to numeric levels.
                                let numeric_level = match value.as_str() {
                                    "-" => 1.0,  // Low
                                    "W" => 2.0,  // Warning
                                    "C" => 3.0,  // Critical
                                    _ => {
                                        // Try to parse as number (fallback).
                                        value.parse::<f32>().unwrap_or(1.0)
                                    }
                                };
                                Some(numeric_level)
                            })
                            .map(|event_point| SinglePoint {
                                unix_time: data.unix_time,
                                point_value: event_point,
                            })
                    })
                    .collect();

                    if !ev_points.is_empty() {
                    // For impulse data, we don't convert to pulse data.
                    // We keep the original points as instantaneous events.
                    
                    // Push the impulse time series events to list of datasets.
                    datasets.push(TimeSeriesData {
                        data_type: "Impulse".to_string(),
                        series_name: event_type.clone(),
                        units: "Severity".to_string(),
                        levels: vec!["Low".to_string(), "Warning".to_string(), "Critical".to_string()],
                        time_series_points: ev_points,
                        multi_traces: Vec::new(),
                    });
                }
            } 
            "ZONECHANGE" => {
                // Get all points for this event type in the selected trip.
                let ev_points: Vec<SinglePoint> = trip_data.iter()
                    // Filter by event type.
                    .filter(|data| data.event_type == event_type)
                    .filter_map(|data| {
                        // Look for event zone output in the ev_detail vector.
                        data.ev_detail.iter()
                            .find(|(tag, _)| tag == "Zone output")
                            .and_then(|(_, value)| {
                                // Parse the integer string value to f32.
                                // Note that we want the no zone 0 value to
                                // be above the baseline so add 1 to the zone output value.
                                value.parse::<f32>().ok()
                            })
                            .map(|event_point| SinglePoint {
                                unix_time: data.unix_time,
                                point_value: event_point + 1.0,
                            })
                    })
                    .collect();

                    if !ev_points.is_empty() {
                    // For impulse data, we don't convert to pulse data.
                    // We keep the original points as instantaneous events.
                    
                    // Push the impulse time series events to list of datasets.
                    // While there are 4 zones, there is also the condition of no zone,
                    // i.e. not in any zone.
                    datasets.push(TimeSeriesData {
                        data_type: "Impulse".to_string(),
                        series_name: event_type.clone(),
                        units: "Zone Output".to_string(),
                        levels: vec!["No Zone".to_string(), "1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()],
                        time_series_points: ev_points,
                        multi_traces: Vec::new(),
                    });
                }
            }
            "ZONETRANSITION" => {
                // Get all points for this event type in the selected trip.
                let ev_points: Vec<SinglePoint> = trip_data.iter()
                    // Filter by event type.
                    .filter(|data| data.event_type == event_type)
                    .filter_map(|data| {
                        // Look for event zone output in the ev_detail vector.
                        data.ev_detail.iter()
                            .find(|(tag, _)| tag == "Zone output")
                            .and_then(|(_, value)| {
                                // Parse the integer string value to f32.
                                // Note that we want the no zone 0 value to
                                // be above the baseline so add 1 to the zone output value.
                                value.parse::<f32>().ok()
                            })
                            .map(|event_point| SinglePoint {
                                unix_time: data.unix_time,
                                point_value: event_point + 1.0,
                            })
                    })
                    .collect();

                    if !ev_points.is_empty() {
                    // For impulse data, we don't convert to pulse data.
                    // We keep the original points as instantaneous events.
                    
                    // Push the impulse time series events to list of datasets.
                    // While there are 4 zones, there is also the condition of no zone,
                    // i.e. not in any zone.
                    datasets.push(TimeSeriesData {
                        data_type: "Impulse".to_string(),
                        series_name: event_type.clone(),
                        units: "Zone Output".to_string(),
                        levels: vec!["No Zone".to_string(), "1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()],
                        time_series_points: ev_points,
                        multi_traces: Vec::new(),
                    });
                }
            }
            "OVERLOAD" => {
                // Get all points for this event type in the selected trip.
                let ev_points: Vec<SinglePoint> = trip_data.iter()
                    // Filter by event type.
                    .filter(|data| data.event_type == event_type)
                    .filter_map(|data| {
                        // Look for event duration in the ev_detail vector.
                        data.ev_detail.iter()
                            .find(|(tag, _)| tag == "Duration")
                            .and_then(|(_, value)| {
                                // Parse the integer string value to f32.
                                value.parse::<f32>().ok()
                            })
                            .map(|event_point| SinglePoint {
                                unix_time: data.unix_time,
                                point_value: event_point,
                            })
                    })
                    .collect();
                
                if !ev_points.is_empty() {
                    // Convert single points to pulse data.
                    let pulse_points = helpers_ts::convert_to_pulse_data(&ev_points, trip_start_time, trip_end_time, "Digital");    
    
                    // Push the digital time series events to list of datasets.
                    datasets.push(TimeSeriesData {
                        data_type: "Digital".to_string(),
                        series_name: event_type.clone(),
                        units: "Active".to_string(),
                        levels: Vec::new(),
                        time_series_points: pulse_points,
                        multi_traces: Vec::new(),
                    });
                }
            }
            "OFFSEAT" => {
                // Get all points for this event type in the selected trip.
                let ev_points: Vec<SinglePoint> = trip_data.iter()
                    // Filter by event type.
                    .filter(|data| data.event_type == event_type)
                    .filter_map(|data| {
                        // Look for event duration in the ev_detail vector.
                        data.ev_detail.iter()
                            .find(|(tag, _)| tag == "Duration")
                            .and_then(|(_, value)| {
                                // Parse the integer string value to f32.
                                value.parse::<f32>().ok()
                            })
                            .map(|event_point| SinglePoint {
                                unix_time: data.unix_time,
                                point_value: event_point,
                            })
                    })
                    .collect();
                
                if !ev_points.is_empty() {
                    // Convert single points to pulse data.
                    let pulse_points = helpers_ts::convert_to_pulse_data(&ev_points, trip_start_time, trip_end_time, "Digital");    
    
                    // Push the digital time series events to list of datasets.
                    datasets.push(TimeSeriesData {
                        data_type: "Digital".to_string(),
                        series_name: event_type.clone(),
                        units: "Active".to_string(),
                        levels: Vec::new(),
                        time_series_points: pulse_points,
                        multi_traces: Vec::new(),
                    });
                }
            }
            "UNBUCKLED" => {
                let mut driver_points: Vec<SinglePoint> = Vec::new();
                let mut passenger_points: Vec<SinglePoint> = Vec::new();
                
                // Add trip start baseline for driver trace.
                driver_points.push(SinglePoint {
                    unix_time: trip_start_time,
                    point_value: 0.0,
                });
                
                // Process Driver unbuckled events (pulses at level 2.0).
                for data in trip_data.iter().filter(|d| d.event_type == event_type) {
                    let is_driver = data.ev_detail.iter()
                        .find(|(tag, _)| tag == "Seat owner")
                        .map(|(_, value)| value == "D")
                        .unwrap_or(false);
                    
                    if is_driver {
                        if let Some(duration) = data.ev_detail.iter()
                            .find(|(tag, _)| tag == "Duration")
                            .and_then(|(_, value)| value.parse::<u64>().ok())
                        {
                            let event_end_time = data.unix_time;
                            let event_start_time = if event_end_time >= duration {
                                event_end_time - duration
                            } else {
                                trip_start_time
                            };
                            
                            // Create pulse at level 2.0.
                            driver_points.push(SinglePoint {
                                unix_time: event_start_time,
                                point_value: 0.0,
                            });
                            driver_points.push(SinglePoint {
                                unix_time: event_start_time,
                                point_value: 1.0,
                            });
                            driver_points.push(SinglePoint {
                                unix_time: event_end_time,
                                point_value: 1.0,
                            });
                            driver_points.push(SinglePoint {
                                unix_time: event_end_time,
                                point_value: 0.0,
                            });
                        }
                    }
                }
                
                // Add trip end baseline for driver trace.
                driver_points.push(SinglePoint {
                    unix_time: trip_end_time,
                    point_value: 0.0,
                });
                
                // Add trip start baseline for passenger trace.
                passenger_points.push(SinglePoint {
                    unix_time: trip_start_time,
                    point_value: 0.0,
                });
                
                // Process Passenger unbuckled events (pulses at level 1.0).
                for data in trip_data.iter().filter(|d| d.event_type == event_type) {
                    let is_passenger = data.ev_detail.iter()
                        .find(|(tag, _)| tag == "Seat owner")
                        .map(|(_, value)| value == "P")
                        .unwrap_or(false);
                    
                    if is_passenger {
                        if let Some(duration) = data.ev_detail.iter()
                            .find(|(tag, _)| tag == "Duration")
                            .and_then(|(_, value)| value.parse::<u64>().ok())
                        {
                            let event_end_time = data.unix_time;
                            let event_start_time = if event_end_time >= duration {
                                event_end_time - duration
                            } else {
                                trip_start_time
                            };
                            
                            // Create pulse at level 1.0.
                            passenger_points.push(SinglePoint {
                                unix_time: event_start_time,
                                point_value: 0.0,
                            });
                            passenger_points.push(SinglePoint {
                                unix_time: event_start_time,
                                point_value: 0.5,
                            });
                            passenger_points.push(SinglePoint {
                                unix_time: event_end_time,
                                point_value: 0.5,
                            });
                            passenger_points.push(SinglePoint {
                                unix_time: event_end_time,
                                point_value: 0.0,
                            });
                        }
                    }
                }

                // Only create dataset if there's at least one trace with events.
                // Using "Crew" instead of Passenger as it fits on the plot better.
                if driver_points.len() > 2 || passenger_points.len() > 2 {
                    datasets.push(TimeSeriesData {
                        data_type: "DualDigital".to_string(),
                        series_name: "UNBUCKLED".to_string(),
                        units: "Active".to_string(),
                        levels: vec!["Crew".to_string(), "Driver".to_string()],
                        time_series_points: Vec::new(),
                        multi_traces: vec![passenger_points, driver_points],
                    });
                }
            } _ => {}
                
        // Set of all data series to plot.
        }
    }
    datasets
}
