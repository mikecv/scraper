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

        // Display first help image.
        if let Some(texture) = &app.help_image_1 {
            ui.add_space(10.0);
            ui.image((texture.id(), egui::Vec2::new(400.0, 200.0))); 
            ui.add_space(10.0);
        }
    });

    ui.collapsing("Supported File Types", |ui| {
        ui.label("• Debuglog files");
        ui.label("• Debuglog CSV files (.csv)");
    });
    
    ui.collapsing("Features", |ui| {
        ui.label("• Scrapes debuglogs for trip details.");
        ui.label("• Reports all events belonging to a trip.");
        ui.label("• Reports all parameter values for events.");
    });
    
    ui.collapsing("Keyboard Shortcuts", |ui| {
        ui.label("Ctrl+O: Open file.");
        ui.label("Ctrl+Q: Quit application.");
        ui.label("F1: Show this help.");
    });
    
    ui.collapsing("Troubleshooting", |ui| {
        ui.label("If you encounter issues:");
        ui.label("• Check that your file is a valid debuglog file.");
        ui.label("• Check the application logs.");
    });
    
    ui.separator();
    
    ui.horizontal(|ui| {
        if ui.button("Close Help").clicked() {
            app.show_help = false;
        }
    });
}
