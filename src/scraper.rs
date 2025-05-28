// Scraper structure and methods.

use log::info;
use rfd::AsyncFileDialog;
use std::path::PathBuf;
use std::sync::mpsc;

use crate::egui;
use crate::settings::Settings;
use crate::SETTINGS;

#[allow(dead_code)]

#[derive(Debug)]
pub enum FileDialogMessage {
    FileSelected(PathBuf),
    DialogClosed,
}

// Scraper struct and methods.
#[derive(Debug)]
pub struct Scraper {
    pub settings: Settings,
    pub selected_file: Option<PathBuf>,
    pub file_dialog_open: bool,
    pub file_receiver: Option<mpsc::Receiver<FileDialogMessage>>,
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
        }
    }

    // Method to browse for file to open.
    pub fn load_file(&mut self, ctx: &egui::Context) {
        // info!("{:?}", self.settings.test_string);
        info!("Browsing for file to open.");

        // Prevent multiple dialogs.
        if self.file_dialog_open {
            return;
        }

        // Set that file open dialog is now open.
        self.file_dialog_open = true;
        
        // Create a channel for communication.
        let (sender, receiver) = mpsc::channel();
        self.file_receiver = Some(receiver);
        
        let ctx = ctx.clone();
        let task = AsyncFileDialog::new()
            // Set file types to accept (from settings).
            .add_filter("Log file", &self.settings.file_types)
            .pick_file();

        // Execute the async task to browse for file.
        tokio::spawn(async move {
            match task.await {
                Some(file) => {
                    let path = file.path().to_path_buf();
                    info!("File selected: {:?}", path);
                    let _ = sender.send(FileDialogMessage::FileSelected(path));
                }
                None => {
                    info!("No file was selected.");
                    let _ = sender.send(FileDialogMessage::DialogClosed);
                }
            }
            ctx.request_repaint();
        });
 
        // Set that file open dialog is closed.
        // This allows recovery from an aborted browse.
        self.file_dialog_open = false;
    }

    // Method to check for file dialog results.
    pub fn check_file_dialog(&mut self) {
        if let Some(receiver) = &self.file_receiver {
            if let Ok(message) = receiver.try_recv() {
                match message {
                    FileDialogMessage::FileSelected(path) => {
                        self.selected_file = Some(path.clone());
                        self.process_file(&path);
                    }
                    FileDialogMessage::DialogClosed => {
                        // Dialog was closed without selection.
                        info!("Dialog was closed without selection.");
                        self.file_dialog_open = false;
                    }
                }
                self.file_dialog_open = false;
                self.file_receiver = None;
            }
        }
    }

    // Method to scrape the selected file.
    fn process_file(&self, path: &PathBuf) {
        info!("Scraping file: {:?}", path);
        // TODO
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
