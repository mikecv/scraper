// Draw the time series data plots to a separate UI.
// All plots share the same time x axis.
// Pan and zoom on any one plot pan and zooms the others.
// Note that zooming is only in the time axis.

use log::info;

use eframe::egui;

use crate::colours;
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
    pub levels: Vec<String>,
    pub time_series_points: Vec<SinglePoint>,
}

// Plot state to maintain synchronized pan/zoom across multiple plots.
#[derive(Debug, Clone)]
pub struct PlotState {
    pub x_range: Option<(f64, f64)>,
    pub auto_bounds: bool,
    pub zoom_factor: f32,
    pub pan_offset: f32,
}

impl Default for PlotState {
    fn default() -> Self {
        Self {
            x_range: None,
            auto_bounds: true,
            zoom_factor: 1.0,
            pan_offset: 0.0,
        }
    }
}

// Constants for plot dimensions,
// as well as format and spacing between plots.
const PLOT_HEIGHT: f32 = 140.0;
const SPACE_BETWEEN_PLOTS: f32 = 5.0;
const MARGIN_LEFT: f32 = 55.0;
const MARGIN_RIGHT: f32 = 35.0;
const MARGIN_TOP: f32 = 30.0;
const MARGIN_BOTTOM: f32 = 40.0;
const SHOW_MARKERS: bool = false;
const LINE_THIVKNESS: f32 = 1.5;

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
    // let unique_event_types: std::collections::HashSet<String> = trip_data.iter()
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
                    let pulse_points = convert_to_pulse_data(&ev_points, trip_start_time, trip_end_time, "Digital");    
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
                    let pulse_points = convert_to_pulse_data(&ev_points, trip_start_time, trip_end_time, "Digital");    
    
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
                    let pulse_points = convert_to_pulse_data(&ev_points, trip_start_time, trip_end_time, "Digital");    
    
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
                    let pulse_points = convert_to_pulse_data(&ev_points, trip_start_time, trip_end_time, "Digital");    
    
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
                    // Filter by event type
                    .filter(|data| data.event_type == event_type)
                    .filter_map(|data| {
                        // Look for event severity in the ev_detail vector.
                        data.ev_detail.iter()
                            .find(|(tag, _)| tag == "Severity")
                            .and_then(|(_, value)| {
                                // Translate severity strings to numeric levels.
                                let numeric_level = match value.as_str() {
                                    "-" => 1.0,  // Low level
                                    "W" => 2.0,  // Warning level
                                    "C" => 3.0,  // Critical level
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
                    // For impulse data, we don't convert to pulse data
                    // We keep the original points as instantaneous events
                    
                    // Push the impulse time series events to list of datasets.
                    datasets.push(TimeSeriesData {
                        data_type: "Impulse".to_string(),
                        series_name: event_type.clone(),
                        units: "Severity".to_string(),
                        levels: vec!["Low".to_string(), "Warning".to_string(), "Critical".to_string()],
                        time_series_points: ev_points,
                    });
                }
            }            _ => info!("Event type not supported for plotting.")
        }
    }
    
    // Set of all data series to plot.
    datasets
}

