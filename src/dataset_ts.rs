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
                    });
                }
            } _ => {}
        }
    }
    
    // Set of all data series to plot.
    datasets
}
