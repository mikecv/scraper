// Place for changelog content.
// Refer to ui.rs for associated ui definitions.

use eframe::egui;

use crate::app::MyApp;

// Render changelog entriest using eframe calls.
pub fn draw_changelog_content(ui: &mut egui::Ui, _app: &mut MyApp) {
    ui.heading("Scraper Changelog");
    ui.separator();
    
    ui.collapsing("1.0.0 - Initial release", |ui| {
        ui.label("Initial release with trip and most event logs.");
        ui.label("Includes all event sub-data, as well as gps data.");
        ui.label("Options to show diagnostic events, and out of trip events.");
        ui.label("Support for light and dark display modes.");
        ui.label("Plotting of gps breadcrumb trails included, with and without OSM tiles.");
    });
}
