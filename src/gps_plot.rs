// Draw the gps data plots to a separate UI.

use eframe::egui;
use egui::epaint;
use geo_types::Point;
use chrono::{DateTime, NaiveDateTime, Utc, ParseError};
use walkers::{Map, MapMemory, HttpTiles};
use walkers::Plugin;
use walkers::sources::{TileSource, Attribution};
use reqwest::Client;

use crate::scraper::{Scraper, ScrapedData};
use crate::app::PlotViewState;

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

#[derive(Debug, Clone)]
pub struct MapState {
    pub center_lat: f64,
    pub center_lon: f64,
    pub zoom: f64,
}

impl MapState {
    pub fn new(lat: f64, lon: f64, zoom: f64) -> Self {
        Self {
            center_lat: lat,
            center_lon: lon,
            zoom,
        }
    }
}

// PlotPoint struct instantiated from scraped data.
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

// Custom satellite tile source.
#[derive(Clone, Debug)]
pub struct SatelliteTiles;

impl TileSource for SatelliteTiles {
    fn tile_url(&self, tile_id: walkers::TileId) -> String {
        // Using ESRI World Imagery (free satellite tiles)'
        format!(
            "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{}/{}/{}",
            tile_id.zoom, tile_id.y, tile_id.x
        )
    }

