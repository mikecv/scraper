// Application to perform custom scraping of log files,
// and present results in tabular and graphical format.

// Release build for Windows without launching a console window.
#![windows_subsystem = "windows"]

use log4rs;
use log::info;

use eframe::{egui};
use lazy_static::lazy_static;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::{Mutex};

use app::MyApp;
use crate::settings::Settings;
use crate::settings::Details;

mod settings;
mod app;
mod scraper;
mod ui;
mod help_content;
mod changelog_content;
mod log_display;
mod colours;
mod gps_plot;
mod time_series_plot;
mod helpers_ts;
mod dataset_ts;

// Create a global variable for application settings.
// This will be available in other files.
lazy_static! {
    static ref SETTINGS: Mutex<Settings> = {
        let settings = match File::open("settings.yml") {
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
                            "# User settings for Scraper application\n\
                             # Font sizes must be between 12.0 and 20.0\n\n\
                             {}", 
                            yaml
                        );
                        let _ = file.write_all(content.as_bytes());
                    }
                }
                
                default_settings
            }
        };
        Mutex::new(settings)
    };
}

// Create a global variable for program details.
// Not included in settings as not user settable.
// This will be available in other files.
lazy_static! {
    static ref DETAILS: Mutex<Details> = {
        let details = Details {
            program_name:           "Scraper".to_string(),
            program_ver:            "0.2.0".to_string(),
            program_date:           "2025".to_string(),
            program_devs:           vec!["mdc".to_string()],
            program_web:            "galacticwingcommander".to_string(),

            min_win_width:          400.0,
            win_width:              450.0,
            max_win_width:          500.0,

            min_win_height:         400.0,
            win_height:             500.0,
            max_win_height:         650.0,

            help_win_width:         450.0,
            
            min_help_win_height:    400.0,
            help_win_height:        450.0,
            max_help_win_height:    500.0,

            gps_win_width:          500.0,
            gps_win_height:         500.0,

            time_series_win_width:  500.0,
            time_series_win_height: 700.0,

            changelog_win_width:    450.0,
            changelog_win_height:   450.0,
        };
        Mutex::new(details)
    };
}

// Application launch.
#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // Create folder for logs if it doesn't already exist.
    let _ = fs::create_dir_all("./logs");

    // Logging configuration held in log4rs.yml.
    // Set up logging.
    let _ = set_up_logging();

    // Get application settings in scope.
    let _settings: Settings = SETTINGS.lock().unwrap().clone();

    // Get application details in scope.
    let details: Details = DETAILS.lock().unwrap().clone();

    // Do initial application information.
    info!("Application: {:?} v({:?})", details.program_name, details.program_ver);  

    info!("Configuring the options for the window.");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([details.win_width, details.win_height])
            .with_resizable(true)
            .with_min_inner_size([details.min_win_width, details.min_win_height])
            .with_max_inner_size([details.max_win_width, details.max_win_height]),
            ..Default::default()
    };

    // Run the application.
    info!("Running the application, creating the window app...");
    eframe::run_native(
        "Scraper",
        options,
        Box::new(|cc| {
            // Force dark theme.
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(MyApp::default()))
        }),
    )
}

// Function to set up default logging if log settings file
// is not available or invalid.
fn set_up_logging() {
    // Attempt to open logging file.
    match log4rs::init_file("log4rs.yml", Default::default()) {
        Ok(_) => {},

        // Log settings not found or invalid, so
        // create default log4rs.yml file.
        Err(_) => {
            // Create default log4rs.yml content.
            // Formatting is important so don't mess with it.
            let default_log_config = "\
appenders:
  log_file:
    kind: rolling_file
    append: true
    path: \"logs/scraper.log\"
    encoder:
      pattern: \"{h({d(%d-%m-%Y %H:%M:%S)})} - {l:<5} {t}:{L} - {m}{n}\"
    policy:
      kind: compound
      trigger:
        kind: size
        limit: 10mb
      roller:
        kind: fixed_window
        base: 1
        count: 3
        pattern: \"logs/scraper{}.log\"

root:
  level: debug
  appenders:
    - log_file
";
            // Try to create the log4rs.yml file.
            if let Ok(mut file) = File::create("log4rs.yml") {
                let _ = file.write_all(default_log_config.as_bytes());
            }

            // Now try to initialize logging again with the newly created file.
            match log4rs::init_file("log4rs.yml", Default::default()) {
                Ok(_) => {},
                // If it still fails, fall back to console logging
                Err(_) => {
                    use log4rs::append::console::ConsoleAppender;
                    use log4rs::encode::pattern::PatternEncoder;
                    use log4rs::config::{Appender, Config, Root};
                    use log::LevelFilter;

                    let stdout = ConsoleAppender::builder()
                        .encoder(Box::new(PatternEncoder::new(
                            "{h({d(%H:%M:%S)})} - {m}{n}"
                        )))
                        .build();

                    let config = Config::builder()
                        .appender(Appender::builder().build("stdout", Box::new(stdout)))
                        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
                        .unwrap();

                    log4rs::init_config(config).unwrap();
                }
            }
        }
    }
}
