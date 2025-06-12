// Log display on UI.

use crate::egui::Color32;
use crate::egui::RichText;
use crate::egui::{ScrollArea, Ui};

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

        // If UI not ready or nothing to render then return.
        if !ui_state.display_ready || scraped_data.is_empty() {
            return;
        }

        let mut current_trip_header: Option<&ScrapedData> = None;
        let mut trip_events: Vec<(usize, &ScrapedData)> = Vec::new();
        let mut in_trip = false;

        for (index, item) in scraped_data.iter().enumerate() {
            match item.event_type.as_str() {
                "SIGNON" => {
                    // Start a new trip.
                    current_trip_header = Some(item);
                    trip_events.clear();
                    trip_events.push((index, item));
                    in_trip = true;
                }
                "TRIP" => {
                    if in_trip {
                        // Add TRIP event to current trip and then render the complete trip.
                        trip_events.push((index, item));
                        if let Some(trip_data) = current_trip_header {
                            render_trip_section(ui, trip_data, &trip_events);
                        }
                        // End the trip.
                        current_trip_header = None;
                        trip_events.clear();
                        in_trip = false;
                    } else {
                        // TRIP event without SIGNON,
                        // Out of trip events, generally at the start of a log file.
                        // These events get displayed at the top level.
                        render_top_level_event(ui, index, item);
                    }
                }
                _ => {
                    if in_trip {
                        // Add event to current trip.
                        trip_events.push((index, item));
                    } else {
                        // Not in trip.
                        // Display at top level (in between TRIP and SIGNON).
                        render_top_level_event(ui, index, item);
                    }
                }
            }
        }

        // Handle case where data ends without a TRIP event (incomplete trip).
        if in_trip && !trip_events.is_empty() {
            if let Some(trip_data) = current_trip_header {
                render_trip_section(ui, trip_data, &trip_events);
            }
        }
    });
}

// Helper function to render a complete trip.
fn render_trip_section(ui: &mut Ui, trip_data: &ScrapedData, trip_events: &[(usize, &ScrapedData)]) {
    ui.collapsing(
        RichText::new(format!("Trip {} at {}", trip_data.trip_num, &trip_data.date_time))
            .color(Color32::WHITE), 
        |ui| {
            // Display all events for this trip.
            for (index, item) in trip_events {
                let event_id = format!("{}_{}", index, &item.event_type);
                ui.push_id(&event_id, |ui| {
                    ui.collapsing(
                        RichText::new(&item.event_type).color(Color32::GREEN), 
                        |ui| {
                            for (key, value) in &item.ev_detail {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}:", key));
                                    ui.colored_label(Color32::YELLOW, value);
                                });
                            }
                        }
                    );
                });
            }
        }
    );
}

// Helper function to render top-level events,
// that is, events outside of a trip.
fn render_top_level_event(ui: &mut Ui, index: usize, item: &ScrapedData) {
    let event_id = format!("{}_{}", index, &item.event_type);
    ui.push_id(&event_id, |ui| {
        ui.collapsing(
            RichText::new(&item.event_type).color(Color32::GREEN),
            |ui| {
                for (key, value) in &item.ev_detail {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", key));
                        ui.colored_label(Color32::YELLOW, value);
                    });
                }
            }
        );
    });
}