// Helper function to convert single event points to pulse data.
fn convert_to_pulse_data(ev_points: &[SinglePoint], trip_start: u64, trip_end: u64, data_type: &str) -> Vec<SinglePoint> {
    let mut pulse_points = Vec::new();
    
    // Determine baseline value based on data type
    let baseline_value = if data_type == "Impulse" {
        1.0 // Impulse signal - baseline is "Low" level
    } else {
        0.0 // Digital signal - baseline is inactive
    };
    
    // Add starting point at trip start (baseline level).
    pulse_points.push(SinglePoint {
        unix_time: trip_start,
        point_value: baseline_value,
    });
    
    // Convert each event point into a pulse (rising pulse).
    for point in ev_points {
        let event_end_time = point.unix_time;
        let duration_seconds = if data_type == "Digital" {
            // For digital signals, use the point value as duration.
            point.point_value as u64
        } else {
            // For impulse signals, we want instantaneous events, so use 0 duration.
            0
        };
        
        // Calculate when the event started (going back in time by duration).
        let event_start_time = if event_end_time >= duration_seconds {
            event_end_time - duration_seconds
        } else {
            // If duration is longer than time since trip start, use trip start.
            trip_start
        };
        
        // Add point just before signal changes (still at baseline).
        if event_start_time > trip_start {
            pulse_points.push(SinglePoint {
                unix_time: event_start_time,
                point_value: baseline_value,
            });
        }
        
        // Add point where signal becomes active (rising edge).
        pulse_points.push(SinglePoint {
            unix_time: event_start_time,
            point_value: if data_type == "Digital" { 1.0 } else { point.point_value },
        });
        
        // Add point where signal is active (constant at level).
        pulse_points.push(SinglePoint {
            unix_time: event_end_time,
            point_value: if data_type == "Digital" { 1.0 } else { point.point_value },
        });

        // Add point where signal returns to baseline (falling edge).
        pulse_points.push(SinglePoint {
            unix_time: event_end_time,
            point_value: baseline_value,
        });
    }
    
    // Add ending point at trip end (baseline level).
    pulse_points.push(SinglePoint {
        unix_time: trip_end,
        point_value: baseline_value,
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
        ui.horizontal(|ui| {
            ui.heading("Time Series Plots");
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Reset Pan/Zoom").clicked() {
                    plot_state.x_range = None;
                    plot_state.auto_bounds = true;
                    plot_state.pan_offset = 0.0;
                    plot_state.zoom_factor = 1.0;
                }
            });
        });
        
        // Show trip selection status.
        match selected_trip {
            Some(trip_id) if !trip_id.is_empty() => {
                ui.label(format!("Current trip ID: {}", trip_id));
            }
            _ => info!("No trip selected."),
        }
    });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        let mut scroll_area = egui::ScrollArea::vertical();
        scroll_area = scroll_area.auto_shrink([false, false]);
        scroll_area.show(ui, |ui| {
            ui.set_max_width(ui.available_width() - 20.0);

            // Check if a trip is currently selected.
            if let Some(trip_id) = selected_trip {
                if !trip_id.is_empty() {
                    // If trip selected, and not empty, get datasets to plot.
                    let datasets = create_time_series_datasets(scraper, trip_id);

                    // Calculate overall time range for all datasets.
                    let (time_min, time_max) = calculate_time_range(&datasets);

                    for dataset in datasets {
                        // Here's the space allocation for a single plot.
                        let plot_size = egui::vec2(ui.available_width(), PLOT_HEIGHT);
                        let (plot_response, painter) = ui.allocate_painter(plot_size, egui::Sense::click_and_drag());

                        // Handle panning.
                        if plot_response.dragged() {
                            let drag_delta = plot_response.drag_delta();
                            plot_state.pan_offset += drag_delta.x;
                            plot_state.auto_bounds = false;
                        }

                        // Handle limited zooming with mouse wheel.
                        if plot_response.hovered() {
                            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
                            if scroll_delta != 0.0 {
                                let zoom_speed = 0.001;
                                let zoom_change = 1.0 + (scroll_delta * zoom_speed);
                                plot_state.zoom_factor *= zoom_change;
                                plot_state.zoom_factor = plot_state.zoom_factor.clamp(0.1, 10.0);
                                plot_state.auto_bounds = false;
                            }
                        }

                        // Draw the plot with axes.
                        draw_plot_with_axes(
                            &painter, 
                            &plot_response.rect, 
                            &dataset, 
                            time_min, 
                            time_max,
                            plot_state,
                            *dark_mode
                        );
                        
                        // Add some vertical spacing between plots.
                        ui.add_space(SPACE_BETWEEN_PLOTS);
                    }
                } else {
                    // Show a centreed message when trip is empty.
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.label(egui::RichText::new("No time series plots to display.")
                            .color(colours::ts_notices_colour(*dark_mode)));
                    });
                }
            } else {
                // Show a centreed message when no trip is selected.
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.label(egui::RichText::new("Please select a trip from the trip list to view time series plots.")
                        .color(colours::ts_notices_colour(*dark_mode)));
                });
            }
        });
    });
}

