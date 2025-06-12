// Scraper structure and methods.

use log::info;
use log::warn;

use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::mpsc;
use tinyfiledialogs::open_file_dialog;

use crate::egui;

#[allow(dead_code)]

#[derive(Debug)]
pub enum FileDialogMessage {
    FileSelected(PathBuf),
    DialogClosed,
}

// Data that is scraped.
#[derive(Debug)]
pub struct ScrapedData {
    pub date_time: String,
    pub _on_trip: bool,
    pub _new_trip: bool,
    pub trip_num: String,
    pub event_type: String,
    pub ev_detail: Vec<(String, String)>,
}

// Scraper struct and methods.
#[derive(Debug)]
pub struct Scraper {
    pub selected_file: Option<PathBuf>,
    pub file_dialog_open: bool,
    pub file_receiver: Option<mpsc::Receiver<FileDialogMessage>>,
    pub processing_status: String,
    pub controller_id: String,
    pub controller_fw: String,
    pub scrapings: Vec<ScrapedData>,
}

// Implement Scraper class.
impl Scraper {
    // A function to create a new Scraper instance.
    pub fn new() -> Self {
        info!("Creating new instance of Scraper.");

        Self {
            selected_file: None,
            file_dialog_open: false,
            file_receiver: None,
            processing_status: "No file selected.".to_string(),
            controller_id: "".to_string(),
            controller_fw: "".to_string(),
            scrapings: Vec::new(),
        }
    }
}

impl Scraper {
    pub fn load_file(&mut self, ctx: &egui::Context) {
        info!("Browsing for file to open.");

        // Prevent multiple dialogs.
        if self.file_dialog_open {
            return;
        }

        self.file_dialog_open = true;

        // Use tinyfiledialogs synchronous dialog.
        let file_path = open_file_dialog(
            "Select log file",
            "",
            Some((&["*.log", "*.bak", "*.csv"], "Log files (log, bak, csv)")),
        );

        match file_path {
            Some(path_string) => {
                let path = PathBuf::from(path_string);
                info!("File selected: {:?}", path);
                self.selected_file = Some(path.clone());
                self.process_file(&path);
            }
            None => {
                info!("No file was selected.");
            }
        }

        self.file_dialog_open = false;
        ctx.request_repaint();
    }

    // Method to reinitialize/clear data before loading new file.
    // This is required as there is no close file menu option.
    pub fn reinitialize_data(&mut self) {
        info!("Reinitializing scraper data for new file.");
        // self.selected_file = None;
        self.processing_status = "Loading new file...".to_string();
        self.controller_id = "".to_string();
        self.controller_fw = "".to_string();
        // Clear any ongoing file dialog state.
        self.file_dialog_open = false;
        self.file_receiver = None;
        self.scrapings.clear();
    }

    // Method to load file from a given path.
    // Required for drag and drop files.
    pub fn load_file_from_path(&mut self, path: &std::path::Path) {
        // First initialize scraped data.
        self.reinitialize_data();

        info!("Loading file from path: {:?}", path);
        
        let path_buf = path.to_path_buf();
        self.selected_file = Some(path_buf.clone());
        self.process_file(&path_buf);
    }

    // Method to scrape the selected file.
    fn process_file(&mut self, path: &PathBuf) {
        // First initialize scraped data.
        self.reinitialize_data();

        info!("Processing file: {:?}", path);

        match self.read_and_process_file(path) {
            Ok(_sn) => {
                self.processing_status = format!("Successfully completed processing.");
                info!("Successfully completed processing.");
            }
            Err(e) => {
                self.processing_status = format!("Error processing file: {}", e);
                info!("File processing error: {}", e);
            }
        }
    }

