// plots.rs

use log::info;

use eframe::egui;
use egui::epaint;
use geo_types::Point;
use chrono::{DateTime, NaiveDateTime, Utc, ParseError};
use walkers::{Map, MapMemory, HttpTiles};
// use walkers::{Map, MapMemory, HttpTiles, sources::OpenStreetMap};
// Removed unused import: use std::sync::Arc;

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
            lon: data.gps_locn.long,
            speed: data.gps_speed,
            _rssi: data.gps_rssi,
        }
    }
}

// Helper function for getting dates.
pub fn parse_datetime(date_str: &str) -> Result<DateTime<Utc>, ParseError> {
    let naive = NaiveDateTime::parse_from_str(date_str, "%d/%m/%Y %H:%M:%S")?;
    Ok(naive.and_utc())
}

// // Struct to hold tiles instance persistently
// pub struct MapTiles {
//     pub tiles: HttpTiles,
// }

// impl MapTiles {
//     pub fn new(ctx: egui::Context) -> Self {
//         Self {
//             tiles: HttpTiles::new(OpenStreetMap, ctx),
//         }
//     }
// }

// Function to plot GPS data using custom drawing.
pub fn plot_gps_data(ui: &mut egui::Ui, scraper: &Scraper, selected_id: &Option<String>) {
    info!("Initiating GPS plotting.");

    // Check to see if there is a current trip selected.
    if selected_id.as_deref() == Some("") {
        ui.label("No trip selected.");
        return;
    }

    // Get id of selected trip, or return if no trip selected.
    let selected_trip = match selected_id.as_ref() {
        Some(id) => id,
        None => {
            ui.label("No trip selected.");
            return;
        }
    };

    info!("Selected trip number: {}", selected_trip);

    // Get all the plotting points.
    // Filter out bad gps points, i.e lat and long = 0;
    // Lat / Lon equal to 0,0 legitimate but ignored as out in the ocean.
    let plot_points: Vec<PlotPoint> = scraper.scrapings.iter()
        .filter(|scraped| scraped.trip_num == *selected_trip)
        .filter(|scraped| scraped.gps_locn.lat != 0.0 && scraped.gps_locn.long != 0.0)
        .map(PlotPoint::from)
        .collect();

    info!("Number of plot points in trip: {}", plot_points.len());

    if plot_points.is_empty() {
        ui.label("No valid GPS points found for this trip.");
        return;
    }

    // Create a custom plot area.
    let plot_height = 400.0;
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

    // Draw border - Fixed: Use StrokeKind::Inside
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
        // Convert GPS coordinates to screen coordinates
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

    // Add axis labels.
    ui.painter().text(
        egui::Pos2::new(rect.left() + 5.0, rect.bottom() - 10.0),
        egui::Align2::LEFT_BOTTOM,
        format!("Lon: {:.4}", min_lon),
        egui::FontId::default(),
        ui.visuals().text_color(),
    );
    
    ui.painter().text(
        egui::Pos2::new(rect.right() - 5.0, rect.bottom() - 10.0),
        egui::Align2::RIGHT_BOTTOM,
        format!("Lon: {:.4}", max_lon),
        egui::FontId::default(),
        ui.visuals().text_color(),
    );
    
    ui.painter().text(
        egui::Pos2::new(rect.left() + 5.0, rect.top() + 10.0),
        egui::Align2::LEFT_TOP,
        format!("Lat: {:.4}", max_lat),
        egui::FontId::default(),
        ui.visuals().text_color(),
    );
    
    ui.painter().text(
        egui::Pos2::new(rect.left() + 5.0, rect.bottom() - 30.0),
        egui::Align2::LEFT_BOTTOM,
        format!("Lat: {:.4}", min_lat),
        egui::FontId::default(),
        ui.visuals().text_color(),
    );

    // Show legend.
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Speed legend:");
        ui.colored_label(egui::Color32::GREEN, "● 60 km/h");
        ui.colored_label(egui::Color32::ORANGE, "● 60-80 km/h");
        ui.colored_label(egui::Color32::BLUE, "● 80-100 km/h");
        ui.colored_label(egui::Color32::RED, "● 100 km/h");
    });

    // Show some statistics.
    ui.separator();
    ui.label(format!("Total GPS points: {}", plot_points.len()));
    
    if let (Some(first), Some(last)) = (plot_points.first(), plot_points.last()) {
        ui.label(format!("Trip duration: {} to {}", 
            first._timestamp.format("%H:%M:%S"),
            last._timestamp.format("%H:%M:%S")));
    }

    // Show coordinate range.
    ui.label(format!("Latitude range: {:.4} to {:.4}", min_lat + lat_range * padding, max_lat - lat_range * padding));
    ui.label(format!("Longitude range: {:.4} to {:.4}", min_lon + lon_range * padding, max_lon - lon_range * padding));
}









// Replace the plot_gps_data_with_osm function with this corrected version
// Alternative: Much simpler approach that should work reliably
// Replace the problematic section with this simpler coordinate transformation