    fn attribution(&self) -> Attribution {
        Attribution {
            text: "Tiles © Esri — Source: Esri, i-cubed, USDA, USGS, AEX, GeoEye, Getmapping, Aerogrid, IGN, IGP, UPR-EGP, and the GIS User Community".into(),
            url: "https://www.esri.com/".into(),
            logo_dark: None,
            logo_light: None,
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

// Modified function with pan and zoom support
pub fn plot_gps_data(
    ui: &mut egui::Ui, 
    scraper: &Scraper, 
    selected_id: &Option<String>,
    view_state: &mut PlotViewState,
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

    // Calculate bounds of gps points.
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

    // Add some padding.
    let lat_range = max_lat - min_lat;
    let lon_range = max_lon - min_lon;
    let padding = 0.1;
    
    let padded_min_lat = min_lat - lat_range * padding;
    let padded_max_lat = max_lat + lat_range * padding;
    let padded_min_lon = min_lon - lon_range * padding;
    let padded_max_lon = max_lon + lon_range * padding;

    // Reset view if trip changed.
    let trip_changed = last_trip_id.as_ref() != Some(selected_trip);
    if trip_changed {
        let center_lat = (padded_min_lat + padded_max_lat) / 2.0;
        let center_lon = (padded_min_lon + padded_max_lon) / 2.0;
        view_state.reset(center_lat, center_lon);
        *last_trip_id = Some(selected_trip.clone());
    }

    // Create a custom plot area with drag sensing.
    let plot_height = ui.available_height() - 95.0;
    let plot_width = ui.available_width();
    
    let (rect, response) = ui.allocate_exact_size(
        egui::Vec2::new(plot_width, plot_height),
        egui::Sense::click_and_drag()
    );

    // Handle dragging.
    if response.dragged() {
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            if let Some(drag_start) = view_state.drag_start {
                view_state.drag_offset = pointer_pos - drag_start;
            } else {
                view_state.drag_start = Some(pointer_pos);
            }
        }
    } else if response.drag_stopped() {
        // Apply the drag offset to the centre position.
        if view_state.drag_offset != egui::Vec2::ZERO {
            let plot_rect = rect.shrink(20.0);
            
            // Convert drag pixels to lat/lon offset.
            let lat_span = (padded_max_lat - padded_min_lat) / view_state.zoom * -1.0;
            let lon_span = (padded_max_lon - padded_min_lon) / view_state.zoom * -1.0;
            
            let lat_offset = -(view_state.drag_offset.y as f64 / plot_rect.height() as f64) * lat_span;
            let lon_offset = (view_state.drag_offset.x as f64 / plot_rect.width() as f64) * lon_span;
            
            view_state.center_lat += lat_offset;
            view_state.center_lon += lon_offset;
            view_state.drag_offset = egui::Vec2::ZERO;
        }
        view_state.drag_start = None;
    }

    // Handle zooming with scroll wheel.
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta != 0.0 {
            let zoom_factor = 1.0 + scroll_delta * 0.001;
            view_state.zoom = (view_state.zoom as f64 * zoom_factor as f64).clamp(0.5, 20.0);
        }
    }

    // Draw the plot background.
    ui.painter().rect_filled(
        rect,
        epaint::CornerRadius::same(5),
        ui.visuals().extreme_bg_color,
    );

    // Draw border.
    ui.painter().rect_stroke(
        rect,
        epaint::CornerRadius::same(5),
        egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
        egui::epaint::StrokeKind::Inside,
    );

    // Calculate visible bounds based on centre and zoom.
    let lat_span = (padded_max_lat - padded_min_lat) / view_state.zoom;
    let lon_span = (padded_max_lon - padded_min_lon) / view_state.zoom;
    
    let visible_min_lat = view_state.center_lat - lat_span / 2.0;
    let visible_max_lat = view_state.center_lat + lat_span / 2.0;
    let visible_min_lon = view_state.center_lon - lon_span / 2.0;
    let visible_max_lon = view_state.center_lon + lon_span / 2.0;

    // Draw the GPS points with offset from dragging.
    let plot_rect = rect.shrink(20.0);
    
    // Iterate and plot points.
    for (i, point) in plot_points.iter().enumerate() {
        // Convert GPS coordinates to screen coordinates.
        let x = plot_rect.left() as f64 + 
            ((point.lon - visible_min_lon) / (visible_max_lon - visible_min_lon)) * plot_rect.width() as f64;
        let y = plot_rect.bottom() as f64 - 
            ((point.lat - visible_min_lat) / (visible_max_lat - visible_min_lat)) * plot_rect.height() as f64;
        
        let mut screen_pos = egui::Pos2::new(x as f32, y as f32);
        
        // Apply drag offset if currently dragging.
        if response.dragged() {
            screen_pos += view_state.drag_offset;
        }
        
        // Only draw if within bounds.
        if !plot_rect.contains(screen_pos) {
            continue;
        }
        
        // Colour based on speed.
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
            let prev_x = plot_rect.left() as f64 + 
                ((prev_point.lon - visible_min_lon) / (visible_max_lon - visible_min_lon)) * plot_rect.width() as f64;
            let prev_y = plot_rect.bottom() as f64 - 
                ((prev_point.lat - visible_min_lat) / (visible_max_lat - visible_min_lat)) * plot_rect.height() as f64;
            let mut prev_screen_pos = egui::Pos2::new(prev_x as f32, prev_y as f32);
            
            if response.dragged() {
                prev_screen_pos += view_state.drag_offset;
            }
            
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
    
    // the statistics of gps points and zoom level.
    ui.horizontal(|ui| {
        ui.label("GPS points:");
        ui.strong(format!("{}", plot_points.len()));
        ui.separator();
        ui.label("Zoom:");
        ui.strong(format!("{:.1}x", view_state.zoom));
    });
    
    if let (Some(first), Some(last)) = (plot_points.first(), plot_points.last()) {
        ui.horizontal(|ui| {
            let trip_dur = last._timestamp - first._timestamp;
            
            // Extract hours, minutes, seconds from the duration.
            let total_seconds = trip_dur.num_seconds();
            let hours = total_seconds / 3600;
            let minutes = (total_seconds % 3600) / 60;
            let seconds = total_seconds % 60;

            // Show start and end trip times, and also duration.
            ui.label("Trip time:");
            ui.strong(format!(
                "{} to {}  ({:02}:{:02}:{:02})", 
                first._timestamp.format("%H:%M:%S"), 
                last._timestamp.format("%H:%M:%S"),
                hours, 
                minutes, 
                seconds
            ));
        });
    }

    // Show centre coordinates.
    ui.horizontal(|ui| {
        ui.label("Map centre:");
        ui.strong(format!("{:.6}, {:.6}", view_state.center_lat, view_state.center_lon));
    });
}

// Replace the plot_gps_data_with_tiles function with this updated version:
pub fn plot_gps_data_with_tiles(
    ui: &mut egui::Ui, 
    scraper: &Scraper, 
    selected_id: &Option<String>, 
    map_memory: &mut MapMemory,
    tiles: &mut HttpTiles,
    last_trip_id: &mut Option<String>,
    map_state: &mut Option<MapState>, // Add this parameter
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

    // Store the previous map state to detect changes.
    let prev_zoom = map_state.as_ref().map(|s| s.zoom);

    // Only centre the map if the trip has changed.
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
        let center_lat = (min_lat + max_lat) / 2.0;
        let center_lon = (min_lon + max_lon) / 2.0;
        let center_position = walkers::Position::from(Point::new(center_lon, center_lat));

        // Calculate appropriate zoom level to fit all points.
        let lat_span = max_lat - min_lat;
        let lon_span = max_lon - min_lon;
        let max_span = lat_span.max(lon_span);
        
        // Approximate zoom level calculation (fine-tune this).        
        let zoom = if max_span > 0.1 {
            11.0
        } else if max_span > 0.01 {
            13.0
        } else if max_span > 0.001 {
            15.0
        } else {
            17.0
        };

        // Set the centre and zoom for the plot.
        map_memory.center_at(center_position);
        let _ = map_memory.set_zoom(zoom);
        
        // Update our tracked state.
        *map_state = Some(MapState::new(center_lat, center_lon, zoom));
        *last_trip_id = Some(selected_trip.clone());
    }

    // Get current state or use default.
    let current_state = map_state.as_ref().map(|s| s.clone()).unwrap_or_else(|| {
        let center_lat = plot_points.iter().map(|p| p.lat).sum::<f64>() / plot_points.len() as f64;
        let center_lon = plot_points.iter().map(|p| p.lon).sum::<f64>() / plot_points.len() as f64;
        MapState::new(center_lat, center_lon, 14.0)
    });

    let center_position = walkers::Position::from(Point::new(current_state.center_lon, current_state.center_lat));

    // Create the GPS plotting plugin.
    let gps_plugin = GpsPlotPlugin { 
        plot_points: plot_points.clone() 
    };

    // Create the map widget with the plugin.
    let map_size = egui::Vec2::new(ui.available_width().min(800.0), ui.available_height() - 95.0);
    let map_response = ui.add_sized(
        map_size,
        Map::new(Some(tiles), map_memory, center_position)
            .with_plugin(gps_plugin)
    );
    
    // Handle map interactions.
    if map_response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
    }
    
