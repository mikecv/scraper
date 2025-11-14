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
            tall_chart: false,
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
            tall_chart: false,
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
            multi_traces: vec![xsidle_trace_points, xsidlestart_points],
            tall_chart: false,
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
                        tall_chart: false,
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
                        tall_chart: false,
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
                        tall_chart: false,
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
                        tall_chart: false,
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
                        tall_chart: false,
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
                                    "1" => 1.0,
                                    "2" => 2.0,
                                    "3" => 3.0,
                                    "4" => 4.0,
                                    "5" => 5.0,
                                    "6" => 6.0,
                                    "7" => 7.0,
                                    "8" => 8.0,
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
                        tall_chart: false,
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
                        tall_chart: false,
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
                        tall_chart: false,
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
                        tall_chart: false,
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
                        tall_chart: false,
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
                        data_type: "MultiDigital".to_string(),
                        series_name: "UNBUCKLED".to_string(),
                        units: "Active".to_string(),
                        levels: vec!["Crew".to_string(), "Driver".to_string()],
                        time_series_points: Vec::new(),
                        multi_traces: vec![passenger_points, driver_points],
                        tall_chart: false,
                    });
                }
            }
            "INPUT" => {
                let mut input_traces: Vec<Vec<SinglePoint>> = vec![Vec::new(); 8];
                // Track the polarity (state) for each input trace.
                let mut input_polarities: Vec<Option<i32>> = vec![None; 8];
                const NUM_TRACES: f32 = 8.0;

                // Process all events and add pulses.
                for data in trip_data.iter().filter(|d| d.event_type == event_type) {
                    if let Some(input_num) = data.ev_detail.iter()
                        .find(|(tag, _)| tag == "Input")
                        .and_then(|(_, value)| value.parse::<usize>().ok())
                    {
                        if input_num >= 1 && input_num <= 8 {
                            let trace_index = input_num - 1; 
                            
                            // Get the state: 0 = active LOW, 1 = active HIGH
                            let state_tag = data.ev_detail.iter()
                                .find(|(tag, _)| tag == "State")
                                .and_then(|(_, value)| value.parse::<i32>().ok())
                                .unwrap_or(1);
                            
                            // Store the polarity for this trace.
                            input_polarities[trace_index] = Some(state_tag);
                            
                            // Vertical Stacking Calculation.
                            let i = trace_index as f32;
                            let h_trace = 1.0 / NUM_TRACES; 
                            // 10% bottom margin for separation between traces.
                            let y_base = i * h_trace + h_trace * 0.10;
                            // 80% signal height.
                            let h_sig = h_trace * 0.80;

                            // Polarity Calculation:
                            // state_tag == 0: Active LOW  -> pulse is LOW (y_base), baseline is HIGH (y_base + h_sig)
                            // state_tag == 1: Active HIGH -> pulse is HIGH (y_base + h_sig), baseline is LOW (y_base)
                            let (y_pulse, y_baseline) = if state_tag == 0 { 
                                // State 0: Active LOW (Pulse is low, Baseline is high)
                                (y_base, y_base + h_sig)
                            } else {
                                // State 1: Active HIGH (Pulse is high, Baseline is low)
                                (y_base + h_sig, y_base)
                            };
                            
                            if let Some(duration) = data.ev_detail.iter()
                                .find(|(tag, _)| tag == "Duration")
                                .and_then(|(_, value)| value.parse::<u64>().ok())
                            {
                                // Only process events with duration > 0
                                if duration > 0 {
                                    let event_end_time = data.unix_time;
                                    let calculated_start_time = if event_end_time >= duration {
                                        event_end_time - duration
                                    } else {
                                        0
                                    };
                                    
                                    // If the event started before the trip, clamp it to trip start.
                                    let event_start_time = if calculated_start_time < trip_start_time {
                                        trip_start_time
                                    } else {
                                        calculated_start_time
                                    };
                                    
                                    let current_trace = &mut input_traces[trace_index];
                                    
                                    // Only add baseline before pulse if event didn't start before trip.
                                    if calculated_start_time >= trip_start_time {
                                        let last_point_is_baseline_at_same_time = current_trace.last()
                                            .map_or(false, |last| last.unix_time == event_start_time && (last.point_value - y_baseline).abs() < f32::EPSILON);
                                        
                                        // Baseline before pulse (conditional insertion).
                                        if !last_point_is_baseline_at_same_time {
                                            current_trace.push(SinglePoint {
                                                unix_time: event_start_time,
                                                point_value: y_baseline, 
                                            });
                                        }
                                    }

                                    // The pulse rectangle.
                                    current_trace.push(SinglePoint { unix_time: event_start_time, point_value: y_pulse });
                                    current_trace.push(SinglePoint { unix_time: event_end_time, point_value: y_pulse });
                                    current_trace.push(SinglePoint { unix_time: event_end_time, point_value: y_baseline });
                                }
                            }
                        }
                    }
                }
                
                // Add trip start and end baselines for each trace that has events.
                for (i, trace) in input_traces.iter_mut().enumerate() {
                    if !trace.is_empty() { 
                        let i_f32 = i as f32;
                        let h_trace = 1.0 / NUM_TRACES; 
                        // Use the SAME 10% margin as in the pulse calculation.
                        let y_base = i_f32 * h_trace + h_trace * 0.10;
                        let h_sig = h_trace * 0.80;
                        
                        // Determine the baseline using the stored polarity.
                        let y_start_end_baseline = if let Some(state_tag) = input_polarities[i] {
                            if state_tag == 0 {
                                // Active LOW: baseline is HIGH
                                y_base + h_sig
                            } else {
                                // Active HIGH: baseline is LOW
                                y_base
                            }
                        } else {
                            // Fallback to low baseline if we somehow don't have polarity info.
                            y_base
                        };
                        
                        // Insert the trip start baseline point at the beginning (index 0).
                        trace.insert(0, SinglePoint {
                            unix_time: trip_start_time,
                            point_value: y_start_end_baseline,
                        });
                        
                        // Add trip end baseline point.
                        trace.push(SinglePoint {
                            unix_time: trip_end_time,
                            point_value: y_start_end_baseline,
                        });
                    }
                }

                // Only create dataset if there's at least one trace with events (more than 2 baseline points).
                let has_events = input_traces.iter().any(|trace| trace.len() > 2);
                
                if has_events {
                    datasets.push(TimeSeriesData {
                        data_type: "StackedPulses".to_string(),
                        series_name: "INPUT".to_string(),
                        units: "Active".to_string(),
                        levels: vec![
                            "1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(),
                            "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string()
                        ],
                        time_series_points: Vec::new(),
                        multi_traces: input_traces,
                        tall_chart: true,
                    });
                }
            }  _ => {}                
        // Set of all data series to plot.
        }
    }
    datasets
}
