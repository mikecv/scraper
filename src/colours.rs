// Functions to provide custom colours
// for dark and light themes.

use eframe::egui::Color32;

// Colours for event labels in scraped logs.
pub fn event_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(58, 235, 14)
    } else {
        Color32::from_rgb(56, 218, 4)
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
        Color32::from_rgb(186, 92, 18)
    }
}

// Colours for event key / value pairs (value).
pub fn value_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(112, 123, 124)
    } else {
        Color32::from_rgb(52, 73, 94)
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
        Color32::from_rgb(178, 186, 187)
    } else {
        Color32::from_rgb(74, 35, 90)
    }
}

// Colours for gps key / value pairs (value).
pub fn gps_value_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(112, 123, 124)
    } else {
        Color32::from_rgb(52, 73, 94)
    }
}

// Colours for screen and dialog borders.
pub fn border_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(100, 100, 100)
    } else {
        Color32::from_rgb(128, 128, 128)
    }
}

// // Colours for time series plot notices.
pub fn ts_notices_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(255, 255, 0)
    } else {
        Color32::from_rgb(0, 102, 5)
    }
}

// Colours for time series digital plot line.
pub fn ts_digital_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(255, 153, 0)
    } else {
        Color32::from_rgb(255, 173, 51)
    }
}

// Shading colours for time series digital plot line.
pub fn ts_digital_fill_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgba_unmultiplied(255, 153, 0, 50)
    } else {
        Color32::from_rgba_unmultiplied(255, 173, 51, 50)
    }
}

// Colours for time series analog plot line.
pub fn ts_analog_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(0, 255, 0)
    } else {
        Color32::from_rgb(102, 255, 102)
    }
}

// Impulse signals colour.
pub fn ts_impulse_colour(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(255, 165, 0)
    } else {
        egui::Color32::from_rgb(255, 140, 0)
    }
}

// Impulse signals error colour.
pub fn ts_impulse_error_colour(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(178, 102, 255)
    } else {
        egui::Color32::from_rgb(76, 0, 153)
    }
}