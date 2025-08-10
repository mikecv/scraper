// Draw the time series data plots to a separate UI.
// All plots share the same time x axis.
// Pan and zoom on any one plot pan and zooms the others.
// Note that zooming is only in the time axis.

use eframe::egui;
use egui::{Rect, Stroke, Sense};
use chrono::{DateTime, Local, TimeZone};

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
    pub _data_type: String,
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

// Function to create multiple time series datasets.
// The idea is that datasets for all event types that lend themselves to being plotted
// that are in the particular trip get plotted.
// Included in every plotting is the battery voltage and gps speed as these aren't included
// in actual events.
fn create_time_series_datasets(scraper: &Scraper, selected_trip: &str) -> Vec<TimeSeriesData> {
    let mut datasets = Vec::new();

    // Get all points for the selected trip.
    let trip_data: Vec<&ScrapedData> = scraper.scrapings.iter()
        .filter(|scraped| scraped.trip_num == *selected_trip)
        .collect();

    // Abort if no data in the trip.
    if trip_data.is_empty() {
        return datasets;
    }

    // TEST DATA
    // Weight time series.
    let weight_points: Vec<SinglePoint> = trip_data.iter()
        .map(|data| SinglePoint {
            unix_time: data.unix_time,
            point_value: data.gps_speed as f32 * 0.35,
        })
        .collect();

    if !weight_points.is_empty() {
        datasets.push(TimeSeriesData {
            _data_type: "Analog".to_string(),
            series_name: "Weight".to_string(),
            units: "kg".to_string(),
            time_series_points: weight_points,
        });
    }

    // TEST DATA
    // Battery voltage time series.
    let battery_points: Vec<SinglePoint> = trip_data.iter()
    .filter_map(|data| {
        // Look for "Battery voltage" in the ev_detail vector.
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
            _data_type: "Analog".to_string(),
            series_name: "Battery".to_string(),
            units: "V".to_string(),
            time_series_points: battery_points,
        });
    }

    // TEST DATA
    // Speed time series.
    let speed_points: Vec<SinglePoint> = trip_data.iter()
        .map(|data| SinglePoint {
            unix_time: data.unix_time,
            point_value: data.gps_speed as f32,
        })
        .collect();

    if !speed_points.is_empty() {
        datasets.push(TimeSeriesData {
            _data_type: "Analog".to_string(),
            series_name: "Speed".to_string(),
            units: "kph".to_string(),
            time_series_points: speed_points,
        });
    }

    // Set of all data series to plot.
    datasets
}

// Helper function to generate the time grid labels.
// Aim is to convert Unix time to dates, further, to show dates and time as applicable.
fn generate_grid_time_labels(
    reference_time: u64,
    display_time_min: f32,
    display_time_max: f32,
    num_grid_lines: usize,
) -> Vec<(f32, String, Option<String>)> {
    let mut labels = Vec::new();
    let time_span = display_time_max - display_time_min;
    
    for i in 0..=num_grid_lines {
        let relative_time = display_time_min + (i as f32 / num_grid_lines as f32) * time_span;
        let unix_time = reference_time + relative_time as u64;
        let dt: DateTime<Local> = Local.timestamp_opt(unix_time as i64, 0).unwrap();
        
        // Determine time and date labels based on span.
        let (time_label, date_label) = match time_span {
            // Less than 24 hours - show time including seconds
            span if span < 86400.0 => (dt.format("%H:%M:%S").to_string(), None),
            // Less than 7 days - show time and date separately.
            span if span < 604800.0 => (
                dt.format("%H:%M").to_string(),
                Some(dt.format("%m/%d").to_string())
            ),
            // Longer periods - show date and time.
            _ => (
                dt.format("%H:%M").to_string(),
                Some(dt.format("%m/%d/%Y").to_string())
            ),
        };
        
        labels.push((relative_time, time_label, date_label));
    }
    
    // Remove duplicate consecutive dates.
    // Don't show dates more than once or the time axis.
    if labels.len() > 1 {
        for i in 1..labels.len() {
            if let (Some(prev_date), Some(curr_date)) = (&labels[i-1].2, &labels[i].2) {
                if prev_date == curr_date {
                    labels[i].2 = None;
                }
            }
        }
    }
    
    labels
}

