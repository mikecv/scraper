// Log display on UI.

use crate::egui::{ScrollArea, Ui};
use crate::scraper::ScrapedData;

// Simple UI state to hold processed display data
#[derive(Debug, Clone)]
pub struct UiState {
    pub display_ready: bool,
    // Add your own fields here as needed
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            display_ready: false,
        }
    }
}

impl UiState {
    // Process scraped data for display.
    pub fn update_with_scraped_data(&mut self, scraped_data: &[ScrapedData]) {
        if scraped_data.is_empty() {
            self.display_ready = false;
            return;
        }
        
        self.display_ready = true;
        
        // TODO: Process your scraped_data here
        // Transform it into whatever structure you need for display
        // For example:
        // - Group by date, event type, etc.
        // - Filter data
        // - Sort data
        // - Calculate summaries
    }
}

// Main rendering function.
// The display entry point.
pub fn render_scraped_data(ui: &mut Ui, ui_state: &mut UiState, scraped_data: &[ScrapedData], available_height: f32) {
    ScrollArea::vertical()
        .max_height(available_height - 10.0)
        .show(ui, |ui| {
            
            if !ui_state.display_ready || scraped_data.is_empty() {
                ui.label("No data to display. Load a file to see results.");
                return;
            }

            // TODO: Add your display logic here
            // This is where you'll render your processed data
            
            // Simple example to get you started:
            ui.heading("Scraped Events");
            ui.separator();
            
            for (index, item) in scraped_data.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", index + 1));
                    ui.label(&item.date_time);
                    ui.label(&item.event_type);
                    if item.new_trip {
                        ui.colored_label(ui.visuals().warn_fg_color, "NEW TRIP");
                    }
                });
                ui.label(&item.ev_detail);
                ui.separator();
            }
        });
}
