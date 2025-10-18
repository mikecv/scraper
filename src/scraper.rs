// Scraper structure and methods.

use log::info;
use log::warn;

use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Instant, Duration};

use crate::egui;

// Use conditional includes for linux and Windows,
// as tinyfiledialogs doesn't readily compile and
// build for Windows.

#[cfg(target_os = "windows")]
use rfd::FileDialog;
#[cfg(target_os = "linux")]
use tinyfiledialogs::open_file_dialog;

#[allow(dead_code)]

#[derive(Debug)]
pub enum FileDialogMessage {
    FileSelected(PathBuf),
    DialogClosed,
}

// GPS location (lat, lon)
#[derive(Debug)]
pub struct GpsLocation {
    pub lat: f64,
    pub lon: f64,
}

// Data that is scraped.
#[derive(Debug)]
pub struct ScrapedData {
    pub date_time: String,
    pub unix_time: u64,
    pub on_trip: bool,
    pub trip_num: String,
    pub event_type: String,
    pub ev_detail: Vec<(String, String)>,
    pub ev_supported: bool,
    pub gps_rssi: u32,
    pub gps_speed: u32,
    pub gps_locn: GpsLocation,
}

// Scraper struct and methods.
#[derive(Debug)]
pub struct Scraper {
    pub selected_file: Option<PathBuf>,
    pub file_dialog_open: bool,
    pub file_receiver: Option<mpsc::Receiver<FileDialogMessage>>,
    pub processing_status: String,
    pub processing_duration: Duration,
    pub controller_id: String,
    pub controller_fw: String,
    pub scrapings: Vec<ScrapedData>,
}

// Implement Sraper class.
impl Scraper {
    // A function to create a new Scraper instance.
    pub fn new() -> Self {
        info!("Creating new instance of Scraper.");

        Self {
            selected_file: None,
            file_dialog_open: false,
            file_receiver: None,
            processing_status: "No file selected.".to_string(),
            processing_duration: Duration::new(0, 0),
            controller_id: "".to_string(),
            controller_fw: "".to_string(),
            scrapings: Vec::new(),
        }
    }
}

impl Scraper {
    // Load log file for processing.
    // The load file triggers a clearing of any previous selection id.
    pub fn load_file(&mut self, ctx: &egui::Context, selected_id: &mut Option<String>) {
        info!("Browsing for file to open.");

        // Prevent multiple dialogs.
        if self.file_dialog_open {
            return;
        }

        // Before we start we can delete any previously selected trip.
        // Reset selected_id before loading new file.
        *selected_id = None;
 
        self.file_dialog_open = true;

        // Alternate file dialoges used for
        // linux and Windows builds, as tinyfiledialogs
        // doesn't readily build for Windows because of
        // the available toolchain.

        let file_path = {
            #[cfg(target_os = "windows")]
            {
                // For windows use FileDialog.
                FileDialog::new()
                    // .add_filter("text", &["txt"])
                    .add_filter("Log files", &["log", "bak", "csv", "txt"])
                    .add_filter("All files", &["*"])
                    .pick_file()
                    .map(|path| path.to_string_lossy().to_string())
            }
            #[cfg(target_os = "linux")]
            {
                // Use tinyfiledialogs synchronous dialog.
                open_file_dialog(
                    "Select log file",
                    "",
                    Some((&["*.log", "*.bak", "*.csv", "*.txt"], "Log files (log, bak, csv, txt)")),
                )
            }
        };

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

        // Initialise timer for proocessing.
        let processing_start = Instant::now();

        // First initialize scraped data.
        self.reinitialize_data();

        info!("Processing file: {:?}", path);

        match self.read_and_process_file(path) {
            Ok(_sn) => {
                self.processing_duration = processing_start.elapsed();
                self.processing_status = format!("Successfully completed processing in {:?}.", self.processing_duration);
                info!("Successfully completed processing in {:?}", self.processing_duration);
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
                // Group 3 contains the serial number.
                let sn_str = captures.get(3).unwrap().as_str();
                self.controller_id = sn_str.to_string();
                info!("Found controller s/n: {:0>6}", sn_str); 
            }
            if found_sn {
                // Have found one instance of controller number.
                // Don't need to look any further.
                break;
            }
        }
        if found_sn == false {
            self.controller_id = "Not defined.".to_string();
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
                // Group 11 contains the firmware versin.
                let fw_str = captures.get(11).unwrap().as_str();
                self.controller_fw = fw_str.to_string();
                info!("Found controller firmware: {:?}", fw_str); 
            }
            if found_fw {
                // Have found one instance of firmware version.
                // Don't need to look any further as for now we are only
                // looking for the first instance.
                break;
            }
        }
        if found_fw == false {
            self.controller_fw = "Not defined.".to_string();
            info!("Failed to find controller firmware version."); 
        }

