// Place for help content.
// Refer to ui.rs for associated ui definitions.

use eframe::egui;

use crate::app::MyApp;

// Render all help content using eframe calls.
// Temporary help content shown.
// TODO - replace content below with actual content.
pub fn draw_help_content(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.heading("Web Scraper Help");
    ui.separator();
    
    ui.collapsing("Getting Started", |ui| {
        ui.label("1. Click 'File' → 'Open' to select a file to process");
        ui.label("2. Or drag and drop a file onto the application window");
        ui.label("3. Configure your scraping settings in the main panel");
        ui.label("4. Click 'Start Scraping' to begin processing");

        // Display first help image.
        if let Some(texture) = &app.help_image_1 {
            ui.add_space(10.0);
            ui.image((texture.id(), egui::Vec2::new(400.0, 200.0))); 
            ui.add_space(10.0);
        }
    });

    ui.collapsing("Supported File Types", |ui| {
        ui.label("• Log files (.txt, .log)");
        ui.label("• CSV files (.csv)");
        ui.label("• JSON files (.json)");
        ui.label("• Custom formats (configurable)");
    });
    
    ui.collapsing("Features", |ui| {
        ui.label("• Fast file processing");
        ui.label("• Multiple output formats");
        ui.label("• Real-time progress tracking");
        ui.label("• Batch processing support");
    });
    
    ui.collapsing("Keyboard Shortcuts", |ui| {
        ui.label("Ctrl+O: Open file");
        ui.label("Ctrl+Q: Quit application");
        ui.label("F1: Show this help");
    });
    
    ui.collapsing("Troubleshooting", |ui| {
        ui.label("If you encounter issues:");
        ui.label("• Check that your file is not corrupted");
        ui.label("• Ensure you have read permissions");
        ui.label("• Try a smaller file first");
        ui.label("• Check the application logs");
    });
    
    ui.separator();
    
    ui.horizontal(|ui| {
        if ui.button("Close Help").clicked() {
            app.show_help = false;
        }
    });
}
