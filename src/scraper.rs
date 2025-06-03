// Scraper structure and methods.

use log::info;

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
    pub new_trip: bool,
    pub event_type: String,
    pub ev_detail: String,
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
            Some((&["*.bak", "*.csv"], "Log files (bak, csv)")),
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
        self.selected_file = None;
        // self.processed_data.clear();
        self.processing_status = "Loading new file...".to_string();
        self.controller_id = "".to_string();
        self.controller_fw = "".to_string();
        // Clear any ongoing file dialog state.
        self.file_dialog_open = false;
        self.file_receiver = None;
    }

    // Method to load file from a given path.
    // Required for drag and drop files.
    pub fn load_file_from_path(&mut self, path: &std::path::Path) {
        info!("Loading file from path: {:?}", path);
        
        let path_buf = path.to_path_buf();
        self.selected_file = Some(path_buf.clone());
        self.process_file(&path_buf);
    }

    // Method to scrape the selected file.
    fn process_file(&mut self, path: &PathBuf) {
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

        // Clear fields at start of processing to ensure clean state
        self.controller_id.clear();
        self.controller_fw.clear();

        // Open the file.
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        info!("Searching file for controller serial number.");
    
        // Get the serial number of the controller.
        let sn_pattern = Regex::new(r"([0-9]{1,2}/[0-9]{2}/[0-9]{4}) ([0-9]{1,2}:[0-9]{2}:[0-9]{2}(?:\.\d{3})?(?: [AP]M)?)[:, ]UNIT ([0-9]+)$")?;
        let mut _found_sn = false;
        
        // Process file line by line,
        for line_result in reader.lines() {
            let line = line_result?;
            
            // Check if we should stop processing.
            if let Some(captures) = sn_pattern.captures(&line) {
                _found_sn = true;
                // Group 3 contains the serialnumber.
                let sn_str = captures.get(3).unwrap().as_str();
                self.controller_id = sn_str.to_string();
                info!("Found controller s/n: {:0>6}", sn_str); 
            }
            if _found_sn == true {
                break
            }
        }
        if _found_sn == false {
            info!("Failed to find controller serial number."); 
        }

        // Initialise file reader again.
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        info!("Searching file for controller firmware version.");
       
        // Get the controller firmware version.
        let fw_pattern = Regex::new(r"([0-9]{1,2}/[0-9]{2}/[0-9]{4}) ([0-9]{1,2}:[0-9]{2}:[0-9]{2}\.[0-9]{3}) EVENT ([0-9]+) ([0-9]+) ([-0-9]+)/([0-9]+)/([0-9]+)/([0-9]+)/([0-9]+) ([A-Z_]+) ([0-9]+) ([A-Z]+) (.+)$")?;
        let mut _found_fw = false;


        // Process file line by line,
        for line_result in reader.lines() {
            let line = line_result?;
            
            // Check if we should stop processing.
            if let Some(captures) = fw_pattern.captures(&line) {
                _found_fw = true;
                // Group 11 contains the firmware version.
                let fw_str = captures.get(11).unwrap().as_str();
                self.controller_fw = fw_str.to_string();
                info!("Found controller firmware: {:?}", fw_str); 
            }
            if _found_fw == true {
                break
            }
        }
        if _found_fw == false {
                info!("Failed to find controller firmware version."); 
        }

        // Initialise file reader again.
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        info!("Searching file for controller events.");

        // Get the controller events.
        let ev_pattern = Regex::new(r"([0-9]{1,2}/[0-9]{2}/[0-9]{4}) ([0-9]{1,2}:[0-9]{2}:[0-9]{2}\.[0-9]{3}) EVENT ([0-9]+) ([0-9]+) ([-0-9]+)/([0-9]+)/([0-9]+)/([0-9]+)/([0-9]+) ([A-Z_]+) (.+)$")?;
        let mut _event_count = 0;

        // Process file line by line.
        for line_result in reader.lines() {
            let line = line_result?;
            
            // Check for event pattern
            if let Some(captures) = ev_pattern.captures(&line) {
                _event_count += 1;
                
                // Extract some key fields for logging.
                let date = captures.get(1).unwrap().as_str();
                let time = captures.get(2).unwrap().as_str();
                let event_type = captures.get(10).unwrap().as_str();
                let event_detail = captures.get(11).unwrap().as_str();
                
                // Create and populate the struct correctly
                let ev_data = ScrapedData {
                    date_time: format!("{} {}", date, time),
                    new_trip: event_type == "SIGNON",
                    event_type: event_type.to_string(),
                    ev_detail: event_detail.to_string(),
                };

                // Push the struct onto the vector.
                self.scrapings.push(ev_data);
            }
        }

        for item in self.scrapings.iter() {
            info!("Date: {:?} New trip: {:?} Event: {:?} Detail: {:?}", item.date_time, item.new_trip, item.event_type, item.ev_detail);
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

// Implement Default for way to create a 'blank' instance.
impl Default for Scraper {
    fn default() -> Self {
        Self::new()
    }
}