        // Initialise file reader again.
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        info!("Searching file for controller events.");

        // Get the controller events.
        let ev_pattern = Regex::new(r"([0-9]{1,2}/[0-9]{2}/[0-9]{4}) ([0-9]{1,2}:[0-9]{2}:[0-9]{2})(?:\.[0-9]{3})? EVENT ([0-9]+) ([0-9]+) ([-0-9]+)/([0-9]+)/([0-9]+)/([0-9]+)/([0-9]+) ([A-Z_]+) (.+)$")?;

        // Trip number for SIGNON event so that it can
        // be copied to the other events in the trip.
        let mut trip_num_id = "".to_string();

        // Process file line by line.
        for line_result in reader.lines() {
            let line = line_result?;
            
            // Check for event pattern.
            if let Some(captures) = ev_pattern.captures(&line) {
                
                // Extract key fields for logging.
                let date = captures.get(1).unwrap().as_str();
                let time = captures.get(2).unwrap().as_str();
                let unix_time = captures.get(4).unwrap().as_str();
                let event_type = captures.get(10).unwrap().as_str();
                let event_detail = captures.get(11).unwrap().as_str();
                let mut on_trip = true;
                let mut ev_supported = true;
                let ev_key_vals = ungroup_event_data(event_type.to_string(), event_detail, &mut on_trip, &mut ev_supported);
                let trip_id = captures.get(3).unwrap().as_str();

                // Get the gps location from the event data.
                // While gps location is included in the event string,
                // it's not part of the event detail.
                let gps_latitude = captures.get(5)
                    .expect("Latitude capture group not found.")
                    .as_str()
                    .parse::<f64>()
                    .expect("Failed to parse latitude as f64");
                let gps_longitude = captures.get(6)
                    .expect("Longitude capture group not found.")
                    .as_str()
                    .parse::<f64>()
                    .expect("Failed to parse longitude as f64");
                let gps_locn = GpsLocation {
                    lat: gps_latitude / 10_000_000.0,
                    lon: gps_longitude / 10_000_000.0,
                };

                // Get the gps RSSI from the event data.
                let gps_rssi = captures.get(8)
                    .expect("GPS RSSI capture group not found.")
                    .as_str()
                    .parse::<u32>()
                    .expect("Failed to parse gps rssi as u32");

                // Get the gps speed from the event data.
                let gps_speed = captures.get(9)
                    .expect("GPS speed capture group not found.")
                    .as_str()
                    .parse::<u32>()
                    .expect("Failed to parse gps speed as u32");

                // Keep track of on-trip state.
                // SIGNON sets TRIP clears.
                if event_type == "SIGNON" { 
                    // Save the trip number to apply to other events.
                    trip_num_id = trip_id.to_string();   
                }   

                // Create and populate the struct.
                // Initialise events to be supported; change later if not.
                let ev_data = ScrapedData {
                    date_time: format!("{} {}", date, time),
                    unix_time: unix_time.parse().expect("Invalid Unix time string"),
                    on_trip: on_trip,
                    trip_num: trip_num_id.clone(),
                    event_type: event_type.to_string(),
                    ev_detail: ev_key_vals,
                    ev_supported: ev_supported,
                    gps_locn: gps_locn,
                    gps_rssi: gps_rssi,
                    gps_speed: gps_speed,
                };

                // Push the struct onto the vector.
                self.scrapings.push(ev_data);

                // Clear on trip flag after TRIP event.
                // This makes TRIP still part of the trip.
                if event_type == "TRIP" {
                   // Clear the saved trip number as
                   // following events are out of trip.
                   trip_num_id = "".to_string();
                }
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
fn ungroup_event_data(event_type: String, sub_data: &str, on_trip: &mut bool, ev_supported: &mut bool) -> Vec<(String, String)> {
    // Initialise result vector.
    let mut result = Vec::new();

    // Search for the event sub-data for the SIGNON event.
    // CHECKED //
    match event_type.as_str() {
        "SIGNON" => {
            let sub_signon_pattern = Regex::new(r"([-\*\+0-9]+) ([0-9a-fA-F]+) (.+?) ([0-9]+) ([0-9]+) ([0-9]+) v:(.+?)$")
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
                if let Some(battery) = captures.get(7) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                    warn!("Failed to extract sub-data from SIGNON");
            }
        },

        // Search for the event sub-data for the CHECKLIST event.
        "CHECKLIST" => {
            let sub_checklist_pattern = Regex::new(r"([0-9]+) (OK|CANCEL|NOFILE) ([0-9]+) ([0-9]+) ([0-9]+) ([\-a-zA-Z]+) v:(.+?)$")
                .expect("Invalid CHECKLIST regex pattern");

            if let Some(captures) = sub_checklist_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(chk_result) = captures.get(2) {
                    result.push(("Result".to_string(), chk_result.as_str().to_string()));
                }
                if let Some(failed_q) = captures.get(3) {
                    result.push(("Failed questions".to_string(), failed_q.as_str().to_string()));
                }
                if let Some(chklist_dur) = captures.get(4) {
                    result.push(("Checklist duration".to_string(), chklist_dur.as_str().to_string()));
                }
                if let Some(chklist_ver) = captures.get(5) {
                    result.push(("Checklist version".to_string(), chklist_ver.as_str().to_string()));
                }
                if let Some(chklist_type) = captures.get(6) {
                    result.push(("Checklist type".to_string(), chklist_type.as_str().to_string()));
                }
                if let Some(battery) = captures.get(7) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from CHECKLIST");
            }
        },

        // Search for the event sub-data for the CLFAIL event.
        "CLFAIL" => {
            let sub_clfail_pattern = Regex::new(r"([0-9]+) ([0-9]+) v:(.+?)$")
                .expect("Invalid CLFAIL regex pattern");

            if let Some(captures) = sub_clfail_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(chk_fail_q) = captures.get(2) {
                    result.push(("Failded question".to_string(), chk_fail_q.as_str().to_string()));
                }
                if let Some(battery) = captures.get(3) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from CLFAIL");
            }
        },

        // Search for the event sub-data for the CONFIG event.
        "CONFIG" => {
                info!("CONFIG event found, no sub-data applicable.");
        },

        // Search for the event sub-data for the CRITICALOUTPUTSET event.
        "CRITICALOUTPUTSET" => {
            let sub_co_pattern = Regex::new(r"([0-9]+) ([0-9]+) v:(.+?)$")
                .expect("Invalid CRITICALOUTPUTSET regex pattern");

            if let Some(captures) = sub_co_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(speed) = captures.get(2) {
                    result.push(("Speed".to_string(), speed.as_str().to_string()));
                }
                if let Some(battery) = captures.get(3) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from CRITICALOUTPUTSET");
            }
        },