// Generate smart y graduations.
// Base on scale and grid lines with sensible increments.
// Pass min and max values and number of graduations to use.
fn generate_smart_y_graduations(value_min: f32, value_max: f32, units: &str, num_divisions: usize) -> Vec<f32> {
    let range = value_max - value_min;
    if range <= 0.0 {
        return vec![value_min];
    }
    
    // Determine smart step size based on units and range.
    // Include known grid increments where it makes sense.
    let base_step = match units.to_lowercase().as_str() {
        "kph" | "mph" => {
            // Speed: use steps of 5, 10, 15, 20, 25, 50, etc.
            let candidates = vec![5.0, 10.0, 15.0, 20.0, 25.0, 50.0, 100.0];
            candidates.into_iter().find(|&step| range / step <= num_divisions as f32).unwrap_or(range / num_divisions as f32)
        },
        "V" => {
            // Voltage: use steps of 0.5, 1.0, 1.5, 2.0, 5.0, etc.
            let candidates = vec![0.5, 1.0, 1.5, 2.0, 2.5, 5.0, 10.0];
            candidates.into_iter().find(|&step| range / step <= num_divisions as f32).unwrap_or(range / num_divisions as f32)
        },
        _ => {
            // Generic: use nice round numbers.
            let raw_step = range / num_divisions as f32;
            let magnitude = 10.0_f32.powf(raw_step.log10().floor());
            let normalized = raw_step / magnitude;
            let nice_normalized = if normalized <= 1.0 { 1.0 }
                else if normalized <= 2.0 { 2.0 }
                else if normalized <= 5.0 { 5.0 }
                else { 10.0 };
            nice_normalized * magnitude
        }
    };
    
    // Generate graduations.
    let start = (value_min / base_step).ceil() * base_step;
    let mut graduations = Vec::new();
    let mut current = start;
    
    while current <= value_max && graduations.len() < 10 {
        if current >= value_min {
            graduations.push(current);
        }
        current += base_step;
    }
    
    graduations
}

// Helper function to convert unix timestamp to seconds for plotting.
fn unix_to_plot_time(unix_time: u64, reference_time: u64) -> f32 {
    (unix_time - reference_time) as f32
}

// Convert screen position to time value.
fn screen_to_time(screen_x: f32, plot_rect: Rect, time_min: f32, time_max: f32) -> f32 {
    let relative_x = (screen_x - plot_rect.min.x) / plot_rect.width();
    time_min + relative_x * (time_max - time_min)
}

