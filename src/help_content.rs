// Place for help content.
// Refer to ui.rs for associated ui definitions.

use eframe::egui;

use crate::app::MyApp;

// Render all help content using eframe calls.
pub fn draw_help_content(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.heading("Scraper Help");
    ui.separator();
    
    ui.collapsing("1.0 Getting Started", |ui| {
        ui.label("From the main menu select 'File' / 'Open' to select a file to process.");
        ui.label("Alternatively, drag and drop a file onto the application window.");
    });
    ui.collapsing("2.0 Scraped Data", |ui| {
        ui.label("On load a scraped log file will list the trips in the file as illusrated below.");
    });

    // Loaded file top level.
    if let Some(texture) = &app.help_image_1 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        // ui.add(egui::Image::new(texture).max_size(egui::Vec2::new(400.0, 300.0)));
        ui.add_space(10.0);
    }

    ui.collapsing("2.1 Initial View", |ui| {
        ui.label("At the top of the image above we see that the log file contains 5 trips.");
        ui.label("The trip is identified by the trip number, and the date and time of the start of the trip.");
        ui.label("Notice the small arrow at the start of each trip - this indicates that the trip has collapsed data associated with it.");
        ui.label("Clicking on the trip label will expand 1 level below.");
    });
    
    ui.collapsing("2.1 Program info and status", |ui| {
        ui.label("At the bottom of the screen information about the file, and status of the program is shown as illustrated below.");
        ui.label("Also in the bottom panel is the detected controller ID, and the firmware version running on the controller.");
        ui.label("Note that the controller ID and firmware version is only the first of each record encountered in the file.");
        ui.label("At the far right is the trip ID of the currently selected trip.");
    });

    // Loaded file info panel.
    if let Some(texture) = &app.help_image_2 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        // ui.add(egui::Image::new(texture).max_size(egui::Vec2::new(400.0, 300.0)));
        ui.add_space(10.0);
    }
}
