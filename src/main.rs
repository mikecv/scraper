// Application to perform custom scraping of log files,
// and present results in tabular and graphical format.

use log::info;
use log4rs;

use eframe::{egui};
use lazy_static::lazy_static;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::sync::{Mutex};

use app::MyApp;
use crate::settings::Settings;
use crate::settings::Details;

mod settings;
mod app;
mod scraper;
mod ui;
mod help_content;
mod log_display;
mod colours;
mod plots;

// Create a global variable for application settings.
// This will be available in other files.
lazy_static! {
static ref SETTINGS: Mutex<Settings> = {
        // Try to read YAML settings file.
        let settings = match File::open("settings.yml") {
            Ok(mut file) => {
                // Logging settings found and read.
                let mut contents = String::new();
                match file.read_to_string(&mut contents) {
                    Ok(_) => {
                        // Try to parse YAML, use defaults if parsing fails.
                        match serde_yaml::from_str(&contents) {
                            Ok(settings) => settings,
                            // Setting yaml file invalid.
                            Err(_) => Settings::default(),
                        }
                    }
                    // Can't read from file.
                    Err(_) => Settings::default(), // Can't read file, use defaults
                }
            }
            // File doesn't exist.
            Err(_) => Settings::default(), // File doesn't exist, use defaults
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
            program_name: "Scraper".to_string(),
            program_ver: "0.1.0".to_string(),
            program_date: "2025".to_string(),
            program_devs: vec!["mdc".to_string()],
            program_web: "galacticwingcommander".to_string(),
            win_width: 500.0,
            win_height: 600.0,
            help_win_width: 500.0,
            help_win_height: 600.0,
            gps_win_width: 500.0,
            gps_win_height: 500.0,
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
            .with_min_inner_size([details.win_width, details.win_height])
            .with_inner_size([details.win_width, details.win_height]),
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

// Function to set up defaulr logging if log settings file
// not available.
fn set_up_logging() {
    // Attempt to open logging file.
    match log4rs::init_file("log4rs.yml", Default::default()) {
        Ok(_) => {},

        // Log settings not found or invalid, so
        // set up default console logging instead.
        Err(_) => {
            // log4rs.yml missing or invalid
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
