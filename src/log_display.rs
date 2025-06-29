// Log display on UI.use log::info;

use log::info;

use eframe::egui;
use eframe::egui::{RichText};

use crate::egui::{ScrollArea, Ui};

use crate::colours;
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
pub fn render_scraped_data(
        ui: &mut Ui,
        ui_state: &mut UiState,
        scraped_data: &[ScrapedData], 
        available_height: f32,
        available_width: f32,
        show_oot_events: bool,
        show_input_events: bool,
        show_report_events: bool,
        show_debug_events: bool,
        selected_id: &mut Option<String>,
        dark_mode: bool,
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
                            render_trip_section(ui, trip_data, &trip_events, selected_id, dark_mode);
                        }
                        // End the trip.
                        current_trip_header = None;
                        trip_events.clear();
                        in_trip = false;
                    } else {
                        // TRIP event without SIGNON,
                        // Out of trip events, generally at the start of a log file.
                        // These events get displayed at the top level.
                        render_top_level_event(ui, index, item, dark_mode);
                    }
                }
                _ => {
                    if in_trip {
                        // Add event to current trip.
                        trip_events.push((index, item));
                    } else {
                        // Not in trip.
                        // Display at top level (in between TRIP and SIGNON).
                        render_top_level_event(ui, index, item, dark_mode);
                    }
                }
            }
        }

        // Handle case where data ends without a TRIP event (incomplete trip).
        if in_trip && !trip_events.is_empty() {
            if let Some(trip_data) = current_trip_header {
                render_trip_section(ui, trip_data, &trip_events, selected_id, dark_mode);
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
fn render_trip_section(
    ui: &mut Ui,
    trip_data: &ScrapedData,
    trip_events: &[(usize, &ScrapedData)],
    selected_id: &mut Option<String>,
    dark_mode: bool)
{
    // Generate a unique trip ID.
    let trip_id = format!("{}", trip_data.trip_num);
    let _is_trip_selected = selected_id.as_ref() == Some(&trip_id);
    
    ui.push_id(&trip_id, |ui| {
        let trip_header_response = ui.collapsing(
            RichText::new(format!("TRIP {:} - {}", trip_data.trip_num, &trip_data.date_time))
                .color(colours::trip_colour(dark_mode))
                .family(egui::FontFamily::Monospace)
                .strong(),
            |ui| {
                // Display all events for this trip.
                for (index, item) in trip_events {
                    let event_id = format!("event_{}_{}", index, &item.event_type);
                    let _is_event_selected = selected_id.as_ref() == Some(&trip_data.trip_num);
                    ui.push_id(&event_id, |ui| {
                        let event_header_response = ui.collapsing(
                            // Event name and the date/time.
                            RichText::new(format!("{:20} {}",&item.event_type, &item.date_time))
                                .color(colours::event_colour(dark_mode))
                                .family(egui::FontFamily::Monospace),
                            |ui| {
                                // Do the event detail key-value pairs
                                for (key, value) in &item.ev_detail {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(format!("{:20}", key))
                                            .color(colours::key_colour(dark_mode))
                                            .family(egui::FontFamily::Monospace)
                                            .italics());
                                        ui.label(RichText::new(format!("{}", value))
                                            .color(colours::value_colour(dark_mode))
                                            .family(egui::FontFamily::Monospace)
                                            .italics());
                                    });
                                }
                                // Add the gps lat/long value from GPS to key value data for the event.
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("{:20}", "GPS Latitude"))
                                        .color(colours::gps_key_colour(dark_mode))
                                        .family(egui::FontFamily::Monospace)
                                        .italics());
                                    ui.label(RichText::new(format!("{}", &item.gps_locn.lat))
                                        .color(colours::gps_value_colour(dark_mode))
                                        .family(egui::FontFamily::Monospace)
                                        .italics());
                                });
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("{:20}", "GPS Longitude"))
                                        .color(colours::gps_key_colour(dark_mode))
                                        .family(egui::FontFamily::Monospace)
                                        .italics());
                                    ui.label(RichText::new(format!("{}", &item.gps_locn.long))
                                        .color(colours::gps_value_colour(dark_mode))
                                        .family(egui::FontFamily::Monospace)
                                        .italics());
                                });
                                // Add the speed from GPS to key value data for the event.
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("{:20}", "GPS Speed"))
                                        .color(colours::gps_key_colour(dark_mode))
                                        .family(egui::FontFamily::Monospace)
                                        .italics());
                                    ui.label(RichText::new(format!("{}", &item.gps_speed))
                                        .color(colours::gps_value_colour(dark_mode))
                                        .family(egui::FontFamily::Monospace)
                                        .italics());
                                });
                                // Add the gps RSSI value from GPS to key value data for the event.
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("{:20}", "GPS RSSI"))
                                        .color(colours::gps_key_colour(dark_mode))
                                        .family(egui::FontFamily::Monospace)
                                        .italics());
                                    ui.label(RichText::new(format!("{}", &item.gps_rssi))
                                        .color(colours::gps_value_colour(dark_mode))
                                        .family(egui::FontFamily::Monospace)
                                        .italics());
                                });
                            }
                        );
                        
                        // Check if event header was clicked.
                        if event_header_response.header_response.clicked() {
                            *selected_id = Some(trip_data.trip_num.to_string());
                            handle_event_selected(&trip_data.trip_num, &item.event_type);
                        }
                    });
                }
            }
        );
        
        // Check if trip header was clicked.
        if trip_header_response.header_response.clicked() {
            *selected_id = Some(trip_data.trip_num.to_string());
            handle_trip_selected(&trip_data.trip_num);
        }
    });
}

