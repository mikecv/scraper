// Application struct and methods.

use log::info;

use eframe::{egui, App, Frame};

use crate::scraper::Scraper;
use crate::ui;
use crate::log_display::UiState;

// Make the MyApp struct public.
pub struct MyApp {
    pub scraper: Scraper,
    pub show_oot_events: bool,
    pub show_input_events: bool,
    pub show_report_events: bool,
    pub show_debug_events: bool,
    pub show_about: bool,
    pub show_help: bool,
    pub help_detached: bool,
    pub about_icon: Option<egui::TextureHandle>,
    pub help_image_1: Option<egui::TextureHandle>,
    pub ui_state: UiState,
}

impl Default for MyApp {
    fn default() -> Self {
        info!("Creating new instance of MyApp.");

        Self {
            scraper: Scraper::default(),
            show_oot_events: false,
            show_input_events: false,
            show_report_events: false,
            show_debug_events: false,
            show_about: false,
            show_help: false,
            help_detached: false,
            about_icon: None,
            help_image_1: None,
            ui_state: UiState::default(),
        }
    }
}

impl MyApp {
    // Load the about icon (call this once when needed);
    pub fn load_about_icon(&mut self, ctx: &egui::Context) {
        if self.about_icon.is_none() {
            // Embed the icon at compile time.
            // Icon file should be in the assets folder.
            let icon_bytes = include_bytes!("../assets/about.png");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.about_icon = Some(ctx.load_texture("about_icon", color_image, Default::default()));
                    info!("About icon loaded successfully.");
                }
                Err(e) => {
                    info!("Failed to load embedded about icon: {}", e);
                }
            }
        }
    }

    // Load all help images here.
    pub fn load_help_images(&mut self, ctx: &egui::Context) {
        // Load first help image.
        if self.help_image_1.is_none() {
            let icon_bytes = include_bytes!("../assets/podaca.jpeg");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.help_image_1 = Some(ctx.load_texture("help_image_1", color_image, Default::default()));
                }
                Err(e) => info!("Failed to load help image 1: {}", e),
            }
        }
    }
}

// Implement the eframe::App trait for MyApp.
impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Check for dropped file first.
        // Then check for file dialog file.
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            info!("File dropped - reinitializing data");
            self.scraper.reinitialize_data();
            
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            if let Some(file) = dropped_files.first() {
                if let Some(path) = &file.path {
                    info!("Processing dropped file: {:?}", path);
                    self.scraper.load_file_from_path(path);
                }
            }
        }
        
        // Here, we delegate the actual UI drawing to functions
        // in the ui module. We pass `self` (or parts of it)
        // so the UI functions can access and modify the state.
        ui::draw_menu_bar(self, ctx);
        ui::draw_bottom_panel(self, ctx); // Add the bottom panel
        ui::draw_central_panel(self, ctx);

        // Handle modal dialogs.
        ui::draw_about_dialog(self, ctx);
        ui::draw_help_panel(self, ctx);
    }
}
