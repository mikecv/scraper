// Module to handle initialization of application settings and details.

use lazy_static::lazy_static;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::Mutex;

use crate::settings::{Settings, Details};

// Create a global variable for application settings.
// This will be available in other files.
lazy_static! {
    pub static ref SETTINGS: Mutex<Settings> = {
        Mutex::new(load_settings())
    };
}

// Create a global variable for program details.
// Not included in settings as not user settable.
// This will be available in other files.
lazy_static! {
    pub static ref DETAILS: Mutex<Details> = {
        Mutex::new(create_details())
    };
}

/// Load settings from file or create default settings.
fn load_settings() -> Settings {
    match File::open("settings.yml") {
        Ok(mut file) => {
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => {
                    match serde_yaml::from_str::<Settings>(&contents) {
                        Ok(mut settings) => {
                            settings.validate();
                            settings
                        }
                        // Settings invalid values.
                        Err(_) => Settings::default(),
                    }
                }
                // Failed to read from settings file.
                Err(_) => Settings::default(),
            }
        }
        // Settings file not found - create it with defaults.
        Err(_) => {
            let default_settings = Settings::default();
            
            // Try to create the settings file.
            if let Ok(yaml) = serde_yaml::to_string(&default_settings) {
                if let Ok(mut file) = File::create("settings.yml") {
                    let content = format!(
                        "# User settings for Scraper application.\n\
                         # Font sizes must be between 12.0 and 20.0\n\n\
                         {}", 
                        yaml
                    );
                    let _ = file.write_all(content.as_bytes());
                }
            }
            default_settings
        }
    }
}

/// Create application details (not user settable).
fn create_details() -> Details {
    Details {
        program_name:           "Scraper".to_string(),
        program_ver:            "0.5.0".to_string(),
        program_date:           "2025".to_string(),
        program_devs:           vec!["mdc".to_string()],
        program_web:            "galacticwingcommander".to_string(),

        min_win_width:          400.0,
        win_width:              450.0,
        max_win_width:          500.0,

        min_win_height:         400.0,
        win_height:             500.0,
        max_win_height:         650.0,

        help_win_width:         500.0,
        
        min_help_win_height:    500.0,
        help_win_height:        550.0,
        max_help_win_height:    600.0,

        gps_win_width:          600.0,
        gps_win_height:         600.0,

        time_series_win_width:  500.0,
        time_series_win_height: 650.0,

        changelog_win_width:    300.0,
        changelog_win_height:   350.0,
    }
}