// Helper function to calculate the overall time range across all datasets.
fn calculate_time_range(datasets: &[TimeSeriesData]) -> (u64, u64) {
    let mut time_min = u64::MAX;
    let mut time_max = u64::MIN;
    
    for dataset in datasets {
        for point in &dataset.time_series_points {
            time_min = time_min.min(point.unix_time);
            time_max = time_max.max(point.unix_time);
        }
    }
    
    // If no data points, return reasonable defaults.
    if time_min == u64::MAX {
        (0, 1)
    } else {
        (time_min, time_max)
    }
}

// Helper function to draw a plot with axes.
fn draw_plot_with_axes(
    painter: &egui::Painter,
    rect: &egui::Rect,
    dataset: &TimeSeriesData,
    time_min: u64,
    time_max: u64,
    plot_state: &PlotState,
    dark_mode: bool,
) {
    // Calculate the actual plotting area.
    let plot_rect = egui::Rect::from_min_size(
        egui::pos2(rect.min.x + MARGIN_LEFT, rect.min.y + MARGIN_TOP),
        egui::vec2(
            rect.width() - MARGIN_LEFT - MARGIN_RIGHT,
            rect.height() - MARGIN_TOP - MARGIN_BOTTOM,
        ),
    );

    // Calculate panned and zoomed time range.
    let time_range = time_max - time_min;
    let zoomed_time_range = (time_range as f32 / plot_state.zoom_factor) as u64;
    let pan_pixels_to_time = zoomed_time_range as f32 / plot_rect.width();
    let pan_time_offset = (plot_state.pan_offset * pan_pixels_to_time) as i64;

    let center_time = (time_min + time_max) / 2;
    let half_zoomed_range = zoomed_time_range / 2;

    let panned_time_min = ((center_time as i64) - (half_zoomed_range as i64) - pan_time_offset).max(0) as u64;
    let panned_time_max = ((center_time as i64) + (half_zoomed_range as i64) - pan_time_offset).max(0) as u64;

    // Choose colours based on dark mode.
    let (bg_color, axis_colour, text_color) = if dark_mode {
        (egui::Color32::from_rgb(30, 30, 30), egui::Color32::LIGHT_GRAY, egui::Color32::WHITE)
    } else {
        (egui::Color32::WHITE, egui::Color32::DARK_GRAY, egui::Color32::BLACK)
    };
    
    // Draw background.
    painter.rect_filled(*rect, 4.0, bg_color);
    
    // Draw plot border/frame.
    painter.rect_stroke(*rect, 4.0, egui::Stroke::new(1.0, axis_colour), egui::epaint::StrokeKind::Inside);
    
    // Draw the main plot area background.
    painter.rect_filled(plot_rect, 0.0, if dark_mode { 
        egui::Color32::from_rgb(40, 40, 40) 
    } else { 
        egui::Color32::from_rgb(250, 250, 250) 
    });
    
    // Draw axes.
    // X-axis (bottom).
    painter.line_segment(
        [egui::pos2(plot_rect.min.x, plot_rect.max.y), 
         egui::pos2(plot_rect.max.x, plot_rect.max.y)],
        egui::Stroke::new(2.0, axis_colour),
    );
    
    // Y-axis (left).
    painter.line_segment(
        [egui::pos2(plot_rect.min.x, plot_rect.min.y), 
         egui::pos2(plot_rect.min.x, plot_rect.max.y)],
        egui::Stroke::new(2.0, axis_colour),
    );
    
    // Calculate Y-axis range for this dataset.
    let (y_min, y_max) = calculate_y_range(dataset);
    
    // Add plot title.
    let title_pos = egui::pos2(rect.center().x, rect.min.y + 15.0);
    painter.text(
        title_pos,
        egui::Align2::CENTER_CENTER,
        format!("{} ({})", dataset.series_name, dataset.units),
        egui::FontId::proportional(14.0),
        text_color,
    );
    
    // Draw X-axis tick marks and time labels (4 marks total).
    if panned_time_max > panned_time_min {
        let time_positions = [
            (panned_time_min, plot_rect.min.x),
            (panned_time_min + (panned_time_max - panned_time_min) / 3, plot_rect.min.x + plot_rect.width() / 3.0),
            (panned_time_min + 2 * (panned_time_max - panned_time_min) / 3, plot_rect.min.x + 2.0 * plot_rect.width() / 3.0),
            (panned_time_max, plot_rect.max.x),
        ];
        
        for (time_value, x_pos) in time_positions {
            // Draw tick mark.
            painter.line_segment(
                [egui::pos2(x_pos, plot_rect.max.y), 
                 egui::pos2(x_pos, plot_rect.max.y + 5.0)],
                egui::Stroke::new(1.0, axis_colour),
            );
            
            // Draw time label (actual clock time).
            painter.text(
                egui::pos2(x_pos, plot_rect.max.y + 20.0),
                egui::Align2::CENTER_CENTER,
                unix_time_to_hms(time_value),
                egui::FontId::proportional(10.0),
                text_color,
            );
        }
    }
    
    // Draw Y-axis tick marks and labels.
    if dataset.data_type == "Digital" {
        // For digital signals, only show 0 and 1.
        let positions_and_values = [
            (plot_rect.max.y, 0.0),
            (plot_rect.min.y, 1.0),
        ];
        
        for (pos_y, value) in positions_and_values {
            // Draw tick marks.
            painter.line_segment(
                [egui::pos2(plot_rect.min.x - 5.0, pos_y), 
                egui::pos2(plot_rect.min.x, pos_y)],
                egui::Stroke::new(1.0, axis_colour),
            );
            
            // Draw value label.
            painter.text(
                egui::pos2(plot_rect.min.x - 10.0, pos_y),
                egui::Align2::RIGHT_CENTER,
                format!("{:.0}", value),
                egui::FontId::proportional(10.0),
                text_color,
            );
        }
    } else if dataset.data_type == "Analog" {
        // For analog signals, show min, middle, and max.
        let positions_and_values = [
            (plot_rect.min.y, y_max),
            (plot_rect.center().y, (y_max + y_min) / 2.0),
            (plot_rect.max.y, y_min),
        ];
        
        for (pos_y, value) in positions_and_values {
            // Draw tick mark.
            painter.line_segment(
                [egui::pos2(plot_rect.min.x - 5.0, pos_y), 
                egui::pos2(plot_rect.min.x, pos_y)],
                egui::Stroke::new(1.0, axis_colour),
            );
            
            // Draw value label.
            painter.text(
                egui::pos2(plot_rect.min.x - 10.0, pos_y),
                egui::Align2::RIGHT_CENTER,
                format!("{:.1}", value),
                egui::FontId::proportional(10.0),
                text_color,
            );
        }
    } else if dataset.data_type == "Impulse" {
        // For impulse signals, show levels 0, 1, 2, 3, 4
        let positions_and_values = [
            // Don't need to display the outer tic mark text,
            // as the impact level is all that is required.
            (plot_rect.max.y - plot_rect.height() * 0.25, 1.0, "Low"),
            (plot_rect.max.y - plot_rect.height() * 0.5, 2.0, "Warning"),
            (plot_rect.max.y - plot_rect.height() * 0.75, 3.0, "Critical"),
        ];
        
        for (pos_y, _value, label) in positions_and_values {
            // Draw tick marks.
            painter.line_segment(
                [egui::pos2(plot_rect.min.x - 5.0, pos_y), 
                egui::pos2(plot_rect.min.x, pos_y)],
                egui::Stroke::new(1.0, axis_colour),
            );
            
            // Show the level name.
            painter.text(
                egui::pos2(plot_rect.min.x - 10.0, pos_y),
                egui::Align2::RIGHT_CENTER,
                label.to_string(),
                egui::FontId::proportional(10.0),
                text_color,
            );
        }
    }

    // Draw grid lines
    let grid_color = if dark_mode {
        egui::Color32::from_rgba_unmultiplied(100, 100, 100, 100)
    } else {
        egui::Color32::from_rgba_unmultiplied(200, 200, 200, 150)
    };
    let grid_stroke = egui::Stroke::new(0.5, grid_color);

    // Vertical grid lines (same X positions as time tick marks).
    if panned_time_max > panned_time_min {
        let time_positions = [
            plot_rect.min.x,
            plot_rect.min.x + plot_rect.width() / 3.0,
            plot_rect.min.x + 2.0 * plot_rect.width() / 3.0,
            plot_rect.max.x,
        ];
        
        for x_pos in time_positions {
            painter.line_segment(
                [egui::pos2(x_pos, plot_rect.min.y), 
                egui::pos2(x_pos, plot_rect.max.y)],
                grid_stroke,
            );
        }
    }

    // Horizontal grid lines (same Y positions as value tick marks).
    if dataset.data_type == "Digital" {
        let y_positions = [plot_rect.max.y, plot_rect.min.y];
        for y_pos in y_positions {
            painter.line_segment(
                [egui::pos2(plot_rect.min.x, y_pos), 
                egui::pos2(plot_rect.max.x, y_pos)],
                grid_stroke,
            );
        }
    } else if dataset.data_type == "Analog" {
        let y_positions = [plot_rect.min.y, plot_rect.center().y, plot_rect.max.y];
        for y_pos in y_positions {
            painter.line_segment(
                [egui::pos2(plot_rect.min.x, y_pos), 
                egui::pos2(plot_rect.max.x, y_pos)],
                grid_stroke,
            );
        }
    } else if dataset.data_type == "Impulse" {
        // Grid lines at levels 0, 1, 2, 3, 4
        // Level 0 (baseline)
        // Level 1 (Low)
        // Level 2 (Warning)
        // Level 3 (Critical)
        // Level 4 (top)
        let y_positions = [
            plot_rect.max.y,
            plot_rect.max.y - plot_rect.height() * 0.25,
            plot_rect.max.y - plot_rect.height() * 0.5,
            plot_rect.max.y - plot_rect.height() * 0.75,
            plot_rect.min.y,
        ];
        
        for y_pos in y_positions {
            painter.line_segment(
                [egui::pos2(plot_rect.min.x, y_pos), 
                egui::pos2(plot_rect.max.x, y_pos)],
                grid_stroke,
            );
        }
    }

    // Plot the actual data.
    plot_data_points(painter, &plot_rect, dataset, panned_time_min, panned_time_max, y_min, y_max, dark_mode);
}

