// Log display on UI.
use crate::egui::{ScrollArea, Ui, CollapsingHeader};
use crate::scraper::ScrapedData;

// Simple data structures for your actual scraped data
#[derive(Debug, Clone)]
pub struct EventDetail {
    pub name: String,
    pub value: String,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct EventItem {
    pub date_time: String,
    pub event_type: String,
    pub summary: String,
    pub expanded: bool,
    pub details: Vec<EventDetail>,
}

#[derive(Debug, Clone)]
pub struct EventGroup {
    pub name: String,
    pub count: usize,
    pub expanded: bool,
    pub events: Vec<EventItem>,
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub event_groups: Vec<EventGroup>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            event_groups: Vec::new(),
        }
    }
}

impl UiState {
    pub fn update_with_scraped_data(&mut self, scraped_data: &[ScrapedData]) {
        self.event_groups = create_event_groups_from_scraped_data(scraped_data);
    }
}

// Convert your scraped data into UI-friendly groups
fn create_event_groups_from_scraped_data(scraped_data: &[ScrapedData]) -> Vec<EventGroup> {
    if scraped_data.is_empty() {
        return Vec::new();
    }

    // For now, just put all events in a single group
    // Later you can group by event type, date, etc.
    let events: Vec<EventItem> = scraped_data.iter().map(|item| {
        EventItem {
            date_time: item.date_time.clone(),
            event_type: item.event_type.clone(),
            summary: item.ev_detail.clone(),
            expanded: false,
            details: vec![
                EventDetail {
                    name: "Event Type".to_string(),
                    value: item.event_type.clone(),
                    details: if item.new_trip { "New Trip" } else { "Continuing Trip" }.to_string(),
                },
                EventDetail {
                    name: "Details".to_string(),
                    value: item.ev_detail.clone(),
                    details: "Raw event data".to_string(),
                },
            ],
        }
    }).collect();

    vec![EventGroup {
        name: "All Events".to_string(),
        count: scraped_data.len(),
        expanded: false,
        events,
    }]
}

// Simplified render function.
pub fn render_event_table(ui: &mut Ui, ui_state: &mut UiState, available_height: f32) {
    ScrollArea::vertical()
        .max_height(available_height - 10.0)
        .show(ui, |ui| {
            
            // If nothing to display then return now.
            if ui_state.event_groups.is_empty() {
                return;
            }

            // Event groups.
            for group in &mut ui_state.event_groups {
                CollapsingHeader::new(&format!("{} ({})", group.name, group.count))
                    .default_open(group.expanded)
                    .show(ui, |ui| {
                        
                        // Individual events
                        for event in &mut group.events {
                            CollapsingHeader::new(&format!("{} - {}", event.date_time, event.event_type))
                                .default_open(event.expanded)
                                .show(ui, |ui| {
                                    
                                    ui.label(&format!("Summary: {}", event.summary));
                                    ui.separator();
                                    
                                    // Event details in a table
                                    crate::egui::Grid::new(&format!("grid_{}", event.date_time))
                                        .num_columns(3)
                                        .spacing([10.0, 4.0])
                                        .striped(true)
                                        .show(ui, |ui| {
                                            ui.strong("Field");
                                            ui.strong("Value");
                                            ui.strong("Details");
                                            ui.end_row();
                                            
                                            for detail in &event.details {
                                                ui.label(&detail.name);
                                                ui.label(&detail.value);
                                                ui.label(&detail.details);
                                                ui.end_row();
                                            }
                                        });
                                });
                        }
                    });
            }
        });
}
