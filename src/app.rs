// Application struct and methods.

use log::info;

use eframe::{egui, App, Frame};

use crate::settings::Settings;
use crate::SETTINGS;
use crate::scraper::Scraper;
use crate::ui;

// Make the struct public.
pub struct MyApp {
    pub settings: Settings,
    pub scraper: Scraper,
}

impl Default for MyApp {
    fn default() -> Self {
        info!("Creating new instance of MyApp.");

        // Lock the global SETTINGS to obtain access to the Settings object.
        let settings = SETTINGS.lock().unwrap().clone();

        Self {
            settings: settings,
            scraper: Scraper::default(),
        }
    }
}

// Implement the eframe::App trait for MyApp.
impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Here, we delegate the actual UI drawing to functions
        // in the ui module. We pass `self` (or parts of it)
        // so the UI functions can access and modify the state.
        ui::draw_menu_bar(self, ctx);
        ui::draw_central_panel(self, ctx);
    }
}