    // Main file processing logic.
    fn read_and_process_file(&mut self, path: &PathBuf) -> Result<usize, Box<dyn std::error::Error>> {

        // Clear fields at start of processing to ensure clean state.
        self.controller_id.clear();
        self.controller_fw.clear();

        // Open the file.
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        info!("Searching file for controller serial number.");
    
        // Get the serial number of the controller.
        let sn_pattern = Regex::new(r"([0-9]{1,2}/[0-9]{2}/[0-9]{4}) ([0-9]{1,2}:[0-9]{2}:[0-9]{2}(?:\.\d{3})?(?: [AP]M)?)[:, ]UNIT ([0-9]+)$")?;
        let mut found_sn = false;
        
        // Process file line by line,
        for line_result in reader.lines() {
            let line = line_result?;
            
            // Check if we should stop processing.
            if let Some(captures) = sn_pattern.captures(&line) {
                found_sn = true;
                // Group 3 contains the serialnumber.
                let sn_str = captures.get(3).unwrap().as_str();
                self.controller_id = sn_str.to_string();
                info!("Found controller s/n: {:0>6}", sn_str); 
            }
            if found_sn == true {
                break
            }
        }
        if found_sn == false {
            info!("Failed to find controller serial number."); 
        }

        // Initialise file reader again.
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        info!("Searching file for controller firmware version.");
       
        // Get the controller firmware version.
        let fw_pattern = Regex::new(r"([0-9]{1,2}/[0-9]{2}/[0-9]{4}) ([0-9]{1,2}:[0-9]{2}:[0-9]{2}\.[0-9]{3}) EVENT ([0-9]+) ([0-9]+) (.+)/(.+)/(.+)/([-0-9]+)/([0-9]+) SWSTART (.+) ([.0-9]+.+) v(.+)$")?;
        let mut found_fw = false;


        // Process file line by line,
        for line_result in reader.lines() {
            let line = line_result?;
            
            // Check if we should stop uprocessing.
            if let Some(captures) = fw_pattern.captures(&line) {
                found_fw = true;
                // Group 11 contains the firmware version.
                let fw_str = captures.get(11).unwrap().as_str();
                self.controller_fw = fw_str.to_string();
                info!("Found controller firmware: {:?}", fw_str); 
            }
            if found_fw == true {
                break
            }
        }
        if found_fw == false {
            info!("Failed to find controller firmware version."); 
        }

        // Initialise file reader again.
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        info!("Searching file for controller events.");

        // Get the controller events.
        let ev_pattern = Regex::new(r"([0-9]{1,2}/[0-9]{2}/[0-9]{4}) ([0-9]{1,2}:[0-9]{2}:[0-9]{2}\.[0-9]{3}) EVENT ([0-9]+) ([0-9]+) ([-0-9]+)/([0-9]+)/([0-9]+)/([0-9]+)/([0-9]+) ([A-Z_]+) (.+)$")?;

        // Track if we are in or out of trip.
        let mut intrip = false;

        // Process file line by line.
        for line_result in reader.lines() {
            let line = line_result?;
            
            // Check for event pattern
            if let Some(captures) = ev_pattern.captures(&line) {
                
                // Extract key fields for logging.
                let date = captures.get(1).unwrap().as_str();
                let time = captures.get(2).unwrap().as_str();
                let event_type = captures.get(10).unwrap().as_str();
                let event_detail = captures.get(11).unwrap().as_str();
                let ev_key_vals = ungroup_event_data(event_type.to_string(), event_detail);
                let trip_id = captures.get(3).unwrap().as_str();

                // Keep track of on-trip state.
                // SIGNON sets TRIP clears.
                if event_type == "SIGNON" {
                    intrip = true;
                } else if event_type == "TRIP" {
                    intrip = false;
                }

                // Create and populate the struct correctly
                let ev_data = ScrapedData {
                    date_time: format!("{} {}", date, time),
                    _on_trip: intrip,
                    _new_trip: intrip,
                    trip_num: trip_id.to_string(),
                    event_type: event_type.to_string(),
                    ev_detail: ev_key_vals,
                };

                // Push the struct onto the vector.
                self.scrapings.push(ev_data);
            }
        }
        Ok(0)
    }
       
    // Method to get path and filename for display.
    // Not currently used.
    pub fn _get_selected_file(&self) -> Option<&PathBuf> {
        self.selected_file.as_ref()
    }

    // Method to get just the filename for display.
    pub fn get_selected_filename(&self) -> Option<String> {
        self.selected_file.as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    }

    // Get processing status for display.
    pub fn get_processing_status(&self) -> &str {
        &self.processing_status
    }
}

// Function to expand on the scraped data.
fn ungroup_event_data(event_type: String, sub_data: &str) -> Vec<(String, String)> {
    // Initialise result vector.
    let mut result = Vec::new();

    // Search for the event sub-data for the SIGNON event.
    match event_type.as_str() {
        "SIGNON" => {
            let sub_signon_pattern = Regex::new(r"([-\*\+0-9]+) ([0-9a-fA-F]+) (.+?) ([0-9]+) ([0-9]+) ([0-9]+) (.+?)$")
                .expect("Invalid SIGNON regex pattern");

            if let Some(captures) = sub_signon_pattern.captures(sub_data) {
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
            } else {
                warn!("Failed to extract sub-data from SIGNON");
            }
        },

        // Search for the event sub-data for the ZONECHANGE event.
        "ZONECHANGE" => {
            let sub_zone_pattern = Regex::new(r"([0-9]+) ([0-9]+) ([0-9]+) ([0-9]+)(.*)$")
                .expect("Invalid ZONECHANGE regex pattern");

            if let Some(captures) = sub_zone_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(from_zone) = captures.get(2) {
                    result.push(("From zone".to_string(), from_zone.as_str().to_string()));
                }
                if let Some(to_zone) = captures.get(3) {
                    result.push(("To zone".to_string(), to_zone.as_str().to_string())); // FIXED: Label
                }
                if let Some(zone_output) = captures.get(4) {
                    result.push(("Zone output".to_string(), zone_output.as_str().to_string()));
                }
            } else {
                warn!("Failed to extract sub-data from ZONECHANGE");
            }
        },

        // Search for the event sub-data for the TRIP event.
        "TRIP" => {
            let sub_trip_pattern = Regex::new(r"([0-9]+) ([0-9]+) ([0-9]+) ([0-9]+) ([0-9]+) ([0-9]+)(.*)$")
                .expect("Invalid TRIP regex pattern");

            if let Some(captures) = sub_trip_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(time_fwd) = captures.get(2) {
                    result.push(("Time fwd".to_string(), time_fwd.as_str().to_string()));
                }
                if let Some(time_rev) = captures.get(3) {
                    result.push(("Time rev".to_string(), time_rev.as_str().to_string()));
                }
                if let Some(time_idle) = captures.get(4) {
                    result.push(("Time idle".to_string(), time_idle.as_str().to_string()));
                }
                if let Some(max_idle) = captures.get(5) {
                    result.push(("Max idle".to_string(), max_idle.as_str().to_string()));
                }
                if let Some(time_on_seat) = captures.get(6) {
                    result.push(("Time on seat".to_string(), time_on_seat.as_str().to_string()));
                }
            } else {
                warn!("Failed to extract sub-data from TRIP");
            }
        },
        _ => {
            // Handle other event types or add a default case
            result.push(("Raw data".to_string(), sub_data.to_string()));
        }
    }

    result
}

// Implement Default for way to create a 'blank' instance.
impl Default for Scraper {
    fn default() -> Self {
        Self::new()
    }
}
