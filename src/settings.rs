use serde::{Deserialize, Serialize};

// Settings that the user can control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // Add settings fields here as needed.
    // For now, keeping it empty but with proper serde derives.
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            // Add defaults for settings above.
        }
    }
}

#[derive(Debug, Clone)]
// Program settings, not settable by user.
pub struct Details {
    pub program_name:       String,
    pub program_ver:        String,
    pub program_date:       String,
    pub program_devs:       Vec<String>,
    pub program_web:        String,
    pub win_width:          f32,
    pub win_height:         f32,
    pub help_win_width:     f32,
    pub help_win_height:    f32,
}
