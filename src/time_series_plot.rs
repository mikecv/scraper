use log::warn;

use eframe::egui;

use crate::colours;
use crate::helpers_ts;
use crate::dataset_ts;
use crate::scraper::Scraper;

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
    pub multi_traces: Vec<Vec<SinglePoint>>,
    pub tall_chart: bool,
}

// Plot state to maintain synchronized pan/zoom across multiple plots.
// Unix time at cursor position.
#[derive(Debug, Clone)]
pub struct PlotState {
    pub x_range: Option<(f64, f64)>,
    pub auto_bounds: bool,
    pub zoom_factor: f32,
    pub pan_offset: f32,
    pub pan_zoom_enabled: bool,
    pub cursor_enabled: bool,
    pub cursor_time: Option<u64>,
    pub current_trip: Option<String>,
}

// Default plot effect states.
impl Default for PlotState {
    fn default() -> Self {
        Self {
            x_range: None,
            auto_bounds: true,
            zoom_factor: 1.0,
            pan_offset: 0.0,
            pan_zoom_enabled: false,
            cursor_enabled: false,
            cursor_time: None,
            current_trip: None,
        }
    }
}

// Constants for plot dimensions,
// as well as format and spacing between plots.
const PLOT_HEIGHT: f32 = 140.0;
const PLOT_HEIGHT_TALL: f32 = 250.0;
const SPACE_BETWEEN_PLOTS: f32 = 5.0;
const MARGIN_LEFT: f32 = 55.0;
const MARGIN_RIGHT: f32 = 35.0;
const MARGIN_TOP: f32 = 30.0;
const MARGIN_BOTTOM: f32 = 40.0;
const SHOW_MARKERS: bool = false;
const LINE_THICKNESS: f32 = 1.5;

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
                // Reset button - only show when there's actual pan/zoom state to reset.
                if !plot_state.auto_bounds || plot_state.zoom_factor != 1.0 || plot_state.pan_offset != 0.0 {
                    if ui.button("Reset View").clicked() {
                        plot_state.x_range = None;
                        plot_state.auto_bounds = true;
                        plot_state.pan_offset = 0.0;
                        plot_state.zoom_factor = 1.0;
                    }
                }
                
                // Toggle button for time cursor functionality.
                let cursor_button_text = if plot_state.cursor_enabled {
                    "Cursor ON"
                } else {
                    "Cursor OFF"
                };

                let (cursor_button_colour, cursor_text_colour) = if plot_state.cursor_enabled {
                    (colours::ts_enabled_button_colour(*dark_mode), colours::ts_enabled_button_text_colour(*dark_mode))
                } else {
                    (colours::ts_disabled_button_colour(*dark_mode), colours::ts_disabled_button_text_colour(*dark_mode))
                };

                let cursor_button = egui::Button::new(egui::RichText::new(cursor_button_text).color(cursor_text_colour)).fill(cursor_button_colour);
                if ui.add(cursor_button).clicked() {
                    // If enabling cursor, disable pan/zoom.
                    if !plot_state.cursor_enabled {
                        plot_state.pan_zoom_enabled = false;
                    }
                    plot_state.cursor_enabled = !plot_state.cursor_enabled;
                    // Initialize cursor at centre when enabling.
                    if plot_state.cursor_enabled && plot_state.cursor_time.is_none() {
                        plot_state.cursor_time = None;
                    }
                }
                
                // Toggle button for pan/zoom functionality.
                let button_text = if plot_state.pan_zoom_enabled {
                    "Pan/Zoom ON"
                } else {
                    "Pan/Zoom OFF"
                };

                let (button_colour, text_colour) = if plot_state.pan_zoom_enabled {
                    (colours::ts_enabled_button_colour(*dark_mode), colours::ts_enabled_button_text_colour(*dark_mode))
                } else {
                    (colours::ts_disabled_button_colour(*dark_mode), colours::ts_disabled_button_text_colour(*dark_mode))
                };

                let button = egui::Button::new(egui::RichText::new(button_text).color(text_colour)).fill(button_colour);
                if ui.add(button).clicked() {
                    // If enabling pan/zoom, disable cursor AND reset view to full.
                    if !plot_state.pan_zoom_enabled {
                        plot_state.cursor_enabled = false;
                        // Reset to full view when enabling pan/zoom.
                        plot_state.x_range = None;
                        plot_state.auto_bounds = true;
                        plot_state.pan_offset = 0.0;
                        plot_state.zoom_factor = 1.0;
                    }
                    plot_state.pan_zoom_enabled = !plot_state.pan_zoom_enabled;
                }
            });
        });
        
        // Show trip selection status and pan/zoom hint.
        ui.horizontal(|ui| {
            match selected_trip {
                Some(trip_id) if !trip_id.is_empty() => {
                    ui.label(format!("Current trip ID: {}", trip_id));
                }
                _ => (),
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Show cursor time if cursor is enabled and positioned.
                if plot_state.cursor_enabled {
                    if let Some(cursor_time) = plot_state.cursor_time {
                        ui.label(egui::RichText::new(format!("Cursor: {}", helpers_ts::unix_time_to_hms(cursor_time)))
                            .color(colours::ts_notices_colour(*dark_mode))
                            .size(12.0));
                    } else {
                        ui.label(egui::RichText::new("Click on a plot to place cursor")
                            .color(colours::ts_notices_colour(*dark_mode))
                            .size(12.0));
                    }
                }
                // Show helpful hint when pan/zoom is enabled.
                else if plot_state.pan_zoom_enabled {
                    ui.label(egui::RichText::new("Drag to pan, scroll wheel to zoom")
                        .color(colours::ts_notices_colour(*dark_mode))
                        .size(12.0));
                }
            });
        });
    });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        let mut scroll_area = egui::ScrollArea::vertical();
        scroll_area = scroll_area.auto_shrink([false, false]);
        scroll_area.show(ui, |ui| {
            ui.set_max_width(ui.available_width() - 20.0);

            // Check if a trip is currently selected.
            if let Some(trip_id) = selected_trip {
                if !trip_id.is_empty() {
                    // Check if trip has changed and reset state if it has.
                    if plot_state.current_trip.as_ref() != Some(trip_id) {
                        plot_state.x_range = None;
                        plot_state.auto_bounds = true;
                        plot_state.zoom_factor = 1.0;
                        plot_state.pan_offset = 0.0;
                        plot_state.pan_zoom_enabled = false;
                        plot_state.cursor_enabled = false;
                        plot_state.cursor_time = None;
                        plot_state.current_trip = Some(trip_id.clone());
                    }
        
                    // If trip selected, and not empty, get datasets to plot.
                    let datasets = dataset_ts::create_time_series_datasets(scraper, trip_id);

                    // Calculate overall time range for all datasets.
                    let (time_min, time_max) = helpers_ts::calculate_time_range(&datasets);

                    // Initialize cursor to centre time if not set.
                    if plot_state.cursor_enabled && plot_state.cursor_time.is_none() {
                        plot_state.cursor_time = Some((time_min + time_max) / 2);
                    }

                    for dataset in datasets {
                        // Here's the space allocation for a single plot.
                        // There are 2 sizes for plots - normal and tall.
                        // The dataset attribute 'tall_chart' signigies which hight to use.
                        let mut plot_size = egui::vec2(ui.available_width(), PLOT_HEIGHT);
                        if dataset.tall_chart == true {
                            plot_size = egui::vec2(ui.available_width(), PLOT_HEIGHT_TALL);
                        }
                        let (plot_response, painter) = ui.allocate_painter(plot_size, egui::Sense::click_and_drag());

                        // Handle cursor interaction (takes priority when enabled).
                        if plot_state.cursor_enabled {
                            // Calculate the plot rect for cursor positioning.
                            let rect = plot_response.rect;
                            let plot_rect = egui::Rect::from_min_size(
                                egui::pos2(rect.min.x + MARGIN_LEFT, rect.min.y + MARGIN_TOP),
                                egui::vec2(
                                    rect.width() - MARGIN_LEFT - MARGIN_RIGHT,
                                    rect.height() - MARGIN_TOP - MARGIN_BOTTOM,
                                ),
                            );

                            // Calculate current time range (considering pan/zoom).
                            let (panned_time_min, panned_time_max) = if !plot_state.auto_bounds || plot_state.zoom_factor != 1.0 || plot_state.pan_offset != 0.0 {
                                let time_range = time_max - time_min;
                                let zoomed_time_range = (time_range as f32 / plot_state.zoom_factor) as u64;
                                let pan_pixels_to_time = zoomed_time_range as f32 / plot_rect.width();
                                let pan_time_offset = (plot_state.pan_offset * pan_pixels_to_time) as i64;

                                let centre_time = (time_min + time_max) / 2;
                                let half_zoomed_range = zoomed_time_range / 2;

                                let panned_time_min = ((centre_time as i64) - (half_zoomed_range as i64) - pan_time_offset).max(0) as u64;
                                let panned_time_max = ((centre_time as i64) + (half_zoomed_range as i64) - pan_time_offset).max(0) as u64;
                                
                                (panned_time_min, panned_time_max)
                            } else {
                                (time_min, time_max)
                            };

                            // Handle cursor dragging and clicking.
                            if (plot_response.dragged() || plot_response.clicked()) && plot_response.hover_pos().is_some() {
                                let hover_pos = plot_response.hover_pos().unwrap();
                                
                                // Only update cursor if within plot area.
                                if hover_pos.x >= plot_rect.min.x && hover_pos.x <= plot_rect.max.x {
                                    // Convert X position to time.
                                    let x_ratio = (hover_pos.x - plot_rect.min.x) / plot_rect.width();
                                    let cursor_time = panned_time_min + ((panned_time_max - panned_time_min) as f32 * x_ratio) as u64;
                                    plot_state.cursor_time = Some(cursor_time);
                                }
                            }
                        }

                        // Only handle panning and zooming if enabled and cursor is disabled.
                        else if plot_state.pan_zoom_enabled {
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
                    // Show a centred message when trip is empty.
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.label(egui::RichText::new("No time series plots to display.")
                            .color(colours::ts_notices_colour(*dark_mode)));
                    });
                }
            } else {
                // Show a centred message when no trip is selected.
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.label(egui::RichText::new("Please select a trip to view time series plots.")
                        .color(colours::ts_notices_colour(*dark_mode)));
                });
            }
        });
    });
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

    // Calculate panned and zoomed time range (only if pan/zoom is enabled).
    let (panned_time_min, panned_time_max) = if !plot_state.auto_bounds || plot_state.zoom_factor != 1.0 || plot_state.pan_offset != 0.0 {
        let time_range = time_max - time_min;
        let zoomed_time_range = (time_range as f32 / plot_state.zoom_factor) as u64;
        let pan_pixels_to_time = zoomed_time_range as f32 / plot_rect.width();
        let pan_time_offset = (plot_state.pan_offset * pan_pixels_to_time) as i64;

        let centre_time = (time_min + time_max) / 2;
        let half_zoomed_range = zoomed_time_range / 2;

        let panned_time_min = ((centre_time as i64) - (half_zoomed_range as i64) - pan_time_offset).max(0) as u64;
        let panned_time_max = ((centre_time as i64) + (half_zoomed_range as i64) - pan_time_offset).max(0) as u64;
        
        (panned_time_min, panned_time_max)
    } else {
        // Use full time range when no pan/zoom state exists.
        (time_min, time_max)
    };

    // Set colours for plot background, axes, and text.
    let bg_colour = colours::plot_area_colour(dark_mode);
    let axis_colour = colours::plot_axis_colour(dark_mode);
    let text_colour = colours::plot_text_colour(dark_mode);

    // Draw background.
    painter.rect_filled(*rect, 4.0, bg_colour);
    
    // Draw plot border/frame.
    painter.rect_stroke(*rect, 4.0, egui::Stroke::new(1.0, axis_colour), egui::epaint::StrokeKind::Inside);
    
    // Draw the main plot area background.
    painter.rect_filled(plot_rect, 0.0, colours::plot_bkgnd_colour(dark_mode));

    // Draw axes.
    // X-axis (Time at bottom).
    painter.line_segment(
        [egui::pos2(plot_rect.min.x, plot_rect.max.y), 
         egui::pos2(plot_rect.max.x, plot_rect.max.y)],
        egui::Stroke::new(2.0, axis_colour),
    );
    
    // Y-axis (left side).
    painter.line_segment(
        [egui::pos2(plot_rect.min.x, plot_rect.min.y), 
         egui::pos2(plot_rect.min.x, plot_rect.max.y)],
        egui::Stroke::new(2.0, axis_colour),
    );
    
    // Calculate Y-axis range for this dataset.
    let (y_min, y_max) = helpers_ts::calculate_y_range(dataset);
    
    // Add plot title.
    let title_pos = egui::pos2(rect.center().x, rect.min.y + 15.0);
    painter.text(
        title_pos,
        egui::Align2::CENTER_CENTER,
        format!("{} ({})", dataset.series_name, dataset.units),
        egui::FontId::proportional(14.0),
        text_colour,
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
                egui::pos2(x_pos, plot_rect.max.y + 23.0),
                egui::Align2::CENTER_CENTER,
                helpers_ts::unix_time_to_hms(time_value),
                egui::FontId::proportional(10.0),
                text_colour,
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
                text_colour,
            );
        }

    } else if dataset.data_type == "MultiDigital" || dataset.data_type == "ImpulseDigitalCombo" {
        // For dual digital signals, show each level with its label.
        if !dataset.levels.is_empty() {
            let total_levels = dataset.levels.len();
            
            // Draw baseline (0).
            painter.line_segment(
                [egui::pos2(plot_rect.min.x - 5.0, plot_rect.max.y), 
                egui::pos2(plot_rect.min.x, plot_rect.max.y)],
                egui::Stroke::new(1.0, axis_colour),
            );
            
            painter.text(
                egui::pos2(plot_rect.min.x - 10.0, plot_rect.max.y),
                egui::Align2::RIGHT_CENTER,
                "0".to_string(),
                egui::FontId::proportional(10.0),
                text_colour,
            );
            
            // Draw each level.
            for (index, level_name) in dataset.levels.iter().enumerate() {
                let level_value = (index + 1) as f32;
                let y_ratio = level_value / total_levels as f32;
                let pos_y = plot_rect.max.y - (y_ratio * plot_rect.height());
                
                // Draw tick mark.
                painter.line_segment(
                    [egui::pos2(plot_rect.min.x - 5.0, pos_y), 
                    egui::pos2(plot_rect.min.x, pos_y)],
                    egui::Stroke::new(1.0, axis_colour),
                );
                
                // Draw level name.
                painter.text(
                    egui::pos2(plot_rect.min.x - 10.0, pos_y),
                    egui::Align2::RIGHT_CENTER,
                    level_name.clone(),
                    egui::FontId::proportional(10.0),
                    text_colour,
                );
            }
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
                text_colour,
            );
        }
    } else if dataset.data_type == "Impulse" {
        // For impulse signals, show levels dynamically based on dataset levels.
        if !dataset.levels.is_empty() {
            let total_levels = dataset.levels.len() + 1;
            
            // Create positions for each level.
            for (index, level_name) in dataset.levels.iter().enumerate() {
                let level_value = index + 1;
                let y_ratio = level_value as f32 / total_levels as f32;
                let pos_y = plot_rect.max.y - (y_ratio * plot_rect.height());
                
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
                    level_name.clone(),
                    egui::FontId::proportional(10.0),
                    text_colour,
                );
            }
            
            // Add baseline level (0) tick mark.
            // But don't label the tick mark, it is just a separator.
            painter.line_segment(
                [egui::pos2(plot_rect.min.x - 5.0, plot_rect.max.y), 
                egui::pos2(plot_rect.min.x, plot_rect.max.y)],
                egui::Stroke::new(1.0, axis_colour),
            );
        }
    }

    // Draw grid lines.
    let grid_colour = colours::ts_grid_lines_colour(dark_mode);
    let grid_stroke = egui::Stroke::new(0.5, grid_colour);

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
    } else if dataset.data_type == "MultiDigital" || dataset.data_type == "ImpulseDigitalCombo" {
        // Grid lines for each level.
        if !dataset.levels.is_empty() {
            let total_levels = dataset.levels.len();
            let mut y_positions = Vec::new();
            
            // Add baseline.
            y_positions.push(plot_rect.max.y);
            
            // Add positions for each level.
            for level_index in 1..=total_levels {
                let y_ratio = level_index as f32 / total_levels as f32;
                let y_pos = plot_rect.max.y - (y_ratio * plot_rect.height());
                y_positions.push(y_pos);
            }
            
            for y_pos in y_positions {
                painter.line_segment(
                    [egui::pos2(plot_rect.min.x, y_pos), 
                    egui::pos2(plot_rect.max.x, y_pos)],
                    grid_stroke,
                );
            }
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
        // Grid lines based on actual number of levels in the dataset.
        if !dataset.levels.is_empty() {
            let total_levels = dataset.levels.len() + 1;
            let mut y_positions = Vec::new();
            
            // Add baseline position.
            y_positions.push(plot_rect.max.y);
            
            // Add positions for each named level.
            for level_index in 1..total_levels {
                let y_ratio = level_index as f32 / total_levels as f32;
                let y_pos = plot_rect.max.y - (y_ratio * plot_rect.height());
                y_positions.push(y_pos);
            }
            
            // Add top position.
            y_positions.push(plot_rect.min.y);
            
            for y_pos in y_positions {
                painter.line_segment(
                    [egui::pos2(plot_rect.min.x, y_pos), 
                    egui::pos2(plot_rect.max.x, y_pos)],
                    grid_stroke,
                );
            }
        }
    }

    // Plot the actual data.
    plot_data_points(painter, &plot_rect, dataset, panned_time_min, panned_time_max, y_min, y_max, dark_mode);

    // Draw cursor line if enabled and positioned.
    if plot_state.cursor_enabled {
        if let Some(cursor_time) = plot_state.cursor_time {
            // Only draw if cursor is within visible time range.
            if cursor_time >= panned_time_min && cursor_time <= panned_time_max {
                // Calculate cursor X position.
                let x_ratio = (cursor_time as f64 - panned_time_min as f64) / (panned_time_max as f64 - panned_time_min as f64);
                let cursor_x = plot_rect.min.x + (x_ratio as f32 * plot_rect.width());
                
                // Choose cursor colour (bright and visible).
                let cursor_colour = colours::ts_cursor_colour(dark_mode);
                
                // Draw vertical cursor line (thin).
                painter.line_segment(
                    [egui::pos2(cursor_x, plot_rect.min.y), 
                     egui::pos2(cursor_x, plot_rect.max.y)],
                    egui::Stroke::new(1.0, cursor_colour),
                );
                
                // Draw time label at the bottom of the cursor (between plot and time scale).
                let time_text = helpers_ts::unix_time_to_hms(cursor_time);
                let label_bg = colours::ts_cursor_label_colour(dark_mode);
                
                // Calculate label size and position.
                let font_id = egui::FontId::proportional(10.0);
                let galley = painter.layout_no_wrap(time_text.clone(), font_id.clone(), text_colour);
                let label_width = galley.rect.width() + 8.0;
                let label_height = galley.rect.height() + 4.0;
                
                // Position label below plot, between the X-axis and time labels.
                let label_rect = egui::Rect::from_center_size(
                    egui::pos2(cursor_x, plot_rect.max.y + 10.0),
                    egui::vec2(label_width, label_height)
                );
                
                // Draw label background.
                painter.rect_filled(label_rect, 3.0, label_bg);
                painter.rect_stroke(label_rect, 3.0, egui::Stroke::new(1.0, cursor_colour), egui::epaint::StrokeKind::Inside);
                
                // Draw label text.
                painter.text(
                    label_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    time_text,
                    font_id,
                    text_colour,
                );
            }
        }
    }
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
    if time_max == time_min || y_max == y_min {
        return;
    }

    // Special handling for multi-plot each trace separately.
    if dataset.data_type == "MultiDigital" {
        let line_colour = colours::ts_digital_colour(dark_mode);
        let line_stroke = egui::Stroke::new(LINE_THICKNESS, line_colour);
        
        // Plot each trace independently.
        for trace in &dataset.multi_traces {
            if trace.is_empty() {
                continue;
            }
            
            // Convert data points to screen coordinates for this trace.
            let mut screen_points: Vec<egui::Pos2> = Vec::new();
            
            for point in trace {
                // Skip points outside the visible time range.
                if point.unix_time < time_min || point.unix_time > time_max {
                    continue;
                }
                
                // Convert time to X coordinate.
                let x_ratio = (point.unix_time as f64 - time_min as f64) / (time_max as f64 - time_min as f64);
                let x_pos = plot_rect.min.x + (x_ratio as f32 * plot_rect.width());
                
                // Convert value to Y coordinate.
                let y_ratio = (point.point_value - y_min) / (y_max - y_min);
                let y_pos = plot_rect.max.y - (y_ratio * plot_rect.height());
                
                screen_points.push(egui::pos2(x_pos, y_pos));
            }
            
            // Add shading for active regions in this trace.
            let low_y_pos = plot_rect.max.y;
            if screen_points.len() > 1 {
                for i in 0..screen_points.len() - 1 {
                    // Find corresponding point value.
                    let point_value = trace.iter()
                        .filter(|p| p.unix_time >= time_min && p.unix_time <= time_max)
                        .nth(i)
                        .map(|p| p.point_value)
                        .unwrap_or(0.0);
                    
                    // If point value is non-zero (active), shade to baseline.
                    if point_value > 0.0 {
                        let rect = egui::Rect::from_two_pos(
                            egui::pos2(screen_points[i].x, screen_points[i].y),
                            egui::pos2(screen_points[i + 1].x, low_y_pos)
                        );
                        painter.rect_filled(rect, 0.0, colours::ts_digital_fill_colour(dark_mode));
                    }
                }
            }
            
            // Draw lines connecting the points for this trace,
            for i in 1..screen_points.len() {
                painter.line_segment([screen_points[i-1], screen_points[i]], line_stroke);
            }
        }
        
        return;
    }

    // Special handling for stacked pulses (as used for INPUT events).
    else if dataset.data_type == "StackedPulses" {
        
        // Levels for y-ticks.
        let total_levels = dataset.levels.len();

        // Set colours for plot axes, and text.
        let axis_colour = colours::plot_axis_colour(dark_mode);
        let text_colour = colours::plot_text_colour(dark_mode);

        // Draw each level.
        for (index, level_name) in dataset.levels.iter().enumerate() {
            // Calculate the center Y position of this trace's band
            let band_bottom = index as f32 / total_levels as f32;
            let band_top = (index + 1) as f32 / total_levels as f32;
            let band_center = (band_bottom + band_top) / 2.0;
            
            let pos_y = plot_rect.max.y - (band_center * plot_rect.height());
            
            // Draw tick mark at the centered position
            painter.line_segment(
                [egui::pos2(plot_rect.min.x - 5.0, pos_y), 
                egui::pos2(plot_rect.min.x, pos_y)],
                egui::Stroke::new(1.0, axis_colour),
            );
            
            // Draw level name at the centered position
            painter.text(
                egui::pos2(plot_rect.min.x - 10.0, pos_y),
                egui::Align2::RIGHT_CENTER,
                level_name.clone(),
                egui::FontId::proportional(10.0),
                text_colour,
            );
        }

        // Draw grid lines.
        let grid_colour = colours::ts_grid_lines_colour(dark_mode);
        let grid_stroke = egui::Stroke::new(0.5, grid_colour);

        // Grid lines for each stacked level.
        let total_levels = dataset.levels.len();
        let mut y_positions = Vec::new();
        
        // Add baseline at bottom.
        y_positions.push(plot_rect.max.y);
        
        // Add positions for each level.
        for level_index in 1..=total_levels {
            let level_value = level_index as f32;
            let y_ratio = level_value / total_levels as f32;
            let y_pos = plot_rect.max.y - (y_ratio * plot_rect.height());
            y_positions.push(y_pos);
        }
        
        // Draw grid line for each position.
        for y_pos in y_positions {
            painter.line_segment(
                [egui::pos2(plot_rect.min.x, y_pos), 
                egui::pos2(plot_rect.max.x, y_pos)],
                grid_stroke,
            );
        }

        let time_range = (time_max - time_min) as f64;
        let y_range = (y_max - y_min) as f64;
        
        // This function must be called only if time_range and y_range are non-zero.
        if time_range == 0.0 || y_range == 0.0 {
            return;
        }

        for trace in &dataset.multi_traces {
            
            if trace.is_empty() { 
                continue; 
            }

            // Determine Polarity for Colour based on logic mode.
            // The trace points are always structured: [TripStart, BaselineBefore, PulseActive, ...].
            let is_active_high = trace.len() >= 3 && trace[2].point_value > trace[1].point_value;
            
            let impulse_colour = if is_active_high {
                colours::stacked_digital_hi_colour(dark_mode)
            } else {
                colours::stacked_digital_lo_colour(dark_mode)
            };
            
            let line_stroke = egui::Stroke::new(LINE_THICKNESS, impulse_colour);

            // Convert SinglePoints to screen coordinates with time range filtering.
            // Store both position AND original value for shading logic.
            // Using a different name to avoid conflicts with other plot types.
            let stacked_points: Vec<(egui::Pos2, f32)> = trace.iter()
                .filter(|p| {
                    // Only include points within the visible time range
                    p.unix_time >= time_min && p.unix_time <= time_max
                })
                .map(|p| {
                    // Inline X mapping: Scales data to normalized plot width, then offsets by plot start.
                    let x_normalized = (p.unix_time as f64 - time_min as f64) / time_range;
                    let x_pos = plot_rect.left() + (plot_rect.width() * x_normalized as f32);

                    // Inline Y mapping: Scales data to normalized plot height, then maps to inverted screen space.
                    let y_normalized = (p.point_value as f64 - y_min as f64) / y_range;
                    let y_pos = plot_rect.bottom() - (plot_rect.height() * y_normalized as f32);
                    
                    (egui::pos2(x_pos, y_pos), p.point_value)
                })
                .collect();
            
            // Add shading for active pulses.
            // For active HIGH: shade when point_value is at the high level.
            // For active LOW: shade when point_value is at the low level.
            if stacked_points.len() > 1 {
                // Determine baseline Y position for this trace.
                let baseline_value = if is_active_high {
                    // For active HIGH, baseline is the low value (first point after trip start).
                    trace.get(0).map(|p| p.point_value).unwrap_or(0.0)
                } else {
                    // For active LOW, baseline is the high value (first point after trip start).
                    trace.get(0).map(|p| p.point_value).unwrap_or(0.0)
                };
                
                let baseline_y_normalized = (baseline_value as f64 - y_min as f64) / y_range;
                let baseline_y_pos = plot_rect.bottom() - (plot_rect.height() * baseline_y_normalized as f32);
                
                for i in 0..stacked_points.len() - 1 {
                    let (curr_pos, curr_value) = stacked_points[i];
                    let (next_pos, _) = stacked_points[i + 1];
                    
                    // Determine if this segment should be shaded.
                    let should_shade = if is_active_high {
                        // For active HIGH: shade when value is above baseline.
                        curr_value > baseline_value
                    } else {
                        // For active LOW: shade when value is below baseline.
                        curr_value < baseline_value
                    };
                    
                    if should_shade {
                        let rect = egui::Rect::from_two_pos(
                            egui::pos2(curr_pos.x, curr_pos.y),
                            egui::pos2(next_pos.x, baseline_y_pos)
                        );
                        
                        // Use appropriate fill colour based on polarity.
                        let fill_colour = if is_active_high {
                            colours::stacked_digital_hi_fill_colour(dark_mode)
                        } else {
                            colours::stacked_digital_lo_fill_colour(dark_mode)
                        };
                        
                        painter.rect_filled(rect, 0.0, fill_colour);
                    }
                }
            }
            
            // Draw lines connecting the points (the actual pulse shape).
            // This draws the outline of the digital signal.
            for i in 1..stacked_points.len() {
                painter.line_segment([stacked_points[i-1].0, stacked_points[i].0], line_stroke);
            }
        }
    }

    // Special handling for ImpulseDigitalCombo - plot each trace separately.
    if dataset.data_type == "ImpulseDigitalCombo" {
        let line_colour_digital = colours::ts_digital_colour(dark_mode);
        let line_colour_impulse = colours::ts_xsidle_impulse_colour(dark_mode);
        let line_stroke_digital = egui::Stroke::new(LINE_THICKNESS, line_colour_digital);
        let line_stroke_impulse = egui::Stroke::new(LINE_THICKNESS * 2.0, line_colour_impulse);

        // Get min/max Y for shading calculation.
        let low_y_pos = plot_rect.max.y; 
        let baseline_y = plot_rect.max.y;
        
        // XSIDLE Digital Pulse (Index 0 in multi_traces)
        // So that XSIDLESTART pulses not overshadowed by XSIDLE pulses have them at the back.
        let digital_trace = &dataset.multi_traces[0];
        
        // Convert data points to screen coordinates for this trace.
        let mut screen_points: Vec<egui::Pos2> = Vec::new();
        
        for point in digital_trace {
            // Skip points outside the visible time range.
            if point.unix_time < time_min || point.unix_time > time_max {
                continue;
            }
            
            // Convert time to X coordinate.
            let x_ratio = (point.unix_time as f64 - time_min as f64) / (time_max as f64 - time_min as f64);
            let x_pos = plot_rect.min.x + (x_ratio as f32 * plot_rect.width());
            
            // Convert value to Y coordinate (y_min is 0.0, y_max is 2.0).
            let y_ratio = (point.point_value - y_min) / (y_max - y_min); 
            let y_pos = plot_rect.max.y - (y_ratio * plot_rect.height());
            
            screen_points.push(egui::pos2(x_pos, y_pos));
        }

        // Add shading for active regions.
        if screen_points.len() > 1 {
            for i in 0..screen_points.len() - 1 {
                // Determine if the current point is the active level.
                let point_value = digital_trace.iter()
                    .filter(|p| p.unix_time >= time_min && p.unix_time <= time_max)
                    .nth(i)
                    .map(|p| p.point_value)
                    .unwrap_or(0.0);
                
                // If point value is non-zero (active), shade to baseline.
                if point_value > 0.0 {
                    let rect = egui::Rect::from_two_pos(
                        egui::pos2(screen_points[i].x, screen_points[i].y),
                        egui::pos2(screen_points[i + 1].x, low_y_pos)
                    );
                    painter.rect_filled(rect, 0.0, colours::ts_digital_fill_colour(dark_mode));
                }
            }
        }

        // XSIDLESTART Impulse (Index 0 in multi_traces).
        // So that XSIDLESTART pulses stand out have them at the front.
        let impulse_trace = &dataset.multi_traces[1];

        // The Y-max is 2.0 (from helpers_ts::calculate_y_range).
        let total_levels = 2.0; 

        for point in impulse_trace {
            // Skip points outside the visible time range.
            if point.unix_time < time_min || point.unix_time > time_max {
                continue;
            }

            let x_ratio = (point.unix_time as f64 - time_min as f64) / (time_max as f64 - time_min as f64);
            let x_pos = plot_rect.min.x + (x_ratio as f32 * plot_rect.width());
            
            // Calculate Y position based on actual impulse level.
            let y_ratio = point.point_value / total_levels;
            let y_pos = plot_rect.max.y - (y_ratio * plot_rect.height());

            // Only draw visible impulses.
            if point.point_value > 0.0 {
                // Draw vertical line from baseline to impulse level.
                painter.line_segment(
                    [egui::pos2(x_pos, baseline_y), egui::pos2(x_pos, y_pos)],
                    line_stroke_impulse,
                );
                
                // Draw a circle at the top of each impulse.
                painter.circle_filled(egui::pos2(x_pos, y_pos), 3.0, line_colour_impulse);
            }
        }
                
        // Draw lines connecting the points for the digital trace.
        for i in 1..screen_points.len() {
            painter.line_segment([screen_points[i-1], screen_points[i]], line_stroke_digital);
        }

        return;
    }

    // Check if dataset not empty else continue rendering other types.
    if dataset.time_series_points.is_empty() {
        return;
    }
    
    // Choose line colour based on data type and dark mode.
    let line_colour = if dataset.data_type == "Digital" {
        colours::ts_digital_colour(dark_mode)
    } else if dataset.data_type == "Analog" {
        colours::ts_analog_colour(dark_mode)
    } else if dataset.data_type == "Impulse" {
        colours::ts_impulse_colour(dark_mode)
    } else {
        // Fallback colour.
        colours::ts_digital_colour(dark_mode)
    };

    let line_stroke = egui::Stroke::new(LINE_THICKNESS, line_colour);
    
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
        // Need to avoid a panic if the plot is scrolled off the page.
        if screen_points.len() > 1 {
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
        else {
            warn!("Catching panic when scrolling trace off the plot.")
        }
    }

    // Add special handling for impulse signals after the digital shading section.
    if dataset.data_type == "Impulse" {
        // Draw impulse markers as vertical lines from baseline to the impulse level.
        let baseline_y = plot_rect.max.y;
        
        // Calculate total levels for proper scaling.
        let total_levels = if !dataset.levels.is_empty() {
            dataset.levels.len() + 1
        } else {
            4 // Fallback.
        };
        
        for point in &dataset.time_series_points {
            // Skip points outside the visible time range.
            if point.unix_time < time_min || point.unix_time > time_max {
                continue;
            }
            
            let x_ratio = (point.unix_time as f64 - time_min as f64) / (time_max as f64 - time_min as f64);
            let x_pos = plot_rect.min.x + (x_ratio as f32 * plot_rect.width());
            
            // Calculate Y position based on actual impulse level.
            let y_ratio = point.point_value / total_levels as f32;
            let y_pos = plot_rect.max.y - (y_ratio * plot_rect.height());
            
            // Choose colour based on level.
            let mut impulse_colour = egui::Color32::GRAY;

            if dataset.series_name == "IMPACT" {
                impulse_colour = match point.point_value as i32 {
                    1 => colours::ts_impact_low_colour(dark_mode),
                    2 => colours::ts_impact_warning_colour(dark_mode),
                    3 => colours::ts_impact_critical_colour(dark_mode),
                    _ => colours::ts_fallback_colour(dark_mode),
                };
            }
            else if dataset.series_name == "ZONECHANGE" || dataset.series_name == "ZONETRANSITION" {
                    impulse_colour = colours::ts_impulse_colour(dark_mode);
            }

            // Only draw visible impulses (non-zero values).
            if point.point_value > 0.0 {
                // Draw vertical line from baseline to impulse level.
                painter.line_segment(
                    [egui::pos2(x_pos, baseline_y), egui::pos2(x_pos, y_pos)],
                    egui::Stroke::new(LINE_THICKNESS * 2.0, impulse_colour),
                );
                
                // Draw a circle at the top of each impulse.
                painter.circle_filled(egui::pos2(x_pos, y_pos), 3.0, impulse_colour);
            }
        }

        // Don't draw connecting lines for impulse signals.
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
                painter.circle_filled(*point, 2.0, line_colour);
            }
        }
    }
}
