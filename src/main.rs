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

mod settings;
mod app;
mod scraper;
mod ui;
mod help_content;

// Create a global variable for application settings.
// This will be available in other files.
lazy_static! {
    static ref SETTINGS: Mutex<Settings> = {
        // Read YAML settings file.
        let mut file = File::open("settings.yml").expect("Unable to open file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Unable to read file");
        
        // Deserialize YAML into Settings struct.
        let settings: Settings = serde_yaml::from_str(&contents).expect("Unable to parse YAML");
        Mutex::new(settings)
    };
}

// Application launch.
#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // Create folder for logs if it doesn't already exist.
    let _ = fs::create_dir_all("./logs");

    // Logging configuration held in log4rs.yml .
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    // Get application settings in scope.
    let settings: Settings = SETTINGS.lock().unwrap().clone();

    // Do initial application information.
    info!("Application started: {:?} v({:?})", settings.program_name, settings.program_ver);  
    info!("Application devs: {:?} web: ({:?})", settings.program_devs, settings.program_web);  

    // Configure the native options for the window.
    info!("Configuring the options for the window.");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([settings.win_width, settings.win_height]),
        ..Default::default()
    };

    // Run the application.
    info!("Running the application, creating the window app...");
    eframe::run_native(
        "Scraper",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}