// Helper function to plot the actual data points.
fn plot_data_points(
    painter: &egui::Painter,
    plot_rect: &egui::Rect,
    dataset: &TimeSeriesData,
    time_min: u64,
    time_max: u64,
    y_min: f32,
    y_max: f32,
    dark_mode: bool,
) {
    // Check for empty dataset, or no real data.
    if dataset.time_series_points.is_empty() || time_max == time_min || y_max == y_min {
        return;
    }
    
    // Choose line colour based on data type and dark mode.
    let line_color = if dataset.data_type == "Digital" {
        colours::ts_digital_colour(dark_mode)
    } else if dataset.data_type == "Analog" {
        colours::ts_analog_colour(dark_mode)
    } else if dataset.data_type == "Impulse" {
        colours::ts_impulse_colour(dark_mode)
    } else {
        // Fallback colour.
        colours::ts_digital_colour(dark_mode)
    };

    let line_stroke = egui::Stroke::new(LINE_THIVKNESS, line_color);
    
    // Convert data points to screen coordinates.
    let mut screen_points: Vec<egui::Pos2> = Vec::new();

    for point in &dataset.time_series_points {
        // Skip points outside the visible time range,
        // i.e. points that move out due to zooming.
        if point.unix_time < time_min || point.unix_time > time_max {
            continue;
        }
        
        // Convert time to X coordinate.
        let x_ratio = (point.unix_time as f64 - time_min as f64) / (time_max as f64 - time_min as f64);
        let x_pos = plot_rect.min.x + (x_ratio as f32 * plot_rect.width());
        
        // Convert value to Y coordinate (note: Y is inverted, so max value is at top).
        let y_ratio = (point.point_value - y_min) / (y_max - y_min);
        let y_pos = plot_rect.max.y - (y_ratio * plot_rect.height());
        
        screen_points.push(egui::pos2(x_pos, y_pos));
    }
    
    // Add shading for digital signals (active pulses only).
    if dataset.data_type == "Digital" {
        
        // Find the Y position for low (0) values.
        let low_y_pos = plot_rect.max.y - (0.0 - y_min) / (y_max - y_min) * plot_rect.height();
        
        // Find pairs of rising/falling edges to shade.
        for i in 0..screen_points.len() - 1 {
            let current_y = screen_points[i].y;
            
            // If current point is high (active), shade to the next point.
            if current_y < plot_rect.center().y {
                let rect = egui::Rect::from_two_pos(
                    egui::pos2(screen_points[i].x, current_y),
                    egui::pos2(screen_points[i + 1].x, low_y_pos)
                );
                painter.rect_filled(rect, 0.0, colours::ts_digital_fill_colour(dark_mode));
            }
        }
    }

    // Add special handling for impulse signals after the digital shading section.
    if dataset.data_type == "Impulse" {
        // Draw impulse markers as vertical lines from baseline to the impulse level
        // Y position for level 0.
        let baseline_y = plot_rect.max.y;
        
        for point in &dataset.time_series_points {
            // Skip points outside the visible time range
            if point.unix_time < time_min || point.unix_time > time_max {
                continue;
            }
            
            let x_ratio = (point.unix_time as f64 - time_min as f64) / (time_max as f64 - time_min as f64);
            let x_pos = plot_rect.min.x + (x_ratio as f32 * plot_rect.width());
            
            // Calculate Y position based on impulse level (0-4 scale).
            // Map the point value (1=Low, 2=Warning, 3=Critical) to the 0-4 scale,
            // Scale to 0-4 range.
            let y_ratio = point.point_value / 4.0;
            let y_pos = plot_rect.max.y - (y_ratio * plot_rect.height());
            
            // Choose color based on severity level
            let impulse_color = match point.point_value as i32 {
                1 => egui::Color32::from_rgb(255, 255, 0),
                2 => egui::Color32::from_rgb(255, 165, 0),
                3 => egui::Color32::from_rgb(255, 0, 0),
                _ => egui::Color32::GRAY,
            };
            
            // Only draw visible impulses (non-zero values).
            if point.point_value > 0.0 {
                // Draw vertical line from baseline to impulse level.
                painter.line_segment(
                    [egui::pos2(x_pos, baseline_y), egui::pos2(x_pos, y_pos)],
                    egui::Stroke::new(LINE_THIVKNESS * 2.0, impulse_color),
                );
                
                // Draw a circle at the top of each impulse.
                painter.circle_filled(egui::pos2(x_pos, y_pos), 3.0, impulse_color);
            }
        }
        
        // For impulse signals, we don't draw connecting lines, so return early.
        return;
    }

    // Draw lines connecting the points.
    for i in 1..screen_points.len() {
        painter.line_segment([screen_points[i-1], screen_points[i]], line_stroke);
    }

    // Optional.
    // Draw small circles at data points.
    // But not for digital plots as doesn't look as nice.
    if SHOW_MARKERS {
        if dataset.data_type != "Digital" {
            for point in &screen_points {
                painter.circle_filled(*point, 2.0, line_color);
            }
        }
    }
}