// Plotting function with mouse interaction for pan and zoom.
// Plots all the time series data.
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

    // Handle dragging for panning.
    // Dragging only in horizontal direction.
    if response.drag_started() {
        plot_state.is_dragging = true;
        plot_state.drag_start = response.interact_pointer_pos();
    }

    // If detect plot area dragged then pan all plots at the same time.
    if response.dragged() {
        if plot_state.is_dragging {
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

    // Generate smart y-axis graduations.
    let y_graduations = generate_smart_y_graduations(value_min, value_max, &dataset.units, 5);

    // Draw grid lines at the graduations - ensure they stay within plot bounds
    for &grad_value in &y_graduations {
        if grad_value >= value_min && grad_value <= value_max {
            let y = plot_rect.max.y - ((grad_value - value_min) / value_range) * plot_rect.height();
            
            // Ensure the grid line is within the plot bounds
            if y >= plot_rect.min.y && y <= plot_rect.max.y {
                ui.painter().line_segment(
                    [egui::Pos2::new(plot_rect.min.x, y), egui::Pos2::new(plot_rect.max.x, y)],
                    Stroke::new(0.5, colours::ts_grid_colour(*dark_mode))
                );
                
                // Draw y-axis label at this graduation.
                ui.painter().text(
                    egui::Pos2::new(plot_rect.min.x - 5.0, y),
                    egui::Align2::RIGHT_CENTER,
                    format!("{:.1}", grad_value),
                    egui::FontId::proportional(10.0),
                    colours::ts_labels_colour(*dark_mode)
                );
            }
        }
    }

    // Vertical time grid lines are the same all the time.
    // Only the graduation labels change.
    for i in 1..5 {
        let x = plot_rect.min.x + (i as f32 / 5.0) * plot_rect.width();
        ui.painter().line_segment(
            [egui::Pos2::new(x, plot_rect.min.y), egui::Pos2::new(x, plot_rect.max.y)],
            Stroke::new(0.5, colours::ts_grid_colour(*dark_mode))
        );
    }   
    
    // Draw the line connecting the data points.
    if screen_points.len() > 1 {
        for i in 0..screen_points.len() - 1 {
            ui.painter().line_segment(
                [screen_points[i], screen_points[i + 1]],
                Stroke::new(1.0, colours::ts_line_colour(*dark_mode))
            );
        }
    }

    // Draw actual points at the data points.
    for point in &screen_points {
        ui.painter().circle_filled(*point, 1.0, colours::ts_line_colour(*dark_mode));
    }

    // Generate grid-aligned time labels.
    // Number of grid lines as above.
    let num_grid_lines = 5;
    let time_labels = generate_grid_time_labels(reference_time, display_time_min, display_time_max, num_grid_lines);

    // Draw time labels at grid positions.
    for (relative_time, time_label, date_label) in time_labels {
        let x_pos = plot_rect.min.x + ((relative_time - display_time_min) / (display_time_max - display_time_min)) * plot_rect.width();
        
        // Draw time label below the plot
        ui.painter().text(
            egui::Pos2::new(x_pos, plot_rect.max.y + 15.0),
            egui::Align2::CENTER_TOP,
            time_label,
            egui::FontId::proportional(10.0),
            colours::ts_labels_colour(*dark_mode)
        );
        
        // Draw date label if present (on a second line).
        if let Some(date) = date_label {
            ui.painter().text(
                egui::Pos2::new(x_pos, plot_rect.max.y + 35.0),
                egui::Align2::CENTER_TOP,
                date,
                egui::FontId::default(),
                colours::ts_labels_colour(*dark_mode)
            );
        }
    }    
}

// Function to plot time series data using custom drawing with mouse interaction.
// This is called from the ui.rs file where the UI panel is defined and created.
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

        // Button to reset the pan and zoom position to starting position.
        if ui.button("Reset Pan & Zoom").clicked() {
            plot_state.zoom_factor = 1.0;
            plot_state.pan_offset = 0.0;
            plot_state.x_range = None;
        }
    });

    ui.separator();

    // Plot each dataset in a separate plot area.
    let plot_height = 50.0;
    let left_margin = 100.0;
    let right_margin = 40.0;
    let _top_margin = 10.0;
    let bottom_margin = 60.0;
    let plot_spacing = 30.0;

    for (index, dataset) in datasets.iter().enumerate() {
        if dataset.time_series_points.is_empty() {
            continue;
        }

        ui.vertical(|ui| {
            // Add a small space before the first plot title.
            // This is only for the first plot, after that the plot_spacing is enough.
            if index > 0 {
                ui.add_space(plot_spacing / 2.0);
            }

            // The title for this plot. This takes up vertical space naturally.
            ui.horizontal(|ui| {
                ui.strong(&dataset.series_name);
                ui.label(format!("({})", &dataset.units));
            });

            // Calculate the width for the plot
            let available_plot_width = ui.available_width() - left_margin - right_margin;

            // Reserve the space for the plot itself. This response rect
            // is positioned correctly below the title.
            let (response, _painter) = ui.allocate_exact_size(
                egui::Vec2::new(available_plot_width, plot_height),
                egui::Sense::hover()
            );

            // Define the actual plot rectangle within the allocated space
            // It's the response rect, translated to the right by the left margin.
            let plot_rect = response.translate(egui::Vec2::new(left_margin, 0.0));

            // Draw the plot using the correctly positioned rectangle.
            draw_time_series_plot(ui, dataset, reference_time, plot_rect, plot_state, dark_mode);

            // Add a space for the time labels at the bottom of the plot.
            ui.add_space(bottom_margin);

            // Add spacing to the next plot.
            if index < datasets.len() - 1 {
                ui.add_space(plot_spacing / 2.0);
            }
        });
    }
}
