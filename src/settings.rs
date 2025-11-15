use serde::{Deserialize, Serialize};

// Settings that the user can control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub trip_font_size:         f32,
    pub event_font_size:        f32,
}

// Fumction to clamp settings to limits
// when checking if user values.
impl Settings {
    // Font sizes are limited to the range 12.0 to 20.0 .
    pub fn validate(&mut self) {
        self.trip_font_size = self.trip_font_size.clamp(12.0, 20.0);
        self.event_font_size = self.event_font_size.clamp(12.0, 20.0);
    }
}

// Default values if not entered by the user,
// or the user sets out of bounds values.
impl Default for Settings {
    fn default() -> Self {
        Settings {
            trip_font_size:     16.0,
            event_font_size:    13.0,
        }
    }
}

// Program settings, not settable by user.
#[derive(Debug, Clone)]
pub struct Details {
    pub program_name:               String,
    pub program_ver:                String,
    pub program_date:               String,
    pub program_devs:               Vec<String>,
    pub program_web:                String,

    pub min_win_width:              f32,
    pub win_width:                  f32,
    pub max_win_width:              f32,
    pub win_height:                 f32,
    pub min_win_height:             f32,
    pub max_win_height:             f32,

    pub help_win_width:             f32,
    pub min_help_win_height:        f32,
    pub help_win_height:            f32,
    pub max_help_win_height:        f32,

    pub gps_win_width:              f32,
    pub gps_win_min_width:          f32,
    pub gps_win_max_width:          f32,
    pub gps_win_height:             f32,
    pub gps_win_min_height:         f32,
    pub gps_win_max_height:         f32,

    pub time_series_win_min_width:  f32,
    pub time_series_win_width:      f32,
    pub time_series_win_max_width:  f32,
    pub time_series_win_min_height: f32,
    pub time_series_win_height:     f32,
    pub time_series_win_max_height: f32,

    pub changelog_win_width:        f32,
    pub changelog_win_height:       f32,
}
