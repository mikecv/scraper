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
    pub line_number: usize,
    pub content: String,
    pub timestamp: Option<String>,
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
}

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
        }
    }

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
        
        // Clear previous results
        self.processed_data.clear();
        
        match self.read_and_process_file(path) {
            Ok(count) => {
                self.processing_status = format!("Successfully processed {} entries", count);
                info!("File processing completed: {} entries found", count);
            }
            Err(e) => {
                self.processing_status = format!("Error processing file: {}", e);
                info!("File processing error: {}", e);
            }
        }
    }

    // Main file processing logic.
    fn read_and_process_file(&mut self, path: &PathBuf) -> Result<usize, Box<dyn std::error::Error>> {
        // Open the file.
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        // TODO: Define your regex patterns here
        // Example patterns - adjust these for your specific log format
        let start_pattern = Regex::new(r"START_MARKER|ERROR|BEGIN")?; // Pattern to start processing
        let end_pattern = Regex::new(r"END_MARKER|STOP|COMPLETE")?;   // Pattern to stop processing
        let data_pattern = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})")?; // Example: timestamp extraction
        
        let mut processing = false;
        let mut line_number = 0;
        let mut processed_count = 0;
        
        // Process file line by line
        for line_result in reader.lines() {
            line_number += 1;
            let line = line_result?;
            
            // Check if we should start processing.
            if !processing && start_pattern.is_match(&line) {
                processing = true;
                info!("Started processing at line {}: {}", line_number, line.trim());
                continue;
            }
            
            // Check if we should stop processing.
            if processing && end_pattern.is_match(&line) {
                processing = false;
                info!("Stopped processing at line {}: {}", line_number, line.trim());
                // Optionally include the end line in results:
                // self.process_line(&line, line_number, &data_pattern);
                // processed_count += 1;
                break; // Remove this 'break' if you want to continue looking for more start patterns
            }
            
            // Process lines between start and end patterns.
            if processing {
                if self.process_line(&line, line_number, &data_pattern) {
                    processed_count += 1;
                }
            }
        }
        
        // Handle case where we reach end of file while still processing.
        if processing {
            info!("Reached end of file while processing (no end pattern found)");
        }
        
        Ok(processed_count)
    }

    // Process individual line and extract data.
    fn process_line(&mut self, line: &str, line_number: usize, data_pattern: &Regex) -> bool {
        // Skip empty lines.
        if line.trim().is_empty() {
            return false;
        }
        
        // TODO: Add your specific line processing logic here.
        // This is where you'd extract specific data from each line.
        
        // Example: Extract timestamp if present.
        let timestamp = data_pattern.captures(line)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string());
        
        // Create processed entry.
        let entry = ProcessedEntry {
            line_number,
            content: line.to_string(),
            timestamp,
        };
        
        self.processed_data.push(entry);
        true
    }

    // Get processing status for display.
    pub fn get_processing_status(&self) -> &str {
        &self.processing_status
    }
    
    // Get processed data for display.
    pub fn get_processed_data(&self) -> &[ProcessedEntry] {
        &self.processed_data
    }
    
    // Get count of processed entries.
    pub fn get_processed_count(&self) -> usize {
        self.processed_data.len()
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
}

// Implement Default for way to create a 'blank' instance.
impl Default for Scraper {
    fn default() -> Self {
        Self::new()
    }
}
