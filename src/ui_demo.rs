use crate::egui::{ScrollArea, Ui, CollapsingHeader};

// Dummy data structures for testing
#[derive(Debug, Clone)]
pub struct Level3Data {
    pub name: String,
    pub value: String,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct Level2Data {
    pub name: String,
    pub summary: String,
    pub expanded: bool,
    pub level3_items: Vec<Level3Data>,
}

#[derive(Debug, Clone)]
pub struct Level1Data {
    pub name: String,
    pub count: usize,
    pub expanded: bool,
    pub level2_items: Vec<Level2Data>,
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub display_data: Vec<Level1Data>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            display_data: create_dummy_data(),
        }
    }
}

// Create dummy data for testing
pub fn create_dummy_data() -> Vec<Level1Data> {
    vec![
        Level1Data {
            name: "Event Group 1 - SIGNON Events".to_string(),
            count: 5,
            expanded: false,
            level2_items: vec![
                Level2Data {
                    name: "22/04/2025 00:16:44 - SIGNON".to_string(),
                    summary: "Driver 12345 signed on".to_string(),
                    expanded: false,
                    level3_items: vec![
                        Level3Data {
                            name: "Driver ID".to_string(),
                            value: "12345".to_string(),
                            details: "Primary driver".to_string(),
                        },
                        Level3Data {
                            name: "Vehicle".to_string(),
                            value: "Bus 101".to_string(),
                            details: "Route assignment".to_string(),
                        },
                        Level3Data {
                            name: "Location".to_string(),
                            value: "Depot A".to_string(),
                            details: "Starting location".to_string(),
                        },
                    ],
                },
                Level2Data {
                    name: "22/04/2025 08:30:15 - SIGNON".to_string(),
                    summary: "Driver 67890 signed on".to_string(),
                    expanded: false,
                    level3_items: vec![
                        Level3Data {
                            name: "Driver ID".to_string(),
                            value: "67890".to_string(),
                            details: "Relief driver".to_string(),
                        },
                        Level3Data {
                            name: "Vehicle".to_string(),
                            value: "Bus 205".to_string(),
                            details: "Backup vehicle".to_string(),
                        },
                    ],
                },
            ],
        },
        Level1Data {
            name: "Event Group 2 - CHECKLIST Events".to_string(),
            count: 8,
            expanded: false,
            level2_items: vec![
                Level2Data {
                    name: "22/04/2025 00:16:45 - CHECKLIST".to_string(),
                    summary: "Pre-trip inspection completed".to_string(),
                    expanded: false,
                    level3_items: vec![
                        Level3Data {
                            name: "Status".to_string(),
                            value: "OK".to_string(),
                            details: "All systems check passed".to_string(),
                        },
                        Level3Data {
                            name: "Checklist ID".to_string(),
                            value: "8022".to_string(),
                            details: "Standard pre-trip".to_string(),
                        },
                        Level3Data {
                            name: "Items Checked".to_string(),
                            value: "33".to_string(),
                            details: "All mandatory items".to_string(),
                        },
                    ],
                },
            ],
        },
        Level1Data {
            name: "Event Group 3 - ZONECHANGE Events".to_string(),
            count: 12,
            expanded: false,
            level2_items: vec![
                Level2Data {
                    name: "22/04/2025 00:16:46 - ZONECHANGE".to_string(),
                    summary: "Zone changed from 0 to 310".to_string(),
                    expanded: false,
                    level3_items: vec![
                        Level3Data {
                            name: "From Zone".to_string(),
                            value: "0".to_string(),
                            details: "Depot zone".to_string(),
                        },
                        Level3Data {
                            name: "To Zone".to_string(),
                            value: "310".to_string(),
                            details: "Service zone".to_string(),
                        },
                        Level3Data {
                            name: "Time".to_string(),
                            value: "00:16:46".to_string(),
                            details: "Automatic detection".to_string(),
                        },
                    ],
                },
            ],
        },
    ]
}


// Function to render the scrollable collapsible table
// This will be called from draw_central_panel in ui.rs
pub fn render_event_table(ui: &mut Ui, ui_state: &mut UiState, available_height: f32) {
    // Create scrollable area that uses the full available height
    ScrollArea::vertical()
        .max_height(available_height - 10.0) // Leave a small margin
        .show(ui, |ui| {
            
            // Level 1: Main event groups
            for level1_item in &mut ui_state.display_data {
                CollapsingHeader::new(&format!("{} ({})", level1_item.name, level1_item.count))
                    .default_open(level1_item.expanded)
                    .show(ui, |ui| {
                        
                        // Level 2: Individual events
                        for level2_item in &mut level1_item.level2_items {
                            CollapsingHeader::new(&level2_item.name)
                                .default_open(level2_item.expanded)
                                .show(ui, |ui| {
                                    
                                    // Show summary
                                    ui.label(&format!("Summary: {}", level2_item.summary));
                                    ui.separator();
                                    
                                    // Level 3: Event details in a simple table format
                                    crate::egui::Grid::new(&format!("grid_{}", level2_item.name))
                                        .num_columns(3)
                                        .spacing([10.0, 4.0])
                                        .striped(true)
                                        .show(ui, |ui| {
                                            // Table headers
                                            ui.strong("Field");
                                            ui.strong("Value");
                                            ui.strong("Details");
                                            ui.end_row();
                                            
                                            // Table data
                                            for level3_item in &level2_item.level3_items {
                                                ui.label(&level3_item.name);
                                                ui.label(&level3_item.value);
                                                ui.label(&level3_item.details);
                                                ui.end_row();
                                            }
                                        });
                                });
                        }
                    });
            }
        });
}
