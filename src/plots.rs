// Draw the gps data plots to a separate UI.

// use log::info;
use log::debug;

use eframe::egui;
use egui::epaint;
use geo_types::Point;
use chrono::{DateTime, NaiveDateTime, Utc, ParseError};
use walkers::{Map, MapMemory, HttpTiles};
use walkers::Plugin;

use crate::scraper::{Scraper, ScrapedData};

// PlotPoint struct.
#[derive(Debug, Clone)]
pub struct PlotPoint {
    pub _timestamp: DateTime<Utc>,
    pub _trip_num: String,
    pub lat: f64,
    pub lon: f64,
    pub speed: u32,
    pub _rssi: u32,
}

// Plotpoint struct instantiated from scraped data.
impl From<&ScrapedData> for PlotPoint {
    fn from(data: &ScrapedData) -> Self {
        Self {
            _timestamp: parse_datetime(&data.date_time).unwrap(),
            _trip_num : data.trip_num.clone(),
            lat: data.gps_locn.lat,
            lon: data.gps_locn.lon,
            speed: data.gps_speed,
            _rssi: data.gps_rssi,
        }
    }
}

// Create a plugin for GPS plotting.
pub struct GpsPlotPlugin {
    pub plot_points: Vec<PlotPoint>,
}

impl GpsPlotPlugin {
    // Draw a start pin (green flag style).
    fn draw_start_pin(&self, painter: &egui::Painter, pos: egui::Pos2) {
        let pin_height = 20.0;
        let _pin_width = 12.0;
        let flag_height = 8.0;
        let flag_width = 16.0;
        
        // Draw the pin pole (dark gray).
        painter.line_segment(
            [pos, pos + egui::Vec2::new(0.0, -pin_height)],
            egui::Stroke::new(2.0, egui::Color32::DARK_GRAY)
        );
        
        // Draw the flag (bright green).
        let flag_points = [
            pos + egui::Vec2::new(0.0, -pin_height),
            pos + egui::Vec2::new(flag_width, -pin_height + flag_height/2.0),
            pos + egui::Vec2::new(0.0, -pin_height + flag_height),
        ];
        
        painter.add(egui::Shape::convex_polygon(
            flag_points.to_vec(),
            egui::Color32::from_rgb(0, 200, 0),
            egui::Stroke::new(1.0, egui::Color32::DARK_GREEN)
        ));
        
        // Draw a small circle at the base.
        painter.circle_filled(pos, 4.0, egui::Color32::WHITE);
        painter.circle_stroke(pos, 4.0, egui::Stroke::new(2.0, egui::Color32::DARK_GREEN));
    }
    
    // Draw a finish pin (checkered flag style).
    fn draw_finish_pin(&self, painter: &egui::Painter, pos: egui::Pos2) {
        let pin_height = 20.0;
        let flag_height = 12.0;
        let flag_width = 16.0;
        let checker_size = 3.0;
        
        // Draw the pin pole (dark gray).
        painter.line_segment(
            [pos, pos + egui::Vec2::new(0.0, -pin_height)],
            egui::Stroke::new(2.0, egui::Color32::DARK_GRAY)
        );
        
        // Draw the flag background (white).
        let flag_rect = egui::Rect::from_min_size(
            pos + egui::Vec2::new(0.0, -pin_height),
            egui::Vec2::new(flag_width, flag_height)
        );
        painter.rect_filled(flag_rect, egui::CornerRadius::ZERO, egui::Color32::WHITE);
        painter.rect_stroke(flag_rect, egui::CornerRadius::ZERO, egui::Stroke::new(1.0, egui::Color32::BLACK), egui::epaint::StrokeKind::Inside);
        
        // Draw checkered pattern.
        let checkers_x = (flag_width / checker_size) as i32;
        let checkers_y = (flag_height / checker_size) as i32;

        // Draw the checks on the flag.
        for x in 0..checkers_x {
            for y in 0..checkers_y {
                if (x + y) % 2 == 1 {
                    let checker_rect = egui::Rect::from_min_size(
                        pos + egui::Vec2::new(x as f32 * checker_size, -pin_height + y as f32 * checker_size),
                        egui::Vec2::new(checker_size, checker_size)
                    );
                    painter.rect_filled(checker_rect, egui::CornerRadius::ZERO, egui::Color32::BLACK);
                }
            }
        }
        
        // Draw a small circle at the base.
        painter.circle_filled(pos, 4.0, egui::Color32::WHITE);
        painter.circle_stroke(pos, 4.0, egui::Stroke::new(2.0, egui::Color32::BLACK));
    }
}