    // Get the current zoom from map_memory.
    let current_zoom = map_memory.zoom();
    
    // Detect if the map was panned or zoomed by checking response.
    let map_moved = map_response.dragged() || 
                    prev_zoom.map(|z| (z - current_zoom).abs() > 0.01).unwrap_or(false);
    
    // If the map moved, we need to approximate the new center.
    // Since we can't directly read the center, we'll estimate based on drag delta.
    let (display_lat, display_lon) = if map_moved {
        let drag_delta = map_response.drag_delta();
        if drag_delta.length() > 0.1 {
            // Calculate approximate lat/lon change based on drag.
            // This is an approximation - adjust the scale factor as needed.
            let scale_factor = 0.00001 * (20.0 / current_zoom);
            let new_lat = current_state.center_lat - (drag_delta.y as f64 * scale_factor);
            let new_lon = current_state.center_lon + (drag_delta.x as f64 * scale_factor);
            
            // Update our tracked state.
            *map_state = Some(MapState::new(new_lat, new_lon, current_zoom));
            
            (new_lat, new_lon)
        } else {
            // Update zoom only.
            *map_state = Some(MapState::new(current_state.center_lat, current_state.center_lon, current_zoom));
            (current_state.center_lat, current_state.center_lon)
        }
    } else {
        (current_state.center_lat, current_state.center_lon)
    };
    
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
        ui.separator();
        ui.label("Zoom:");
        ui.strong(format!("{:.1}", current_zoom));
    });
    
    if let (Some(first), Some(last)) = (plot_points.first(), plot_points.last()) {
        ui.horizontal(|ui| {
            let trip_dur = last._timestamp - first._timestamp;
            
            // Extract hours, minutes, seconds from the duration
            let total_seconds = trip_dur.num_seconds();
            let hours = total_seconds / 3600;
            let minutes = (total_seconds % 3600) / 60;
            let seconds = total_seconds % 60;
            
            // Show start and end trip times, and also duration.
            ui.label("Trip time:");
            ui.strong(format!(
                "{} to {}  ({:02}:{:02}:{:02})", 
                first._timestamp.format("%H:%M:%S"), 
                last._timestamp.format("%H:%M:%S"),
                hours, 
                minutes, 
                seconds
            ));
        });
    }

    // Show centre coordinates (now updating with map state).
    ui.horizontal(|ui| {
        ui.label("Map centre:");
        ui.strong(format!("{:.6}, {:.6}", display_lat, display_lon));
    });
    
    // Force repaint to ensure tiles keep loading and stats update.
    ui.ctx().request_repaint();
}

// Create a reqwest::Client configured with the Cargo.toml features.
// Ensures that the application uses the system's root certificates for TLS validation.
pub fn create_http_client() -> Client {
    // Client::builder().build() ensures that the client is built using the features.
    match Client::builder().build() {
        Ok(client) => {
            log::info!("Successfully created reqwest::Client with rustls-tls-native-certs configuration.");
            client
        },
        Err(e) => {
            // Log the error and fall back to a default client.
            log::error!("Failed to build reqwest::Client with custom TLS features: {}. Falling back to a default client.", e);
            Client::new()
        }
    }
}
