// Scraper structure and methods.

use log::info;

use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::mpsc;
use tinyfiledialogs::open_file_dialog;

use crate::egui;
use crate::settings::Settings;
use crate::SETTINGS;

#[allow(dead_code)]

#[derive(Debug)]
pub enum FileDialogMessage {
    FileSelected(PathBuf),
    DialogClosed,
}

// Structure to hold processed data.
#[derive(Debug, Clone)]
pub struct ProcessedEntry {
    pub _line_number: usize,
    pub _content: String,
    pub _timestamp: Option<String>,
}

// Scraper struct and methods.
#[derive(Debug)]
pub struct Scraper {
    pub settings: Settings,
    pub selected_file: Option<PathBuf>,
    pub file_dialog_open: bool,
    pub file_receiver: Option<mpsc::Receiver<FileDialogMessage>>,
    pub processed_data: Vec<ProcessedEntry>,
    pub processing_status: String,
    pub controller_id: String,
    pub controller_fw: String,
}

// Implement Scraper class.
impl Scraper {
    // A function to create a new Scraper instance.
    pub fn new() -> Self {
        info!("Creating new instance of Scraper.");

        // Lock the global SETTINGS to obtain access to the Settings object.
        let settings = SETTINGS.lock().unwrap().clone();

        Self {
            settings: settings,
            selected_file: None,
            file_dialog_open: false,
            file_receiver: None,
            processed_data: Vec::new(),
            processing_status: "No file selected.".to_string(),
            controller_id: "".to_string(),
            controller_fw: "".to_string(),
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
        self.processed_data.clear();
        self.processing_status = "Loading new file...".to_string();
        self.controller_id = "".to_string();
        self.controller_fw = "".to_string(); // Clear firmware field
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

        // Clear previous results.
        self.processed_data.clear();
        
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
        let fw_pattern = Regex::new(r"([0-9]{1,2}/[0-9]{2}/[0-9]{4}) ([0-9]{1,2}:[0-9]{2}:[0-9]{2}(?:\.\d{3})?(?: [AP]M)?) .*?EVENT ([0-9]+) ([0-9]+) (.+)/(.+)/(.+)/([-0-9]+)/([0-9]+) SWSTART (.+) ([.0-9]+.+) v:(.+)$")?;        let mut _found_fw = false;


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

    // Get processed data for display.
    pub fn _get_processed_data(&self) -> &[ProcessedEntry] {
        &self.processed_data
    }

    // Get count of processed entries.
    pub fn _get_processed_count(&self) -> usize {
        self.processed_data.len()
    }
}

// Implement Default for way to create a 'blank' instance.
impl Default for Scraper {
    fn default() -> Self {
        Self::new()
    }
}