pub fn plot_gps_data_with_osm(
    ui: &mut egui::Ui, 
    scraper: &Scraper, 
    selected_id: &Option<String>, 
    map_memory: &mut MapMemory,
    tiles: &mut HttpTiles, 
) {
    info!("Initiating GPS plotting with OSM tiles (simple approach).");

    // [Keep the same validation logic as before]
    if selected_id.as_deref() == Some("") {
        ui.label("No trip selected.");
        return;
    }

    let selected_trip = match selected_id.as_ref() {
        Some(id) => id,
        None => {
            ui.label("No trip selected.");
            return;
        }
    };

    let plot_points: Vec<PlotPoint> = scraper.scrapings.iter()
        .filter(|scraped| scraped.trip_num == *selected_trip)
        .filter(|scraped| scraped.gps_locn.lat != 0.0 && scraped.gps_locn.long != 0.0)
        .map(PlotPoint::from)
        .collect();

    if plot_points.is_empty() {
        ui.label("No valid GPS points found for this trip.");
        return;
    }

    // Calculate bounds for normalization
    let min_lat = plot_points.iter().map(|p| p.lat).fold(f64::INFINITY, f64::min);
    let max_lat = plot_points.iter().map(|p| p.lat).fold(f64::NEG_INFINITY, f64::max);
    let min_lon = plot_points.iter().map(|p| p.lon).fold(f64::INFINITY, f64::min);
    let max_lon = plot_points.iter().map(|p| p.lon).fold(f64::NEG_INFINITY, f64::max);

    // Calculate center for map
    let centre_lat = (min_lat + max_lat) / 2.0;
    let centre_lon = (min_lon + max_lon) / 2.0;
    let centre_position = walkers::Position::from(Point::new(centre_lon, centre_lat));

    // Center the map
    map_memory.center_at(centre_position);

    // Create the map widget
    let map_size = egui::Vec2::new(ui.available_width().min(800.0), 400.0);
    let map_response = ui.add_sized(map_size, Map::new(Some(tiles), map_memory, centre_position));

    // Simple coordinate transformation - normalize GPS coordinates to screen space
    let map_rect = map_response.rect.shrink(20.0); // Leave some margin
    
    let lat_range = max_lat - min_lat;
    let lon_range = max_lon - min_lon;
    
    // Add padding to avoid edge clipping
    let padding = 0.1;
    let padded_lat_range = lat_range * (1.0 + padding);
    let padded_lon_range = lon_range * (1.0 + padding);
    
    let gps_to_screen = |lat: f64, lon: f64| -> egui::Pos2 {
        // Normalize coordinates to 0-1 range
        let norm_x = (lon - centre_lon + padded_lon_range / 2.0) / padded_lon_range;
        let norm_y = (lat - centre_lat + padded_lat_range / 2.0) / padded_lat_range;
        
        // Convert to screen coordinates
        let screen_x = map_rect.left() + norm_x as f32 * map_rect.width();
        let screen_y = map_rect.bottom() - norm_y as f32 * map_rect.height(); // Flip Y axis
        
        egui::Pos2::new(screen_x, screen_y)
    };

    let painter = ui.painter_at(map_response.rect);

    // Draw connecting lines
    for window in plot_points.windows(2) {
        let prev_screen = gps_to_screen(window[0].lat, window[0].lon);
        let curr_screen = gps_to_screen(window[1].lat, window[1].lon);
        
        painter.line_segment(
            [prev_screen, curr_screen],
            egui::Stroke::new(3.0, egui::Color32::from_rgba_unmultiplied(0, 120, 255, 200))
        );
    }
    
    // Draw GPS points
    for point in &plot_points {
        let screen_pos = gps_to_screen(point.lat, point.lon);
        
        let color = if point.speed > 100 {
            egui::Color32::RED
        } else if point.speed > 80 {
            egui::Color32::from_rgb(255, 165, 0)
        } else if point.speed > 60 {
            egui::Color32::BLUE
        } else {
            egui::Color32::GREEN
        };
        
        painter.circle_filled(screen_pos, 5.0, color);
        painter.circle_stroke(screen_pos, 5.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
    }
    
    // [Keep the same legend and statistics as before]
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Speed legend:");
        ui.colored_label(egui::Color32::GREEN, "● ≤60 km/h");
        ui.colored_label(egui::Color32::BLUE, "● 60-80 km/h");
        ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "● 80-100 km/h");
        ui.colored_label(egui::Color32::RED, "● >100 km/h");
    });
    
    ui.separator();
    ui.label(format!("Total GPS points: {}", plot_points.len()));
    
    if let (Some(first), Some(last)) = (plot_points.first(), plot_points.last()) {
        ui.label(format!("Trip duration: {} to {}", 
            first._timestamp.format("%H:%M:%S"),
            last._timestamp.format("%H:%M:%S")));
    }
    
    ui.label(format!("Map centre: {:.6}, {:.6}", centre_lat, centre_lon));
    ui.ctx().request_repaint();
}
