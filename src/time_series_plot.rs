use log::info;
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
}

// Plot state to maintain synchronized pan/zoom across multiple plots.
#[derive(Debug, Clone)]
pub struct PlotState {
    pub x_range: Option<(f64, f64)>,
    pub auto_bounds: bool,
    pub zoom_factor: f32,
    pub pan_offset: f32,
    pub pan_zoom_enabled: bool,
}

impl Default for PlotState {
    fn default() -> Self {
        Self {
            x_range: None,
            auto_bounds: true,
            zoom_factor: 1.0,
            pan_offset: 0.0,
            pan_zoom_enabled: false,
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
                // Reset button (only show when pan/zoom is enabled and has been used).
                if plot_state.pan_zoom_enabled && (!plot_state.auto_bounds || plot_state.zoom_factor != 1.0 || plot_state.pan_offset != 0.0) {
                    if ui.button("Reset View").clicked() {
                        plot_state.x_range = None;
                        plot_state.auto_bounds = true;
                        plot_state.pan_offset = 0.0;
                        plot_state.zoom_factor = 1.0;
                    }
                }
                
                // Toggle button for pan/zoom functionality.
                let button_text = if plot_state.pan_zoom_enabled {
                    "🔍 Pan/Zoom ON"
                } else {
                    "🔍 Pan/Zoom OFF"
                };

                // Set botton colour and button text colour.
                let (button_color, text_color) = if plot_state.pan_zoom_enabled {
                    (colours::ts_enabled_button_colour(*dark_mode), colours::ts_enabled_button_text_colour(*dark_mode))
                } else {
                    (colours::ts_disabled_button_colour(*dark_mode), colours::ts_disabled_button_text_colour(*dark_mode))
                };

                let button = egui::Button::new(egui::RichText::new(button_text).color(text_color)).fill(button_color);
                if ui.add(button).clicked() {
                    plot_state.pan_zoom_enabled = !plot_state.pan_zoom_enabled;
                    // Reset view when disabling pan/zoom.
                    if !plot_state.pan_zoom_enabled {
                        plot_state.x_range = None;
                        plot_state.auto_bounds = true;
                        plot_state.pan_offset = 0.0;
                        plot_state.zoom_factor = 1.0;
                    }
                }
            });
        });
        
        // Show trip selection status and pan/zoom hint.
        ui.horizontal(|ui| {
            match selected_trip {
                Some(trip_id) if !trip_id.is_empty() => {
                    ui.label(format!("Current trip ID: {}", trip_id));
                }
                _ => info!("No trip selected."),
            }
            
            // Show helpful hint when pan/zoom is enabled.
            if plot_state.pan_zoom_enabled {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("💡 Drag to pan, scroll wheel to zoom")
                        .color(colours::ts_notices_colour(*dark_mode))
                        .size(12.0));
                });
            }
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
                    // If trip selected, and not empty, get datasets to plot.
                    let datasets = dataset_ts::create_time_series_datasets(scraper, trip_id);

                    // Calculate overall time range for all datasets.
                    let (time_min, time_max) = helpers_ts::calculate_time_range(&datasets);

                    for dataset in datasets {
                        // Here's the space allocation for a single plot.
                        let plot_size = egui::vec2(ui.available_width(), PLOT_HEIGHT);
                        let (plot_response, painter) = ui.allocate_painter(plot_size, egui::Sense::click_and_drag());

                        // Only handle panning and zooming if enabled.
                        if plot_state.pan_zoom_enabled {
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
    let (panned_time_min, panned_time_max) = if plot_state.pan_zoom_enabled {
        let time_range = time_max - time_min;
        let zoomed_time_range = (time_range as f32 / plot_state.zoom_factor) as u64;
        let pan_pixels_to_time = zoomed_time_range as f32 / plot_rect.width();
        let pan_time_offset = (plot_state.pan_offset * pan_pixels_to_time) as i64;

        let center_time = (time_min + time_max) / 2;
        let half_zoomed_range = zoomed_time_range / 2;

        let panned_time_min = ((center_time as i64) - (half_zoomed_range as i64) - pan_time_offset).max(0) as u64;
        let panned_time_max = ((center_time as i64) + (half_zoomed_range as i64) - pan_time_offset).max(0) as u64;
        
        (panned_time_min, panned_time_max)
    } else {
        // Use full time range when pan/zoom is disabled.
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
                egui::pos2(x_pos, plot_rect.max.y + 20.0),
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
        // For impulse signals, show levels dynamically based on dataset.levels
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

    // Draw grid lines
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
            
            // Add top position
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

    let line_stroke = egui::Stroke::new(LINE_THIVKNESS, line_colour);
    
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
            else if dataset.series_name == "ZONECHANGE" {
                if point.point_value as i32 == 1 {
                    impulse_colour = colours::ts_impulse_error_colour(dark_mode);
                }
                else {
                    impulse_colour = colours::ts_impulse_colour(dark_mode);
                }
            }

            // Only draw visible impulses (non-zero values).
            if point.point_value > 0.0 {
                // Draw vertical line from baseline to impulse level.
                painter.line_segment(
                    [egui::pos2(x_pos, baseline_y), egui::pos2(x_pos, y_pos)],
                    egui::Stroke::new(LINE_THIVKNESS * 2.0, impulse_colour),
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
