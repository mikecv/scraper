// Draw the time series data plots to a separate UI.
// All plots share the same time x axis.
// Pan and zoom on any one plot pan and zooms the others.
// Not that zooming is only in the time axis.

use eframe::egui;
use egui::{Rect, Stroke, Sense};

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
    pub units: String,
    pub time_series_points: Vec<SinglePoint>,
}

// Plot state to maintain synchronized pan/zoom across multiple plots.
#[derive(Debug, Clone)]
pub struct PlotState {
    pub x_range: Option<(f64, f64)>,
    pub _auto_bounds: bool,
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
            _auto_bounds: true,
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

// SinglePoint struct instantiated from scraped data.
impl From<&ScrapedData> for SinglePoint {
    fn from(data: &ScrapedData) -> Self {
        Self {
            unix_time: data.unix_time,
            point_value: data.gps_speed as f32,
        }
    }
}

// Helper function to convert unix timestamp to seconds for plotting.
fn unix_to_plot_time(unix_time: u64, reference_time: u64) -> f32 {
    (unix_time - reference_time) as f32
}

// Function to create multiple time series datasets.
fn create_time_series_datasets(scraper: &Scraper, selected_trip: &str) -> Vec<TimeSeriesData> {
    let mut datasets = Vec::new();

    // Get all points for the selected trip
    let trip_data: Vec<&ScrapedData> = scraper.scrapings.iter()
        .filter(|scraped| scraped.trip_num == *selected_trip)
        .collect();

    if trip_data.is_empty() {
        return datasets;
    }

    // Speed time series
    let speed_points: Vec<SinglePoint> = trip_data.iter()
        .map(|data| SinglePoint {
            unix_time: data.unix_time,
            point_value: data.gps_speed as f32,
        })
        .collect();

    if !speed_points.is_empty() {
        datasets.push(TimeSeriesData {
            series_name: "GPS Speed".to_string(),
            units: "m/s".to_string(),
            time_series_points: speed_points,
        });
    }

    datasets
}

// Convert screen position to time value.
fn screen_to_time(screen_x: f32, plot_rect: Rect, time_min: f32, time_max: f32) -> f32 {
    let relative_x = (screen_x - plot_rect.min.x) / plot_rect.width();
    time_min + relative_x * (time_max - time_min)
}

// Custom simple plotting function with mouse interaction.
fn draw_time_series_plot(
    ui: &mut egui::Ui,
    dataset: &TimeSeriesData,
    reference_time: u64,
    plot_rect: Rect,
    plot_state: &mut PlotState,
    dark_mode: &bool,
) {
    if dataset.time_series_points.is_empty() {
        return;
    }

    // Convert points to plot coordinates.
    let points: Vec<(f32, f32)> = dataset.time_series_points.iter()
        .map(|p| (unix_to_plot_time(p.unix_time, reference_time), p.point_value))
        .collect();

    // Find data bounds.
    let time_min = points.iter().map(|(t, _)| *t).fold(f32::INFINITY, f32::min);
    let time_max = points.iter().map(|(t, _)| *t).fold(f32::NEG_INFINITY, f32::max);
    let value_min = points.iter().map(|(_, v)| *v).fold(f32::INFINITY, f32::min);
    let value_max = points.iter().map(|(_, v)| *v).fold(f32::NEG_INFINITY, f32::max);

    // Apply pan and zoom.
    let time_range = time_max - time_min;
    let value_range = if value_max != value_min { value_max - value_min } else { 1.0 };
    
    let zoomed_time_range = time_range / plot_state.zoom_factor;
    let centre_time = time_min + time_range * 0.5 + plot_state.pan_offset * time_range;
    
    let display_time_min = centre_time - zoomed_time_range * 0.5;
    let display_time_max = centre_time + zoomed_time_range * 0.5;

    // Create interactive area for the plot.
    let response = ui.allocate_rect(plot_rect, Sense::click_and_drag());

    // Handle mouse interactions.
    if response.hovered() {
        // Mouse wheel for zooming.
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll_delta != 0.0 {
            let zoom_speed = 0.001;
            let old_zoom = plot_state.zoom_factor;
            plot_state.zoom_factor *= 1.0 + scroll_delta * zoom_speed;
            plot_state.zoom_factor = plot_state.zoom_factor.clamp(0.1, 100.0);
            
            // Adjust pan to zoom towards mouse position.
            if let Some(mouse_pos) = response.hover_pos() {
                let mouse_time_ratio = (mouse_pos.x - plot_rect.min.x) / plot_rect.width();
                let zoom_ratio = plot_state.zoom_factor / old_zoom;
                plot_state.pan_offset += (mouse_time_ratio - 0.5) * (1.0 - 1.0 / zoom_ratio) / plot_state.zoom_factor;
            }
        }
    }

    // Handle dragging for panning or selection.
    if response.drag_started() {
        if ui.input(|i| i.modifiers.shift) {
            // Shift + drag for selection.
            plot_state.is_selecting = true;
            plot_state.selection_start = response.interact_pointer_pos();
            plot_state.selection_end = response.interact_pointer_pos();
        } else {
            // Regular drag for panning.
            plot_state.is_dragging = true;
            plot_state.drag_start = response.interact_pointer_pos();
        }
    }

    if response.dragged() {
        if plot_state.is_selecting {
            // Update selection end.
            plot_state.selection_end = response.interact_pointer_pos();
        } else if plot_state.is_dragging {
            // Handle panning.
            if let (Some(current_pos), Some(start_pos)) = (response.interact_pointer_pos(), plot_state.drag_start) {
                let delta_x = current_pos.x - start_pos.x;
                let pan_speed = 1.0 / plot_state.zoom_factor;
                plot_state.pan_offset -= (delta_x / plot_rect.width()) * pan_speed;
                plot_state.drag_start = Some(current_pos);
            }
        }
    }

    if response.drag_stopped() {
        if plot_state.is_selecting {
            // Handle zoom to selection.
            if let (Some(start), Some(end)) = (plot_state.selection_start, plot_state.selection_end) {
                let left_x = start.x.min(end.x);
                let right_x = start.x.max(end.x);
                
                // Minimum selection width, tweak as necessary.
                if (right_x - left_x) > 10.0 {
                    let start_time = screen_to_time(left_x, plot_rect, display_time_min, display_time_max);
                    let end_time = screen_to_time(right_x, plot_rect, display_time_min, display_time_max);
                    
                    // Calculate new pan and zoom to fit selection.
                    let selection_width = end_time - start_time;
                    let selection_centre = (start_time + end_time) * 0.5;
                    
                    if selection_width > 0.0 {
                        let new_zoom = zoomed_time_range / selection_width;
                        plot_state.zoom_factor *= new_zoom;
                        plot_state.zoom_factor = plot_state.zoom_factor.clamp(0.1, 100.0);
                        
                        // Adjust pan to centre the selection.
                        let data_centre = time_min + time_range * 0.5;
                        plot_state.pan_offset = (selection_centre - data_centre) / time_range;
                    }
                }
            }
            plot_state.is_selecting = false;
            plot_state.selection_start = None;
            plot_state.selection_end = None;
        }
        plot_state.is_dragging = false;
        plot_state.drag_start = None;
    }

    // Clamp pan offset to reasonable bounds.
    plot_state.pan_offset = plot_state.pan_offset.clamp(-2.0, 2.0);

    // Transform points to screen coordinates.
    let screen_points: Vec<egui::Pos2> = points.iter()
        .filter_map(|(t, v)| {
            if *t >= display_time_min && *t <= display_time_max {
                let x = plot_rect.min.x + 
                    ((t - display_time_min) / (display_time_max - display_time_min)) * plot_rect.width();
                let y = plot_rect.max.y - 
                    ((v - value_min) / value_range) * plot_rect.height();
                Some(egui::Pos2::new(x, y))
            } else {
                None
            }
        })
        .collect();

    // Draw the plot background.
    ui.painter().rect_filled(plot_rect, 0.0, colours::ts_back_gnd_colour(*dark_mode));
    ui.painter().rect_stroke(plot_rect, 0.0, Stroke::new(1.0, colours::ts_back_gnd_colour(*dark_mode)), egui::epaint::StrokeKind::Inside);

    // Draw grid lines.
    for i in 1..5 {
        let x = plot_rect.min.x + (i as f32 / 5.0) * plot_rect.width();
        ui.painter().line_segment(
            [egui::Pos2::new(x, plot_rect.min.y), egui::Pos2::new(x, plot_rect.max.y)],
            Stroke::new(0.5, colours::ts_grid_colour(*dark_mode))
        );
        
        let y = plot_rect.min.y + (i as f32 / 5.0) * plot_rect.height();
        ui.painter().line_segment(
            [egui::Pos2::new(plot_rect.min.x, y), egui::Pos2::new(plot_rect.max.x, y)],
            Stroke::new(0.5, colours::ts_grid_colour(*dark_mode))
        );
    }

    // Draw the line connecting points.
    if screen_points.len() > 1 {
        for i in 0..screen_points.len() - 1 {
            ui.painter().line_segment(
                [screen_points[i], screen_points[i + 1]],
                Stroke::new(1.0, colours::ts_line_colour(*dark_mode))
            );
        }
    }

    // Draw points.
    for point in &screen_points {
        ui.painter().circle_filled(*point, 1.0, colours::ts_line_colour(*dark_mode));
    }

    // Draw selection rectangle if selecting.
    if plot_state.is_selecting {
        if let (Some(start), Some(end)) = (plot_state.selection_start, plot_state.selection_end) {
            let selection_rect = Rect::from_two_pos(start, end);
            ui.painter().rect_stroke(selection_rect, 0.0, Stroke::new(2.0, colours::ts_zoom_outline_colour(*dark_mode)), egui::epaint::StrokeKind::Inside);
            ui.painter().rect_filled(selection_rect, 0.0, colours::ts_zoom_fill_colour(*dark_mode));
        }
    }

    // Draw axes labels.
    let time_label = format!("Time: {:.1}s - {:.1}s", display_time_min, display_time_max);
    let value_label = format!("{:.2} - {:.2} {}", value_min, value_max, dataset.units);
    
    // Bottom time label.
    ui.painter().text(
        egui::Pos2::new(plot_rect.center().x, plot_rect.max.y + 15.0),
        egui::Align2::CENTER_TOP,
        time_label,
        egui::FontId::default(),
        colours::ts_labels_colour(*dark_mode)
    );

    // Left value label.
    ui.painter().text(
        egui::Pos2::new(plot_rect.min.x - 5.0, plot_rect.center().y),
        egui::Align2::RIGHT_CENTER,
        value_label,
        egui::FontId::default(),
        colours::ts_labels_colour(*dark_mode)
    );
}

// Function to plot time series data using custom drawing with mouse interaction.
pub fn plot_time_series_data(
    ui: &mut egui::Ui, 
    scraper: &Scraper, 
    selected_id: &Option<String>,
    plot_state: &mut PlotState,
    dark_mode: &bool,
) {
    // Get id of selected trip, or show prompt if no trip selected.
    let selected_trip = match selected_id.as_ref() {
        Some(id) if !id.is_empty() => id,
        _ => {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("Please select a trip to plot time series data.");
            });
            return;
        }
    };

    // Create time series datasets.
    let datasets = create_time_series_datasets(scraper, selected_trip);

    // Check if no datasets to plot.
    if datasets.is_empty() {
        ui.label("No valid time series points found for this trip.");
        return;
    }

    // Find the reference time (earliest timestamp) for relative time plotting.
    let reference_time = datasets
        .iter()
        .flat_map(|dataset| &dataset.time_series_points)
        .map(|point| point.unix_time)
        .min()
        .unwrap_or(0);

    // Display summary info.
    ui.horizontal(|ui| {
        ui.label("Time series datasets:");
        ui.strong(format!("{}", datasets.len()));
        ui.separator();
        let total_points: usize = datasets.iter().map(|d| d.time_series_points.len()).sum();
        ui.label("Total points:");
        ui.strong(format!("{}", total_points));
    });

    // Add controls and usage instructions.
    ui.horizontal(|ui| {
        if ui.button("Reset Zoom").clicked() {
            plot_state.zoom_factor = 1.0;
            plot_state.pan_offset = 0.0;
            plot_state.x_range = None;
        }
        
        ui.separator();
        ui.label("Mouse wheel: zoom | Drag: pan | Shift+drag: zoom to selection");
    });

    ui.separator();

    // Plot each dataset in a separate plot area.
    let available_height = ui.available_height() - 100.0;
    let plot_height = (available_height / datasets.len() as f32).min(200.0).max(100.0);

    for dataset in datasets.iter() {
        if dataset.time_series_points.is_empty() {
            continue;
        }

        ui.vertical(|ui| {
            // Title for this plot.
            ui.horizontal(|ui| {
                ui.strong(&dataset.series_name);
                ui.label(format!("({})", &dataset.units));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Zoom: {:.1}x", plot_state.zoom_factor));
                });
            });

            // Reserve space for the plot.
            let plot_rect = Rect::from_min_size(
                ui.cursor().min,
                egui::Vec2::new(ui.available_width() - 100.0, plot_height)
            );

            // Draw the plot.
            draw_time_series_plot(ui, dataset, reference_time, plot_rect, plot_state, &dark_mode);

            // Advance the cursor.
            ui.allocate_space(egui::Vec2::new(plot_rect.width(), plot_rect.height() + 30.0));

            ui.add_space(10.0);
        });
    }
}
