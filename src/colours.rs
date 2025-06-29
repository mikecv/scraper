// Functions to provide custom colours
// for light and dark themes.

// use log::info;

use eframe::egui::Color32;

// Colours for event labels in scraped logs.
pub fn event_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(58, 235, 14)
    } else {
        Color32::from_rgb(43, 125, 22)
    }
}

// Colours for top level trip labels in scraped logs.
pub fn trip_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(255, 255, 255)
    } else {
        Color32::from_rgb(0, 0, 0)
    }
}
