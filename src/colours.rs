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

// Colours for event key / value pairs (key).
pub fn key_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(255, 214, 51)
    } else {
        Color32::from_rgb(230, 184, 0)
    }
}

// Colours for event key / value pairs (value).
pub fn value_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(153, 255, 206)
    } else {
        Color32::from_rgb(0, 204, 105)
    }
}

// Colours for out of trip values (not outstanding).
pub fn subtle_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(77, 77, 77)
    } else {
        Color32::from_rgb(204, 204, 204)
    }
}

// Colours for gps key / value pairs (key).
pub fn gps_key_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(77, 77, 255)
    } else {
        Color32::from_rgb(128, 128, 255)
    }
}

// Colours for gps key / value pairs (value).
pub fn gps_value_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(179, 179, 0)
    } else {
        Color32::from_rgb(230, 230, 0)
    }
}