// Helper function to convert Unix timestamp to hh:mm:ss format.
fn unix_time_to_hms(unix_time: u64) -> String {
    // Convert Unix timestamp to time of day (UTC).
    let seconds_in_day = unix_time % 86400;
    let hours = seconds_in_day / 3600;
    let minutes = (seconds_in_day % 3600) / 60;
    let seconds = seconds_in_day % 60;
    
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

// Helper function to calculate Y-axis range for a dataset.
fn calculate_y_range(dataset: &TimeSeriesData) -> (f32, f32) {
    if dataset.time_series_points.is_empty() {
        return (0.0, 1.0);
    }
    
    // Special handling for impulse signals.
    if dataset.data_type == "Impulse" {
        // Range from 0 to 4 to accommodate impulse levels:
        // 0 = baseline, 1 = Low impulse, 2 = Warning impulse, 3 = Critical impulse.
        return (0.0, 4.0);
    }
    
    let mut y_min = f32::MAX;
    let mut y_max = f32::MIN;
    
    for point in &dataset.time_series_points {
        y_min = y_min.min(point.point_value);
        y_max = y_max.max(point.point_value);
    }
    
    // Add some padding for better visualization.
    let padding = (y_max - y_min) * 0.1;
    if padding == 0.0 {
        // For flat lines (like digital signals), add fixed padding.
        (y_min - 0.1, y_max + 0.1)
    } else {
        (y_min - padding, y_max + padding)
    }
}
