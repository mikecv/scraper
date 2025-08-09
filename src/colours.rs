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

// Colours for time series plot lines.
pub fn ts_line_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(58, 235, 14)
    } else {
        Color32::from_rgb(0, 51, 204)
    }
}

// Colours for time series plot backgroud.
pub fn ts_back_gnd_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(10, 10, 10)
    } else {
        Color32::from_rgb(250, 250, 250)
    }
}

// Colours for time series plot grid lines.
pub fn ts_grid_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(250, 250, 250)
    } else {
        Color32::from_rgb(10, 10, 10)
    }
}

// Colours for time series plot labels.
pub fn ts_labels_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(204, 255, 204)
    } else {
        Color32::from_rgb(0, 0, 75)
    }
}

// Colours for zoom box outline.
pub fn ts_zoom_outline_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(204, 15, 0)
    } else {
        Color32::from_rgb(51, 0, 10)
    }
}

// Colours for zoom box fill.
pub fn ts_zoom_fill_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgba_unmultiplied(204, 15, 0, 30)
    } else {
        Color32::from_rgba_unmultiplied(204, 15, 0, 30)
    }
}
