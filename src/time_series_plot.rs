// Draw the time series data plots to a separate UI.
// All plots share the same time x axis.
// Pan and zoom on any one plot pan and zooms the others.
// Note that zooming is only in the time axis.

use log::info;

use eframe::egui;

use crate::scraper::{Scraper, ScrapedData};

// SinglePoint struct.
#[derive(Debug, Clone)]
pub struct SinglePoint {
    pub unix_time: u64,
    pub point_value: f32,
}

// TimeSeriesData struct.
#[derive(Debug, Clone)]
pub struct TimeSeriesData {
    pub series_name: String,
    pub data_type: String,
    pub units: String,
    pub time_series_points: Vec<SinglePoint>,
}

// Plot state to maintain synchronized pan/zoom across multiple plots.
#[derive(Debug, Clone)]
pub struct PlotState {
    pub x_range: Option<(f64, f64)>,
    pub auto_bounds: bool,
    pub zoom_factor: f32,
    pub pan_offset: f32,
    pub is_dragging: bool,
    pub drag_start: Option<egui::Pos2>,
    pub is_selecting: bool,
    pub selection_start: Option<egui::Pos2>,
    pub selection_end: Option<egui::Pos2>,
}

impl Default for PlotState {
    fn default() -> Self {
        Self {
            x_range: None,
            auto_bounds: true,
            zoom_factor: 1.0,
            pan_offset: 0.0,
            is_dragging: false,
            drag_start: None,
            is_selecting: false,
            selection_start: None,
            selection_end: None,
        }
    }
}

// Constants for plot dimensions,
// as well as spacing between plots.
const PLOT_HEIGHT: f32 = 200.0;
const SPACE_BETWEEN_PLOTS: f32 = 5.0;

// Function to create the data sets to plot.
fn create_time_series_datasets(scraper: &Scraper, selected_trip: &str) -> Vec<TimeSeriesData> {
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
            time_series_points: speed_points,
        });
    }

    // Process each unique event type once to create combined datasets.
    // That is a combined dataset for each type of event.
    let unique_event_types: std::collections::HashSet<String> = trip_data.iter()
        .map(|data| data.event_type.clone())
        .collect();

    for event_type in unique_event_types {
        match event_type.as_str() {
            "ENGINETEMP" => {
                // Get all points for this event type in the selected trip
                let ev_points: Vec<SinglePoint> = trip_data.iter()
                    .filter(|data| data.event_type == event_type) // Filter by event type
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
                    // Convert single points to pulse data
                    let pulse_points = convert_to_pulse_data(&ev_points, trip_start_time, trip_end_time);
    
                    // Push the digital time series events to list of datasets.
                    datasets.push(TimeSeriesData {
                        data_type: "Digital".to_string(),
                        series_name: event_type.clone(),
                        units: "Active".to_string(),
                        time_series_points: pulse_points,
                    });
                }
            }
            "LOWCOOLANT" => {
                // Get all points for this event type in the selected trip
                let ev_points: Vec<SinglePoint> = trip_data.iter()
                    .filter(|data| data.event_type == event_type) // Filter by event type
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
                    // Convert single points to pulse data
                    let pulse_points = convert_to_pulse_data(&ev_points, trip_start_time, trip_end_time);
    
                    // Push the digital time series events to list of datasets.
                    datasets.push(TimeSeriesData {
                        data_type: "Digital".to_string(),
                        series_name: event_type.clone(),
                        units: "Active".to_string(),
                        time_series_points: pulse_points,
                    });
                }
            }
            "OILPRESSURE" => {
                // Get all points for this event type in the selected trip
                let ev_points: Vec<SinglePoint> = trip_data.iter()
                    .filter(|data| data.event_type == event_type) // Filter by event type
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
                    // Convert single points to pulse data
                    let pulse_points = convert_to_pulse_data(&ev_points, trip_start_time, trip_end_time);
    
                    // Push the digital time series events to list of datasets.
                    datasets.push(TimeSeriesData {
                        data_type: "Digital".to_string(),
                        series_name: event_type.clone(),
                        units: "Active".to_string(),
                        time_series_points: pulse_points,
                    });
                }
            }
            "OVERSPEED" => {
                // Get all points for this event type in the selected trip
                let ev_points: Vec<SinglePoint> = trip_data.iter()
                    .filter(|data| data.event_type == event_type) // Filter by event type
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
                    // Convert single points to pulse data
                    let pulse_points = convert_to_pulse_data(&ev_points, trip_start_time, trip_end_time);
    
                    // Push the digital time series events to list of datasets.
                    datasets.push(TimeSeriesData {
                        data_type: "Digital".to_string(),
                        series_name: event_type.clone(),
                        units: "Active".to_string(),
                        time_series_points: pulse_points,
                    });
                }
            }
            _ => info!("Event type not supported for plotting.")
        }
    }
    
    // Set of all data series to plot.
    datasets
}

