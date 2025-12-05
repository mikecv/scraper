// Scraper structure and methods.

use log::info;
use log::warn;

use chrono::NaiveDateTime;
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
    pub devices: Vec<String>,
    pub selected_device: String,
    pub device_id: String,
    pub device_fw: String,
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
            devices: Vec::new(),
            selected_device: "".to_string(),
            device_id: "".to_string(),
            device_fw: "".to_string(),
            scrapings: Vec::new(),
        }
    }
}

impl Scraper {
    // Load log file for processing.
    // The load file triggers a clearing of any previous selection id.
    pub fn load_file(&mut self, ctx: &egui::Context, selected_id: &mut Option<String>) {
        info!("Browsing for file to load.");

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
                    .add_filter("Log files", &["csv", "txt"])
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
                    Some((&["*.csv", "*.txt"], "Log files (csv, txt)")),
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
        self.devices = Vec::new();
        self.device_id = "".to_string();
        self.selected_device = "".to_string();
        self.device_fw = "".to_string();

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

    // Do the log parsing for the selected device.
    pub fn parse_selected_device(&mut self) {
        if let Some(path) = &self.selected_file.clone() {
            info!("Re-parsing file for device: {}", self.selected_device);
            self.scrapings.clear();
            
            // Clone the selected_device to avoid borrow conflicts.
            let device = self.selected_device.clone();
            
            match self.parse_device_data(&path, &device) {
                Ok(_) => {
                    info!("Successfully parsed data for device {}", device);
                    self.processing_status = format!("Loaded device: {}", device);
                }
                Err(e) => {
                    self.processing_status = format!("Error parsing device data: {}", e);
                }
            }
        }
    }

    // Read_and_process_file done in 2 two passes.
    // First pass: collect all devices
    // Second pass: parse data for the first device (or selected one)
    fn read_and_process_file(&mut self, path: &PathBuf) -> Result<usize, Box<dyn std::error::Error>> {
        // Clear fields at start of processing to ensure clean state.
        self.devices = Vec::new();
        self.selected_device.clear();
        self.device_fw.clear();

        // Open the file.
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        // Collect all lines into a vector.
        let lines: Vec<String> = reader.lines()
            .collect::<Result<_, _>>()?;

        // === FIRST PASS: Find all devices ===
        let sn_pattern = Regex::new(r"\[UNIT\s+(\d+)\]")?;
        
        info!("Searching file for all device serial numbers.");
        
        for line in lines.iter() {
            if let Some(caps) = sn_pattern.captures(&line) {
                let unit_number = &caps[1];
                let unit = if let Some(suffix) = caps.get(10) {
                    format!("{} {}", unit_number, suffix.as_str())
                } else {
                    unit_number.to_string()
                };

                // Add to list of devices if unique.
                if !self.devices.contains(&unit) {
                    info!("Found device: {:?}", unit);
                    self.devices.push(unit.clone());
                }
            }
        }

        // Select first device if we found any.
        if self.devices.len() == 1 {
            self.selected_device = self.devices[0].clone();
            info!("Auto-selecting first device: {}", self.selected_device);
        }
        else {
            if self.devices.is_empty() {
                self.selected_device = "Not defined.".to_string();
                info!("No devices found in file.");
                return Ok(0);
            }
        }

        // SECOND PASS: Parse firmware version for selected device.
        self.parse_firmware(&lines)?;

        // THIRD PASS: Parse events for selected device.
        let selected = self.selected_device.clone();
        self.parse_device_data(path, &selected)?;

        Ok(0)
    }

    // Extract firmware parsing into separate method.
    fn parse_firmware(&mut self, lines: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        info!("Searching file for device firmware version.");
        
        let fw_pattern = Regex::new(r#"^"([^@]+) @ ([^"]+)",[^,]+,[^,]+,"[^<]+ <== \[EVENT (\d+) (\d+) [^\s]+ (SWSTART) FC ([^v]+) v:\d+\]"$"#)?;

        for line in lines.iter().rev() {
            if let Some(captures) = fw_pattern.captures(&line) {
                let fw_str = captures.get(6).unwrap().as_str();
                self.device_fw = fw_str.to_string();
                info!("Found device firmware: {:?}", fw_str);
                return Ok(());
            }
        }
        
        self.device_fw = "Not defined.".to_string();
        info!("Failed to find device firmware version.");
        Ok(())
    }

    // Extract event parsing into separate method that takes device ID.
    // List of devices is found on file load and not repeated until next file.
    // Here there is a selected device nomimated.
    fn parse_device_data(&mut self, path: &PathBuf, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Parsing events for device: {}", device_id);

        // Open file and read lines.
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

        let ev_pattern = Regex::new(r#"^"([^@]+) @ ([^"]+)",[^,]+,[^,]+,"([^<]+) <== \[EVENT (\d+) (\d+) (-?\d+)/(-?\d+)/(\d+)/(-?\d+)/(\d+) (\w+) ([^\]]+)\]"$"#)?;

        let mut trip_num_id = String::new();

        // Process file line by line (reversed to get chronological order).
        for line in lines.iter().rev() {
            if let Some(captures) = ev_pattern.captures(&line) {
                // Check if this event is for the selected device.
                let event_controller = captures.get(3).unwrap().as_str();
                
                // Ignore event as not interested in this device,
                // i.e. not the selected device.
                if event_controller != device_id {
                    continue;
                }

                // Extract date, time, event details, GPS data, etc.              
                let date_str = captures.get(1).unwrap().as_str();
                let time_str = captures.get(2).unwrap().as_str();

                // Need to convert the time as different format in log file.
                // compared to format in trip and event lists.
                let input_datetime = format!("{} {}", date_str, time_str);
                let (date, time) = if let Ok(dt) = NaiveDateTime::parse_from_str(&input_datetime, "%b %d, %Y %H:%M:%S%.f") {
                    let date = dt.format("%d/%m/%Y").to_string();
                    let time = dt.format("%H:%M:%S").to_string();
                    (date, time)
                } else {
                    warn!("Failed to parse datetime: {}", input_datetime);
                    (date_str.to_string(), time_str.to_string())
                };

                let unix_time = captures.get(5).unwrap().as_str();
                let event_type = captures.get(11).unwrap().as_str();
                let event_detail = captures.get(12).unwrap().as_str();
                let mut on_trip = true;
                let mut ev_supported = true;
                let ev_key_vals = ungroup_event_data(event_type.to_string(), event_detail, &mut on_trip, &mut ev_supported);
                let trip_id = captures.get(4).unwrap().as_str();

                let gps_latitude = captures.get(6).unwrap().as_str().parse::<f64>()?;
                let gps_longitude = captures.get(7).unwrap().as_str().parse::<f64>()?;
                let gps_locn = GpsLocation {
                    lat: gps_latitude / 10_000_000.0,
                    lon: gps_longitude / 10_000_000.0,
                };

                let gps_rssi = captures.get(9).unwrap().as_str().parse::<u32>()?;
                let gps_speed = captures.get(10).unwrap().as_str().parse::<u32>()?;

                // If event is SIGNON then this is that start of new trip.
                if event_type == "SIGNON" {
                    trip_num_id = trip_id.to_string();
                }

                let ev_data = ScrapedData {
                    date_time: format!("{} {}", date, time),
                    unix_time: unix_time.parse().expect("Invalid Unix time string"),
                    on_trip,
                    trip_num: trip_num_id.clone(),
                    event_type: event_type.to_string(),
                    ev_detail: ev_key_vals,
                    ev_supported,
                    gps_locn,
                    gps_rssi,
                    gps_speed,
                };

                self.scrapings.push(ev_data);

                // End of the current trip if event is TRIP.
                // If trip not started per chance then these events
                // will be outside of a trip.
                if event_type == "TRIP" {
                    trip_num_id = String::new();
                }
            }
        }

        info!("Parsed {} events for device {}", self.scrapings.len(), device_id);
        Ok(())
    }
}

// Function to expand on the scraped data.
fn ungroup_event_data(event_type: String, sub_data: &str, on_trip: &mut bool, ev_supported: &mut bool) -> Vec<(String, String)> {
    // Initialise result vector.
    let mut result = Vec::new();

    // Search for the event sub-data for the SIGNON event.
    match event_type.as_str() {
        "SIGNON" => {
            let sub_signon_pattern = Regex::new(r"([-*+0-9]+) ([0-9a-fA-F]+) (.+?) ([0-9]+) ([0-9a-fA-F]+) ([0-9]+) v:(.+?)$")
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
                    warn!("Failed to extract sub-data from SIGNON: {:?}", sub_data);
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
        // Loosely one attribute after the DEBUG event name.
        "DEBUG" => {
            let sub_debug_pattern = Regex::new(r"(.+)$")
                .expect("Invalid DEBUG regex pattern");

            if let Some(captures) = sub_debug_pattern.captures(sub_data) {
                if let Some(error) = captures.get(1) {
                    result.push(("Debug error".to_string(), error.as_str().to_string()));
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

                // The HARDWARE event only occurs out of trip.
                // Setting HARDWARE on_trip flag to false.
                *on_trip = false;

            } else {
                warn!("Failed to extract sub-data from HARDWARE");
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
                    result.push(("Input".to_string(), input_num.as_str().to_string()));
                }
                if let Some(input_state) = captures.get(2) {
                    result.push(("State".to_string(), input_state.as_str().to_string()));
                }
                if let Some(active_time) = captures.get(3) {
                    result.push(("Duration".to_string(), active_time.as_str().to_string()));
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

        // Search for the event sub-data for the POWERDOWN event.
        "POWERDOWN" => {
            let sub_power_pattern = Regex::new(r"v:(.+?)$")
                .expect("Invalid POWERDOWN regex pattern");

            if let Some(captures) = sub_power_pattern.captures(sub_data) {
                if let Some(battery) = captures.get(1) {
                    if let Ok(voltage_tens) = battery.as_str().parse::<f32>() {
                        let voltage_volts = voltage_tens / 10.0;
                        result.push(("Battery voltage".to_string(), format!("{:.1}", voltage_volts)));
                    }
                }
            } else {
                warn!("Failed to extract sub-data from POWERDOWN: {:?}", sub_data);
            }
        },

        // Search for the event sub-data for the REPORT event.
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
            let sub_xsidle_pattern = Regex::new(r"([0-9]+) ([0-9]+) v:(.+?)$")
                .expect("Invalid XSIDLE regex pattern");

            if let Some(captures) = sub_xsidle_pattern.captures(sub_data) {
                if let Some(trip_id) = captures.get(1) {
                    result.push(("Trip id".to_string(), trip_id.as_str().to_string()));
                }
                if let Some(max_idle) = captures.get(2) {
                    result.push(("Max idle".to_string(), max_idle.as_str().to_string()));
                }
                if let Some(battery) = captures.get(3) {
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

                // The ZONE_OK event only occurs out of trip.
                // Setting ZONE_OK on_trip flag to false.
                *on_trip = false;

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