// Instantiate plugin for walkers API.
impl Plugin for GpsPlotPlugin {
    fn run(
        self: Box<Self>,
        ui: &mut egui::Ui,
        response: &egui::Response,
        projector: &walkers::Projector,
        _map_memory: &MapMemory,
    ) {
        // Get the painter from the UI.
        let painter = ui.painter();
        
        // Draw connecting lines between GPS points.
        for window in self.plot_points.windows(2) {
            let prev_point = &window[0];
            let curr_point = &window[1];
            
            // Use walkers' projector to convert GPS to screen coordinates.
            let prev_pos = walkers::Position::from(Point::new(prev_point.lon, prev_point.lat));
            let curr_pos = walkers::Position::from(Point::new(curr_point.lon, curr_point.lat));
            
            // Project to screen coordinates using walkers' projector.
            let prev_screen = projector.project(prev_pos);
            let curr_screen = projector.project(curr_pos);
            
            // Convert Vec2 to Pos2 for proper positioning.
            let prev_screen_pos = egui::Pos2::new(prev_screen.x, prev_screen.y);
            let curr_screen_pos = egui::Pos2::new(curr_screen.x, curr_screen.y);
            
            // Only draw if both points are within the visible area.
            if response.rect.contains(prev_screen_pos) && response.rect.contains(curr_screen_pos) {
                painter.line_segment(
                    [prev_screen_pos, curr_screen_pos],
                    egui::Stroke::new(3.0, egui::Color32::from_rgba_unmultiplied(0, 120, 255, 200))
                );
            }
        }
        
        // Draw GPS points.
        for (i, point) in self.plot_points.iter().enumerate() {
            let position = walkers::Position::from(Point::new(point.lon, point.lat));
            let screen_vec = projector.project(position);
            
            // Convert Vec2 to Pos2.
            let screen_pos = egui::Pos2::new(screen_vec.x, screen_vec.y);
            
            // Only draw if the point is within the visible area.
            if response.rect.contains(screen_pos) {
                // Check if this is start or end point.
                let is_start = i == 0;
                let is_end = i == self.plot_points.len() - 1;
                
                if is_start {
                    // Draw start pin (green flag-style).
                    self.draw_start_pin(&painter, screen_pos);
                } else if is_end {
                    // Draw finish pin (checkered flag-style).
                    self.draw_finish_pin(&painter, screen_pos);
                } else {
                    // Regular GPS point - colour based on speed.
                    let color = if point.speed > 100 {
                        egui::Color32::RED
                    } else if point.speed > 80 {
                        egui::Color32::from_rgb(255, 165, 0)
                    } else if point.speed > 60 {
                        egui::Color32::BLUE
                    } else {
                        egui::Color32::GREEN
                    };
                    
                    // Draw the point with outline.
                    painter.circle_filled(screen_pos, 5.0, color);
                    painter.circle_stroke(screen_pos, 5.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
                }
            }
        }
    }
}

// Helper function for getting dates.
pub fn parse_datetime(date_str: &str) -> Result<DateTime<Utc>, ParseError> {
    let naive = NaiveDateTime::parse_from_str(date_str, "%d/%m/%Y %H:%M:%S")?;
    Ok(naive.and_utc())
}

// Function to plot GPS data using custom drawing.
pub fn plot_gps_data(ui: &mut egui::Ui, scraper: &Scraper, selected_id: &Option<String>) {

    // Get id of selected trip, or show prompt if no trip selected.
    let selected_trip = match selected_id.as_ref() {
        Some(id) if !id.is_empty() => id,
        _ => {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("Please select a trip to plot GPS points.");
            });
            return;
        }
    };

    debug!("Selected trip number: {}", selected_trip);

    // Get all the plotting points.
    // Filter out bad gps points, i.e lat and lon = 0;
    let plot_points: Vec<PlotPoint> = scraper.scrapings.iter()
        .filter(|scraped| scraped.trip_num == *selected_trip)
        .filter(|scraped| scraped.gps_locn.lat != 0.0 && scraped.gps_locn.lon != 0.0)
        .map(PlotPoint::from)
        .collect();

    debug!("Number of plot points in trip: {}", plot_points.len());

    if plot_points.is_empty() {
        ui.label("No valid GPS points found for this trip.");
        return;
    }

    // Create a custom plot area.
    let plot_height = ui.available_height() - 95.0;
    let plot_width = ui.available_width();
    
    let (rect, _response) = ui.allocate_exact_size(
        egui::Vec2::new(plot_width, plot_height),
        egui::Sense::click_and_drag()
    );

    // Draw the plot background.
    ui.painter().rect_filled(
        rect,
        epaint::CornerRadius::same(5),
        ui.visuals().extreme_bg_color,
    );

    // Draw border. Use StrokeKind::Inside
    ui.painter().rect_stroke(
        rect,
        epaint::CornerRadius::same(5),
        egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
        egui::epaint::StrokeKind::Inside,
    );

    // Calculate bounds of gps points.
    // This is to fill the plot area initially.
    let mut min_lat = f64::MAX;
    let mut max_lat = f64::MIN;
    let mut min_lon = f64::MAX;
    let mut max_lon = f64::MIN;

    for point in &plot_points {
        min_lat = min_lat.min(point.lat);
        max_lat = max_lat.max(point.lat);
        min_lon = min_lon.min(point.lon);
        max_lon = max_lon.max(point.lon);
    }

    // Calculate the centre point of plot.
    let lat_centre = min_lat + (max_lat - min_lat);
    let lon_centre = min_lon + (max_lon - min_lon);

    // Add some padding.
    let lat_range = max_lat - min_lat;
    let lon_range = max_lon - min_lon;
    let padding = 0.1;
    
    min_lat -= lat_range * padding;
    max_lat += lat_range * padding;
    min_lon -= lon_range * padding;
    max_lon += lon_range * padding;

    // Draw the GPS points.
    // Leave some margin around the sides.
    let plot_rect = rect.shrink(20.0);
    
    // Iterate and plot points.
    for (i, point) in plot_points.iter().enumerate() {
        // Convert GPS coordinates to screen coordinates.
        let x = plot_rect.left() as f64 + ((point.lon - min_lon) / (max_lon - min_lon)) * plot_rect.width() as f64;
        let y = plot_rect.bottom() as f64 - ((point.lat - min_lat) / (max_lat - min_lat)) * plot_rect.height() as f64;
        
        let screen_pos = egui::Pos2::new(x as f32, y as f32);
        
        // Colour based on speed.
        // Set arbitrary speed zones.
        // Potentially include speed zones in settings.
        let color = if point.speed > 100 {
            egui::Color32::RED
        } else if point.speed > 80 {
            egui::Color32::BLUE
        } else if point.speed > 60 {
            egui::Color32::ORANGE
        } else {
            egui::Color32::GREEN
        };
        
        // Draw the point.
        ui.painter().circle_filled(screen_pos, 3.0, color);
        
        // Draw lines connecting consecutive points.
        if i > 0 {
            let prev_point = &plot_points[i - 1];
            let prev_x = plot_rect.left() as f64 + ((prev_point.lon - min_lon) / (max_lon - min_lon)) * plot_rect.width() as f64;
            let prev_y = plot_rect.bottom() as f64 - ((prev_point.lat - min_lat) / (max_lat - min_lat)) * plot_rect.height() as f64;
            let prev_screen_pos = egui::Pos2::new(prev_x as f32, prev_y as f32);
            
            ui.painter().line_segment( 
                [prev_screen_pos, screen_pos],
                egui::Stroke::new(1.0, egui::Color32::from_gray(128))
            );
        }
    }

    // Show legend.
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Speed legend:");
        ui.colored_label(egui::Color32::GREEN, "● ≤60 km/h");
        ui.colored_label(egui::Color32::ORANGE, "● 60-80 km/h");
        ui.colored_label(egui::Color32::BLUE, "● 80-100 km/h");
        ui.colored_label(egui::Color32::RED, "● >100 km/h");
    });

    // Show some statistics.
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("GPS points:");
        ui.strong(format!("{}", plot_points.len()));
    });
    
    if let (Some(first), Some(last)) = (plot_points.first(), plot_points.last()) {
        ui.horizontal(|ui| {
            ui.label("Trip time:");
            ui.strong(format!("{} to {}", first._timestamp.format("%H:%M:%S"), last._timestamp.format("%H:%M:%S")));
        });
    }

    // Show centrepoint of plot.
    ui.horizontal(|ui| {
        ui.label("Map centre:");
        ui.strong(format!("{:.6}, {:.6}", lat_centre, lon_centre));
    });
}

