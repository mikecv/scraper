// Log display on UI.

use crate::egui::Color32;
use crate::egui::RichText;
use crate::egui::{ScrollArea, Ui};

use crate::scraper::ScrapedData;
use crate::DETAILS;


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
pub fn render_scraped_data(ui: &mut Ui, ui_state: &mut UiState, scraped_data: &[ScrapedData], 
        available_height: f32,
        available_width: f32,
        show_oot_events: bool,
        show_input_events: bool,
        show_report_events: bool,
        show_debug_events: bool,
) {
    // Program settings.
    // Not user setable.
    let _details = DETAILS.lock().unwrap().clone();

    ScrollArea::vertical()
    .max_height(available_height - 10.0)
    .max_width(available_width - 10.0)

    .show(ui, |ui| {

        // If UI not ready or nothing to render then return.
        if !ui_state.display_ready || scraped_data.is_empty() {
            return;
        }

        // Set the scrollable width of the centre panel
        // to fulll available width less a margin.
        ui.set_min_width(available_width - 10.0);

        // Filter the data based on current menu settings.
        let filtered_data: Vec<(usize, &ScrapedData)> = scraped_data
            .iter()
            .enumerate()
            .filter(|(_, item)| should_show_event(item, show_oot_events, show_input_events, show_report_events, show_debug_events))
            .collect();

        let mut current_trip_header: Option<&ScrapedData> = None;
        let mut trip_events: Vec<(usize, &ScrapedData)> = Vec::new();
        let mut in_trip = false;

        // Go through all events and if applicable render them to the UI.
        // If "Show" menu settings that events should be ignored then don't render.
        for (index, item) in filtered_data {
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

// Function to determine if an event should be shown based on current menu filter settings.
fn should_show_event(
    item: &ScrapedData,
    show_oot_events: bool,
    show_input_events: bool,
    show_report_events: bool,
    show_debug_events: bool,
) -> bool {
    match item.event_type.as_str() {
        // Always show SIGNON events.
        "SIGNON" => true,
        // Show TRIP events unless not on trip.
        "TRIP" => item.on_trip,
        // Show these events according to the Show menut settings,
        // unless they are out of trip.
        "REPORT" => show_report_events && item.on_trip,
        "DEBUG" => show_debug_events && item.on_trip,
        "INPUT" => show_input_events && item.on_trip,
        _ => {
            // For other events, decide based on whether they're on trip.
            if item.on_trip {
                true
            } else {
                show_oot_events
            }
        }
    }
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
                                    ui.colored_label(Color32::GOLD, value);
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
        RichText::new(&item.event_type).color(Color32::GRAY),
        |ui| {
            for (key, value) in &item.ev_detail {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", key));
                    ui.colored_label(Color32::GOLD, value);
                });
            }
        }
    );
    });
}
