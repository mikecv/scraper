// Functions to provide custom colours
// for dark and light themes.

use eframe::egui::Color32;

// Colours for plot area background.
pub fn plot_area_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(30, 30, 30)
    } else {
        Color32::from_rgb(255, 255, 255)
    }
}

// Colours for plot background.
pub fn plot_bkgnd_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(40, 40, 40)
    } else {
        Color32::from_rgb(250, 250, 250)
    }
}

// Colours for plot axis.
pub fn plot_axis_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(169, 169, 169)
    } else {
        Color32::from_rgb(211, 211, 211)
    }
}

// Shading colours for time series digital plot line.
pub fn ts_grid_lines_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgba_unmultiplied(200, 200, 200, 150)
    } else {
        Color32::from_rgba_unmultiplied(100, 100, 100, 100)
    }
}

// Colours for plot text.
pub fn plot_text_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(255, 255, 255)
    } else {
        Color32::from_rgb(0, 0, 0)
    }
}

// Colours for event labels in scraped logs.
pub fn event_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(58, 235, 14)
    } else {
        Color32::from_rgb(56, 218, 4)
    }
}

// Colours for (Unsupported) event labels in scraped logs.
pub fn us_event_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(204, 153, 255)
    } else {
        Color32::from_rgb(153, 0, 204)
    }
}

// Colours for Out of Trip event labels in scraped logs.
pub fn oot_event_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(255, 204, 102)
    } else {
        Color32::from_rgb(255, 153, 0)
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

// Colours for gps key / value pairs (key).
pub fn gps_key_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(131, 198, 229)
    } else {
        Color32::from_rgb(79, 97, 106)
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

//Colours for time series plot notices.
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

// Impulse signals colour (for XSIDLESTART and other similar impulses).
pub fn ts_xsidle_impulse_colour(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(0, 0, 255)
    } else {
        egui::Color32::from_rgb(128, 128, 255)
    }
}

// Impact level critical colour.
pub fn ts_impact_critical_colour(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(255, 0, 0)
    } else {
        egui::Color32::from_rgb(161, 52, 71)
    }
}

// Impact level warning colour.
pub fn ts_impact_warning_colour(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(242, 165, 93)
    } else {
        egui::Color32::from_rgb(240, 118, 4)
    }
}

// Impact level warning colour.
pub fn ts_impact_low_colour(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(238, 245, 78)
    } else {
        egui::Color32::from_rgb(236, 245, 7)
    }
}

// Time series fallback colour.
pub fn ts_fallback_colour(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(178, 179, 174)
    } else {
        egui::Color32::from_rgb(61, 61, 59)
    }
}

// Pan and zoom (enabled) button colour.
pub fn ts_enabled_button_colour(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(200, 255, 200)
    } else {
        egui::Color32::from_rgb(100, 150, 100)
    }
}

// Pan and zoom (enabled) button text colour.
pub fn ts_enabled_button_text_colour(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(0, 0, 0)
    } else {
        egui::Color32::from_rgb(255, 255, 255)
    }
}

// Pan and zoom (disabled) button colour.
pub fn ts_disabled_button_colour(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(80, 80, 80)
    } else {
        egui::Color32::from_rgb(230, 230, 230)
    }
}

// Pan and zoom (disabled) button text colour.
pub fn ts_disabled_button_text_colour(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(200, 200, 200)
    } else {
        egui::Color32::from_rgb(60, 60, 60)
    }
}

// Cursor colour.
pub fn ts_cursor_colour(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(255, 179, 255)
    } else {
        egui::Color32::from_rgb(153, 0, 153)
    }
}

// Time cursor label colour.
pub fn ts_cursor_label_colour(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgba_premultiplied(40, 40, 40, 230)
    } else {
        Color32::from_rgba_premultiplied(245, 245, 245, 230)
    }
}
