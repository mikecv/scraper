// Scraper structure and methods.

use log::info;

use crate::settings::Settings;
use crate::SETTINGS;

#[derive(Debug)]
pub struct Scraper {
    pub settings: Settings,
}

impl Scraper {
    // A function to create a new Scraper instance.
    pub fn new() -> Self {
        info!("Creating new instance of Scraper.");

        // Lock the global SETTINGS to obtain access to the Settings object.
        let settings = SETTINGS.lock().unwrap().clone();

        Self {
            settings: settings,
        }
    }

    // Example method for your scraper's logic
    pub fn _start_scraping(&mut self) {
        info!("Use of {:?}", self.settings.program_name);
    }
}

// Implement Default for way to create a 'blank' instance.
impl Default for Scraper {
    fn default() -> Self {
        Self::new()
    }
}
