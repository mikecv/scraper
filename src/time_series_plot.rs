// Draw the time series data plots to a separate UI.

use log::debug;

use eframe::egui;

use crate::scraper::{Scraper};

// Function to plot time series data using custom drawing.
pub fn plot_time_series_data(ui: &mut egui::Ui, _scraper: &Scraper, selected_id: &Option<String>) {

    // Get id of selected trip, or show prompt if no trip selected.
    let selected_trip = match selected_id.as_ref() {
        Some(id) if !id.is_empty() => id,
        _ => {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("Please select a trip to plot time series data.");
            });
            return;
        }
    };

    debug!("Selected trip number: {}", selected_trip);
}
