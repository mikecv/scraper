// Log display on UI.

use log::info;
use log::warn;

use crate::egui::Color32;
 use crate::egui::RichText;
use crate::egui::{ScrollArea, Ui};
use regex::Regex;

use crate::scraper::ScrapedData;

// Simple UI state to hold processed display data.
#[derive(Debug, Clone)]
pub struct UiState {
    pub display_ready: bool,
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
    }
}

// Main rendering function.
// The display entry point.
pub fn render_scraped_data(ui: &mut Ui, ui_state: &mut UiState, scraped_data: &[ScrapedData], available_height: f32) {
    ScrollArea::vertical()
        .max_height(available_height - 10.0)
        .show(ui, |ui| {

            // Set content area where scroll bar will appear.
            ui.set_min_width(500.0);
            
            if !ui_state.display_ready || scraped_data.is_empty() {
                return;
            }

            // Process scraped data in order.
            for item in scraped_data {
                if item.event_type == "SIGNON" {
                    info!("Processing SIGNON event.");

                    let ev_info = ungroup_event_data("SIGNON".to_string(), &item.ev_detail);

                    ui.collapsing(RichText::new(format!("Trip {:} - ({:})", item.trip_num, &item.date_time)).color(Color32::WHITE), |ui| {
                        ui.collapsing(RichText::new("SIGNON").color(Color32::GREEN), |ui| {
                        for (key, value) in &ev_info {
                            ui.horizontal(|ui| {
                            ui.label(format!("{}:", key));
                            ui.colored_label(Color32::YELLOW, value);
                            });
                        }
                    });
                });
            }
        }
    });
}

// Function to expand on the scraped data, i.e. the grouped log data
// not expanded on initially.
fn ungroup_event_data(event_type: String, sub_data: &str) -> Vec<(String, String)> {
    // Initialise result vector.
    let mut result = Vec::new();

    // Search for the event sub-data for the SIGNON event.
    if event_type == "SIGNON" {
        // Get the event sub-data.
        let sub_signon_pattern = Regex::new(r"([-\*\+0-9]+) ([0-9a-fA-F]+) (.+?) ([0-9]+) ([0-9]+) ([0-9]+) (.+?)$");
        let mut _found_sub = false;

        if let Some(captures) = sub_signon_pattern.expect("REASON").captures(sub_data) {

            if let Some(driver_id) = captures.get(1) {
                result.push(("Operator id".to_string(), driver_id.as_str().to_string()));
            }
            if let Some(card_id) = captures.get(2) {
                result.push(("Card id".to_string(), card_id.as_str().to_string()));
            }
            if let Some(sign_stat) = captures.get(3) {
                result.push(("Result".to_string(), sign_stat.as_str().to_string()));
            }
            if let Some(bits_read) = captures.get(4) {
                result.push(("Bits read".to_string(), bits_read.as_str().to_string()));
            }
            if let Some(keyboard) = captures.get(5) {
                result.push(("Keyboard".to_string(), keyboard.as_str().to_string()));
            }
            if let Some(card_reader) = captures.get(6) {
                result.push(("Card reader".to_string(), card_reader.as_str().to_string()));
            }
            else {
                warn!("Failed to extract sub-data from SIGNON");
            }
        }
    }   
    result
}
