// Application to perform custom scraping of log files,
// and present results in tabular and graphical format.

// Release build for Windows without launching a console window.
#![windows_subsystem = "windows"]

use log::info;
use eframe::egui;
use std::fs;

use app::MyApp;

mod settings;
mod setting_up;
mod app;
mod logging;
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

// Application launch.
#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // Create folder for logs if it doesn't already exist.
    let _ = fs::create_dir_all("./logs");

    // Logging configuration held in log4rs.yml.
    // Set up logging.
    let _ = logging::set_up_logging();

    // Get application settings in scope (triggers lazy initialization).
    let _settings = setting_up::SETTINGS.lock().unwrap().clone();

    // Get application details in scope.
    let details = setting_up::DETAILS.lock().unwrap().clone();

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