        // Search for the event sub-data for the DEBUG event.
        "DEBUG" => {
            let sub_debug_pattern = Regex::new(r"(.+) v:(.+?)$")
                .expect("Invalid DEBUG regex pattern");

            if let Some(captures) = sub_debug_pattern.captures(sub_data) {
                if let Some(error) = captures.get(1) {
                    result.push(("Debug error".to_string(), error.as_str().to_string()));
                }
                if let Some(battery) = captures.get(2) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from DEBUG");
            }
        },

        // Search for the event sub-data for the ENGINEOVERSPEED event.
        "ENGINEOVERSPEED" => {
            let sub_engine_overspeed_pattern = Regex::new(r"([0-9]+) ([0-9]+) ([0-9]+) v:(.+?)$")
                .expect("Invalid ENGINEOVERSPEED regex pattern");

            if let Some(captures) = sub_engine_overspeed_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(duration) = captures.get(2) {
                    result.push(("Duration".to_string(), duration.as_str().to_string()));
                }
                if let Some(max_rpm) = captures.get(3) {
                    result.push(("Max RPM".to_string(), max_rpm.as_str().to_string()));
                }
                if let Some(battery) = captures.get(4) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from ENGINEOVERSPEED");
            }
        },

         // Search for the event sub-data for the HARDWARE event.
        "HARDWARE" => {
            let sub_hardware_pattern = Regex::new(r"(.*) v:(.+?)$")
                .expect("Invalid HARDWARE regex pattern");

            if let Some(captures) = sub_hardware_pattern.captures(sub_data) {
                if let Some(eq_fail) = captures.get(1) {
                    result.push(("Equipment fault".to_string(), eq_fail.as_str().to_string()));
                }
                if let Some(battery) = captures.get(2) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from ENGINETEMP");
            }
        },

         // Search for the event sub-data for the ENGINETEMP event.
        "ENGINETEMP" => {
            let sub_enginetemp_pattern = Regex::new(r"([0-9]+) ([0-9]+)(.*) v:(.+?)$")
                .expect("Invalid ENGINETEMP regex pattern");

            if let Some(captures) = sub_enginetemp_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(duration) = captures.get(2) {
                    result.push(("Duration".to_string(), duration.as_str().to_string()));
                }
                if let Some(battery) = captures.get(4) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from ENGINETEMP");
            }
        },

        // Search for the event sub-data for the IMPACT event.
        "IMPACT" => {
            let sub_impact_pattern = Regex::new(r"([0-9]+) ([0-9]+) ([0-9]+) ([0-9]+) ([0-9]+) ([0-9]+) ([0-9]+) ([\-a-zA-Z]+) v:(.+?)$")
                .expect("Invalid IMPACT regex pattern");

            if let Some(captures) = sub_impact_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(fwd_g) = captures.get(2) {
                    result.push(("Forward g".to_string(), fwd_g.as_str().to_string()));
                }
                if let Some(rev_g) = captures.get(3) {
                    result.push(("Reverse g".to_string(), rev_g.as_str().to_string()));
                }
                if let Some(left_g) = captures.get(4) {
                    result.push(("Left g".to_string(), left_g.as_str().to_string()));
                }
                if let Some(right_g) = captures.get(5) {
                    result.push(("Right g".to_string(), right_g.as_str().to_string()));
                }
                if let Some(max_g1) = captures.get(6) {
                    result.push(("Max G1".to_string(), max_g1.as_str().to_string()));
                }
                if let Some(max_g2) = captures.get(7) {
                    result.push(("Max G2".to_string(), max_g2.as_str().to_string()));
                }
                if let Some(severity) = captures.get(8) {
                    result.push(("Severity".to_string(), severity.as_str().to_string()));
                }
                if let Some(battery) = captures.get(9) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from IMPACT");
            }
        },

        // Search for the event sub-data for the INPUT event.
        "INPUT" => {
            let sub_input_pattern = Regex::new(r"([0-9]+) ([0-9]+) ([0-9]+) v:(.+?)$")
                .expect("Invalid INPUT regex pattern");

            if let Some(captures) = sub_input_pattern.captures(sub_data) {
                if let Some(input_num) = captures.get(1) {
                    result.push(("Input number".to_string(), input_num.as_str().to_string()));
                }
                if let Some(input_state) = captures.get(2) {
                    result.push(("Active state".to_string(), input_state.as_str().to_string()));
                }
                if let Some(active_time) = captures.get(3) {
                    result.push(("Time active".to_string(), active_time.as_str().to_string()));
                }
                if let Some(battery) = captures.get(4) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from INPUT");
            }
        },

         // Search for the event sub-data for the LOWCOOLANT event.
        "LOWCOOLANT" => {
            let sub_lowcoolant_pattern = Regex::new(r"([0-9]+) ([0-9]+)(.*) v:(.+?)$")
                .expect("Invalid LOWCOOLANT regex pattern");

            if let Some(captures) = sub_lowcoolant_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(duration) = captures.get(2) {
                    result.push(("Duration".to_string(), duration.as_str().to_string()));
                }
                if let Some(battery) = captures.get(3) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from LOWCOOLANT");
            }
        },

         // Search for the event sub-data for the OFFSEAT event.
        "OFFSEAT" => {
            let sub_off_seat_pattern = Regex::new(r"([0-9]+) ([0-9]+)(.*) v:(.+?)$")
                .expect("Invalid OFFSEAT regex pattern");

            if let Some(captures) = sub_off_seat_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(duration) = captures.get(2) {
                    result.push(("Duration".to_string(), duration.as_str().to_string()));
                }
                if let Some(battery) = captures.get(3) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from OFFSEAT");
            }
        },

         // Search for the event sub-data for the OILPRESSURE event.
        "OILPRESSURE" => {
            let sub_oilpressure_pattern = Regex::new(r"([0-9]+) ([0-9]+)(.*) v:(.+?)$")
                .expect("Invalid OILPRESSURE regex pattern");

            if let Some(captures) = sub_oilpressure_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(duration) = captures.get(2) {
                    result.push(("Duration".to_string(), duration.as_str().to_string()));
                }
                if let Some(battery) = captures.get(3) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from OILPRESSURE");
            }
        },

        // Search for the event sub-data for the OOS PM event.
        "OOS PM" => {
            let sub_oospm_pattern = Regex::new(r" v:(.+?)$")
                .expect("Invalid OS PMOS regex pattern");

            if let Some(captures) = sub_oospm_pattern.captures(sub_data) {
                if let Some(battery) = captures.get(1) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from OOS PMS");
            }
        },

        // Search for the event sub-data for the OOS UPM event.
        "OOS UPM" => {
            let sub_oosupm_pattern = Regex::new(r"[0-9]+) ([0-9]+) v:(.+?)$")
                .expect("Invalid OOS UPM regex pattern");

            if let Some(captures) = sub_oosupm_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(reason) = captures.get(2) {
                    result.push(("Reason".to_string(), reason.as_str().to_string()));
                }
                if let Some(battery) = captures.get(3) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from OOS UPM");
            }
        },

        // Search for the event sub-data for the OVERLOAD event.
        "OVERLOAD" => {
            let sub_overload_pattern = Regex::new(r"([0-9]+) ([0-9]+)(.*) v:(.+?)$")
                .expect("Invalid OVERLOAD regex pattern");

            if let Some(captures) = sub_overload_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(duration) = captures.get(2) {
                    result.push(("Duration".to_string(), duration.as_str().to_string()));
                }
                if let Some(battery) = captures.get(3) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from OVERLOAD");
            }
        },

        // Search for the event sub-data for the OVERSPEED event.
        "OVERSPEED" => {
            let sub_overspeed_pattern = Regex::new(r"([0-9]+) ([0-9]+) v:(.+?)$")
                .expect("Invalid OVERSPEED regex pattern");

            if let Some(captures) = sub_overspeed_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(duration) = captures.get(2) {
                    result.push(("Duration".to_string(), duration.as_str().to_string()));
                }
                if let Some(battery) = captures.get(3) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from OVERSPEED");
            }
        },

        // Search for the event sub-data for the CONFIG event.
        "POWERDOWN" => {
                info!("POWERDOWN event found, no sub-data applicable.");
        },

        // Search for the event sub-data for the REPORT event.
        // CHECKED //
        "REPORT" => {
            let sub_report_pattern = Regex::new(r"(\*|[0-9]+) ([0-9]+) ([0-9]+) v:(.+?)$")
                .expect("Invalid REPORT regex pattern");

            if let Some(captures) = sub_report_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(speed) = captures.get(2) {
                    result.push(("Speed".to_string(), speed.as_str().to_string()));
                }
                if let Some(dirn) = captures.get(3) {
                    result.push(("Direction".to_string(), dirn.as_str().to_string()));
                }
                if let Some(battery) = captures.get(4) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from REPORT");
            }
        },
 
        // Search for the event sub-data for the SERVICE event.
        "SERVICE" => {
            let sub_service_pattern = Regex::new(r" v:(.+?)$")
                .expect("Invalid SERVICE regex pattern");

            if let Some(captures) = sub_service_pattern.captures(sub_data) {
                if let Some(battery) = captures.get(1) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from SERVICE");
            }
        },


        // Search for the event sub-data for the SWSTART event.
        // NOTE that the SWSTART event occurs outside of trips.
        "SWSTART" => {
            let sub_swstart_pattern = Regex::new(r"([.0-9]+ .*) v:(.+?)$")
                .expect("Invalid SWSTART regex pattern");

            if let Some(captures) = sub_swstart_pattern.captures(sub_data) {
                if let Some(firmware) = captures.get(1) {
                    result.push(("Firmware".to_string(), firmware.as_str().to_string()));
                }
                if let Some(battery) = captures.get(2) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }

                // The SWSTART event only occurs out of trip.
                // Setting SWSSTART on_trip flag to false.
                *on_trip = false;

            } else {
                warn!("Failed to extract sub-data from SWSTART");
            }
        },

        // Search for the event sub-data for the UNBUCKLED event.
        "UNBUCKLED" => {
            let sub_unbuckled_pattern = Regex::new(r"([0-9]+) ([0-9]+) ([DP]) v:(.+?)$")
                .expect("Invalid UNBUCKLED regex pattern");

            if let Some(captures) = sub_unbuckled_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(duration) = captures.get(2) {
                    result.push(("Duration".to_string(), duration.as_str().to_string()));
                }
                if let Some(seat_owner) = captures.get(3) {
                    result.push(("Seat owner".to_string(), seat_owner.as_str().to_string()));
                }
                if let Some(battery) = captures.get(4) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from UNBUCKLED");
            }
        },

        // Search for the event sub-data for the XSIDLE event.
        "XSIDLE" => {
            let sub_xsidle_pattern = Regex::new(r"([0-9]+) ([0-9]+) ([0-9]+) v:(.+?)$")
                .expect("Invalid XSIDLE regex pattern");

            if let Some(captures) = sub_xsidle_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(max_idle) = captures.get(2) {
                    result.push(("Max idle".to_string(), max_idle.as_str().to_string()));
                }
                if let Some(xsidle_reason) = captures.get(3) {
                    result.push(("Excess idle reason".to_string(), xsidle_reason.as_str().to_string()));
                }
                if let Some(battery) = captures.get(4) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                    else {
                        result.push(("Battery voltage".to_string(), "?".to_string()));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from XSIDLE");
            }
        },

        // Search for the event sub-data for the XSIDLESTART event.
        "XSIDLESTART" => {
            let sub_xsidlest_pattern = Regex::new(r"([0-9]+) v:(.+?)$")
                .expect("Invalid XSIDLESTART regex pattern");

            if let Some(captures) = sub_xsidlest_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(battery) = captures.get(2) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                    else {
                        result.push(("Battery voltage".to_string(), "?".to_string()));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from XSIDLESTART");
            }
        },

        // Search for the event sub-data for the ZONE_OK event.
        "ZONE_OK" => {
            let zone_ok_pattern = Regex::new(r"([0-9]+) ([0-9]+) (.*) v:(.+?)$")
                .expect("Invalid ZONE_OK regex pattern");

            if let Some(captures) = zone_ok_pattern.captures(sub_data) {
                if let Some(zones_loaded) = captures.get(1) {
                    result.push(("Zones loaded".to_string(), zones_loaded.as_str().to_string()));
                }
                if let Some(max_zones) = captures.get(2) {
                    result.push(("Max zones".to_string(), max_zones.as_str().to_string()));
                }
                if let Some(firmware) = captures.get(3) {
                    result.push(("GPS firmware version".to_string(), firmware.as_str().to_string()));
                }
                if let Some(battery) = captures.get(4) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from ZONE_OK");
            }
        },

        // Search for the event sub-data for the ZONECHANGE event.
        "ZONECHANGE" => {
            let sub_zone_pattern = Regex::new(r"([0-9]+) ([0-9]+) ([0-9]+) ([0-9]+) v:(.+?)$")
                .expect("Invalid ZONECHANGE regex pattern");

            if let Some(captures) = sub_zone_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(from_zone) = captures.get(2) {
                    result.push(("From zone".to_string(), from_zone.as_str().to_string()));
                }
                if let Some(to_zone) = captures.get(3) {
                    result.push(("To zone".to_string(), to_zone.as_str().to_string()));
                }
                if let Some(zone_output) = captures.get(4) {
                    result.push(("Zone output".to_string(), zone_output.as_str().to_string()));
                }
                if let Some(battery) = captures.get(5) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from ZONECHANGE");
            }
        },

        // Search for the event sub-data for the ZONEOVERSPEED event.
        "ZONEOVERSPEED" => {
            let sub_zone_overspeed_pattern = Regex::new(r"([0-9]+) ([0-9]+) ([0-9]+) ([0-9]+) v:(.+?)$")
                .expect("Invalid ZONEOVERSPEED regex pattern");

            if let Some(captures) = sub_zone_overspeed_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(duration) = captures.get(2) {
                    result.push(("Duration".to_string(), duration.as_str().to_string()));
                }
                if let Some(max_speed) = captures.get(3) {
                    result.push(("Maximum speed".to_string(), max_speed.as_str().to_string()));
                }
                if let Some(zone_output) = captures.get(4) {
                    result.push(("Zone output".to_string(), zone_output.as_str().to_string()));
                }
                if let Some(battery) = captures.get(5) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from ZONEOVERSPEED");
            }
        },

        // Search for the event sub-data for the ZONETRANSITION event.
        "ZONETRANSITION" => {
            let sub_trans_pattern = Regex::new(r"([0-9]+) ([0-9]+) ([0-9]+) ([0-9]+) (ENTRY|EXIT) v:(.+?)$")
                .expect("Invalid ZONETRANSITION regex pattern");

            if let Some(captures) = sub_trans_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(from_zone) = captures.get(2) {
                    result.push(("From zone".to_string(), from_zone.as_str().to_string()));
                }
                if let Some(to_zone) = captures.get(3) {
                    result.push(("To zone".to_string(), to_zone.as_str().to_string()));
                }
                if let Some(to_zone_output) = captures.get(4) {
                    result.push(("Zone output".to_string(), to_zone_output.as_str().to_string()));
                }
                if let Some(transition) = captures.get(5) {
                    result.push(("Transition".to_string(), transition.as_str().to_string()));
                }
                if let Some(battery) = captures.get(6) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from ZONETRANSITION");
            }
        },

        // Search for the event sub-data for the TRIP event.
        "TRIP" => {
            let sub_trip_pattern = Regex::new(r"([0-9]+) ([0-9]+) ([0-9]+) ([0-9]+) ([0-9]+) ([0-9]+)(.*) v:(.+?)$")
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
                if let Some(battery) = captures.get(8) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from TRIP");
            }
        },
        _ => {
            // Events not currently supported.
            // Only appear if show out of trip or supported flag set.
            // Event and attributes will not be formatted.
            // Setting unsupported flag to false.
            *ev_supported = false;
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
