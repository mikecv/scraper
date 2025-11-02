// Place for changelog content.
// Refer to ui.rs for associated ui definitions.

use eframe::egui;

use crate::app::MyApp;

// Render changelog entriest using eframe calls.
pub fn draw_changelog_content(ui: &mut egui::Ui, _app: &mut MyApp) {
    ui.heading("Scraper Changelog");
    ui.separator();

    ui.collapsing("0.4.0 - update release", |ui| {
        ui.label("Added INPUT events to time series plots.");
        ui.label("Automatically accept certificates for tiles links.");
    });

    ui.collapsing("0.3.0 - update release", |ui| {
        ui.label("Expanded list of events supported in processing.");
        ui.label("Added pan and zoom too plain gps plots.");
        ui.label("Create log config file on start-up if not existing.");
        ui.label("Updates to help.");
    });

    ui.collapsing("0.2.0 - update release", |ui| {
        ui.label("Bug fixes to window sizing.");
        ui.label("Added time series data plotting, for analog, digital, and impulse series.");
        ui.label("Added display of unsupported events; no elaboration of event attributes.");
        ui.label("Added pan/zoom and time cursor function to time series plots.");
    });

    ui.collapsing("0.1.0 - Initial release", |ui| {
        ui.label("Log parsing for trips and most events.");
        ui.label("Includes all event sub-data, as well as gps data.");
        ui.label("Options to show diagnostic events, and out of trip events.");
        ui.label("Support for light and dark display modes.");
        ui.label("Plotting of gps breadcrumb trails included, including with street view or satellite view tiles.");
    });
}
