use serde::{Deserialize};

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub win_width: f32,
    pub win_height: f32,
}

#[derive(Debug, Clone)]
pub struct Details {
    pub program_name: String,
    pub program_ver: String,
    pub program_date: String,
    pub program_devs: Vec<String>,
    pub program_web: String,
}
