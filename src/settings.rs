use serde::{Deserialize};

#[derive(Debug, Deserialize, Clone)]
// User settings to be defines as necessary.
pub struct Settings {
}

#[derive(Debug, Clone)]
// Program settings, not settable by user.
pub struct Details {
    pub program_name:       String,
    pub program_ver:        String,
    pub program_date:       String,
    pub program_devs:       Vec<String>,
    pub program_web:        String,
    pub scroll_win_width:   f32,
    pub win_width:          f32,
    pub win_height:         f32,
}