// Helper function to render top-level events,
// that is, events outside of a trip.
// Note that these out of trip events aren't selectable.
fn render_top_level_event(
    ui: &mut Ui,
    index: usize,
    item: &ScrapedData,
    dark_mode: bool)
{
    let event_id = format!("{}_{}", index, &item.event_type);
    ui.push_id(&event_id, |ui| {
        ui.collapsing(
            // Event name and the date/time.
            RichText::new(format!("{:} {}",&item.event_type, &item.date_time))
                .color(colours::subtle_colour(dark_mode))
                .family(egui::FontFamily::Monospace),
            |ui| {
                
                // Do the event data key value pairs.
                for (key, value) in &item.ev_detail {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{:20}", key))
                            .color(colours::subtle_colour(dark_mode))
                            .family(egui::FontFamily::Monospace)
                            .italics());
                        ui.label(RichText::new(format!("{}", value))
                            .color(colours::subtle_colour(dark_mode))
                            .family(egui::FontFamily::Monospace)
                            .italics());
                    });
                }
                // Add the gps lat/long value from GPS to key value data for the event.
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{:20}", "GPS Latitude"))
                        .color(colours::subtle_colour(dark_mode))
                        .family(egui::FontFamily::Monospace)
                        .italics());
                    ui.label(RichText::new(format!("{}", &item.gps_locn.lat))
                        .color(colours::subtle_colour(dark_mode))
                        .family(egui::FontFamily::Monospace)
                        .italics());
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{:20}", "GPS Longitude"))
                        .color(colours::subtle_colour(dark_mode))
                        .family(egui::FontFamily::Monospace)
                        .italics());
                    ui.label(RichText::new(format!("{}", &item.gps_locn.long))
                        .color(colours::subtle_colour(dark_mode))
                        .family(egui::FontFamily::Monospace)
                        .italics());
                });
                // Add the speed value from GPS to key value data for the event.
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{:20}", "GPS Speed"))
                        .color(colours::subtle_colour(dark_mode))
                        .family(egui::FontFamily::Monospace)
                        .italics());
                    ui.label(RichText::new(format!("{}", &item.gps_speed))
                        .color(colours::subtle_colour(dark_mode))
                        .family(egui::FontFamily::Monospace)
                        .italics());
                });
                // Add the gps RSSI value from GPS to key value data for the event.
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{:20}", "GPS RSSI"))
                        .color(colours::subtle_colour(dark_mode))
                        .family(egui::FontFamily::Monospace)
                        .italics());
                    ui.label(RichText::new(format!("{}", &item.gps_rssi))
                        .color(colours::subtle_colour(dark_mode))
                        .family(egui::FontFamily::Monospace)
                        .italics());
                });
            }
        );
    });
}

// Handler functions for trip header selection.
fn handle_trip_selected(trip_num: &str) {
    info!("Trip selected: {}", trip_num);
}

// Handler functions for event selection.
fn handle_event_selected(trip_num: &str, event_type: &str) {
    info!("Trip selected: {}, event {}", trip_num, event_type);
}