// Plot gps points with tiles.
pub fn plot_gps_data_with_tiles(
    ui: &mut egui::Ui, 
    scraper: &Scraper, 
    selected_id: &Option<String>, 
    map_memory: &mut MapMemory,
    tiles: &mut HttpTiles,
    last_trip_id: &mut Option<String>,
) {

    // Get id of selected trip, or show prompt if no trip selected.
    let selected_trip = match selected_id.as_ref() {
        Some(id) if !id.is_empty() => id,
        _ => {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("Please select a trip to plot GPS points.");
            });
            return;
        }
    };

    // Get all the plotting points.
    let plot_points: Vec<PlotPoint> = scraper.scrapings.iter()
        .filter(|scraped| scraped.trip_num == *selected_trip)
        .filter(|scraped| scraped.gps_locn.lat != 0.0 && scraped.gps_locn.lon != 0.0)
        .map(PlotPoint::from)
        .collect();

    if plot_points.is_empty() {
        ui.label("No valid GPS points found for this trip.");
        return;
    }

    // Only centre the map if the trip has changed.
    // This allows user panning and zooming to work properly.
    let trip_changed = last_trip_id.as_ref() != Some(selected_trip);
    if trip_changed {

        // Calculate bounds of all GPS points.
        let mut min_lat = f64::MAX;
        let mut max_lat = f64::MIN;
        let mut min_lon = f64::MAX;
        let mut max_lon = f64::MIN;

        for point in &plot_points {
            min_lat = min_lat.min(point.lat);
            max_lat = max_lat.max(point.lat);
            min_lon = min_lon.min(point.lon);
            max_lon = max_lon.max(point.lon);
        }

        // Add some padding (10% on each side).
        let lat_range = max_lat - min_lat;
        let lon_range = max_lon - min_lon;
        let padding = 0.1;
        
        min_lat -= lat_range * padding;
        max_lat += lat_range * padding;
        min_lon -= lon_range * padding;
        max_lon += lon_range * padding;

        // Calculate centre.
        let centre_lat = (min_lat + max_lat) / 2.0;
        let centre_lon = (min_lon + max_lon) / 2.0;
        let centre_position = walkers::Position::from(Point::new(centre_lon, centre_lat));

        // Calculate appropriate zoom level to fit all points.
        // This is a rough approximation - adjust the multiplier if necessary.
        let lat_span = max_lat - min_lat;
        let lon_span = max_lon - min_lon;
        let max_span = lat_span.max(lon_span);
        
        // Approximate zoom level calculation (fine-tune this).        
        let zoom = if max_span > 0.1 {
            10.0
        } else if max_span > 0.01 {
            12.0
        } else if max_span > 0.001 {
            14.0
        } else {
            16.0
        };

        debug!("Max span: {:?}, Zoom level: {:?}", max_span, zoom);

        // Set the centre and zoom for the plot.
        map_memory.center_at(centre_position);
        let _ = map_memory.set_zoom(zoom);
        *last_trip_id = Some(selected_trip.clone());
    }

    // Calculate the centre for display purposes.
    let centre_lat = plot_points.iter().map(|p| p.lat).sum::<f64>() / plot_points.len() as f64;
    let centre_lon = plot_points.iter().map(|p| p.lon).sum::<f64>() / plot_points.len() as f64;
    let centre_position = walkers::Position::from(Point::new(centre_lon, centre_lat));

    // Create the GPS plotting plugin.
    let gps_plugin = GpsPlotPlugin { 
        plot_points: plot_points.clone() 
    };

    // Create the map widget with the plugin.
    let map_size = egui::Vec2::new(ui.available_width().min(800.0), ui.available_height() - 95.0);
    let map_response = ui.add_sized(
        map_size,
        Map::new(Some(tiles), map_memory, centre_position)
            .with_plugin(gps_plugin)
    );
    
    // Handle map interactions.
    if map_response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
    }
    
    // Show legend.
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Speed legend:");
        ui.colored_label(egui::Color32::GREEN, "● ≤60 km/h");
        ui.colored_label(egui::Color32::BLUE, "● 60-80 km/h");
        ui.colored_label(egui::Color32::ORANGE, "● 80-100 km/h");
        ui.colored_label(egui::Color32::RED, "● >100 km/h");
    });
    
    // Show some statistics.
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("GPS points:");
        ui.strong(format!("{}", plot_points.len()));
    });
    
    if let (Some(first), Some(last)) = (plot_points.first(), plot_points.last()) {
        ui.horizontal(|ui| {
            ui.label("Trip time:");
            ui.strong(format!("{} to {}", first._timestamp.format("%H:%M:%S"), last._timestamp.format("%H:%M:%S")));
        });
    }
    
    // Show centre coordinates.
    ui.horizontal(|ui| {
        ui.label("Map centre:");
        ui.strong(format!("{:.6}, {:.6}", centre_lat, centre_lon));
    });
    
    // Force repaint to ensure tiles keep loading.
    ui.ctx().request_repaint();
}
