// Place for help content.
// Refer to ui.rs for associated ui definitions.

use eframe::egui;

use crate::app::MyApp;

// Render all help content using eframe calls.
pub fn draw_help_content(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.heading("Debuglog Scraper Help");
    ui.separator();
    
    ui.collapsing("Getting Started", |ui| {
        ui.label("1. Click 'File' / 'Open' to select a file to process.");
        ui.label("2. Or drag and drop a file onto the application window.");
    });
    
    ui.separator();
    
    ui.horizontal(|ui| {
        if ui.button("Close Help").clicked() {
            app.show_help = false;
        }
    });
}