// Helper function to convert single event points to pulse data.
fn convert_to_pulse_data(ev_points: &[SinglePoint], trip_start: u64, trip_end: u64) -> Vec<SinglePoint> {
    let mut pulse_points = Vec::new();
    
    // Add starting point at trip start (signal not active)
    pulse_points.push(SinglePoint {
        unix_time: trip_start,
        point_value: 0.0,
    });
    
    // Convert each event point into a pulse (rising pulse).
    for point in ev_points {
        let event_end_time = point.unix_time;
        let duration_seconds = point.point_value as u64;
        
        // Calculate when the event started (going back in time by duration).
        let event_start_time = if event_end_time >= duration_seconds {
            event_end_time - duration_seconds
        } else {
            // If duration is longer than time since trip start, use trip start.
            // Although this shouldn't happen.
            trip_start 
        };
        
        // Add point just before signal goes active (still 0).
        if event_start_time > trip_start {
            pulse_points.push(SinglePoint {
                unix_time: event_start_time.saturating_sub(1),
                point_value: 0.0,
            });
        }
        
        // Add point where signal becomes active (rising edge).
        pulse_points.push(SinglePoint {
            unix_time: event_start_time,
            point_value: 1.0,
        });
        
        // Add point where signal becomes inactive (falling edge).
        pulse_points.push(SinglePoint {
            unix_time: event_end_time,
            point_value: 0.0,
        });
    }
    
    // Add ending point at trip end (signal inactive).
    pulse_points.push(SinglePoint {
        unix_time: trip_end,
        point_value: 0.0,
    });
    
    // Sort by time to ensure correct order.
    pulse_points.sort_by_key(|p| p.unix_time);
    
    // Remove any duplicate time points (keep the last value for each time).
    pulse_points.dedup_by(|a, b| {
        if a.unix_time == b.unix_time {
            true
        } else {
            false
        }
    });
    
    pulse_points
}

// Function to plot time series data.
// This is called from the ui.rs file where the UI panel is defined and created.
pub fn plot_time_series_data(
    ui: &mut egui::Ui,
    scraper: &Scraper,
    selected_trip: &Option<String>,
    plot_state: &mut PlotState,
    dark_mode: &bool,
) {
    // Create a fixed top panel for info.
    egui::TopBottomPanel::top("info_panel").show_inside(ui, |ui| {
        ui.heading("Time Series Plots");
    });

    // Create a central, vertically scrollable area for the plots.
    egui::CentralPanel::default().show_inside(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {

            // Iterate through datasets and plot each one in turn.
            let datasets = create_time_series_datasets(scraper, selected_trip.as_ref().unwrap());

            for dataset in datasets {

                // Here's the space allocation for a single plot.
                let plot_size = egui::vec2(ui.available_width(), PLOT_HEIGHT);
                let (plot_response, painter) = ui.allocate_painter(plot_size, egui::Sense::hover());

                // Now you would use the 'painter' to draw your plot.
                // You can also use the 'plot_response' to handle user input like hover.

                // Example of drawing a simple rectangle to represent a plot:
                painter.rect_filled(plot_response.rect, 0.0, egui::Color32::from_rgb(50, 100, 150));
                
                // Add some vertical spacing between plots.
                ui.add_space(SPACE_BETWEEN_PLOTS);
            }

            // To demonstrate multiple plots, let's create a few placeholder plots.
            for i in 0..5 {
                 // The allocation must be done for each plot individually.
                let plot_size = egui::vec2(ui.available_width(), PLOT_HEIGHT);
                let (plot_response, painter) = ui.allocate_painter(plot_size, egui::Sense::hover());
                
                // Example of drawing a simple placeholder plot.
                painter.rect_filled(plot_response.rect, 0.0, egui::Color32::from_rgb(50 + i * 20, 100, 150));
                ui.label(format!("Placeholder Plot {}", i));
                
                // Add some vertical spacing between plots.
                ui.add_space(SPACE_BETWEEN_PLOTS);
            }
        });
    });
}
