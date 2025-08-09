// Application struct and methods.

use log::info;

use eframe::{egui, App};
use walkers::{MapMemory, sources::OpenStreetMap};
use egui::epaint::{CornerRadius};

use crate::scraper::Scraper;
use crate::ui;
use crate::log_display::UiState;
use crate::time_series_plot::PlotState;

// Make the MyApp struct public.
pub struct MyApp {
    pub scraper: Scraper,
    pub show_oot_events: bool,
    pub show_input_events: bool,
    pub show_report_events: bool,
    pub show_debug_events: bool,
    pub show_gps_events: bool,
    pub show_time_series: bool,
    pub show_about: bool,
    pub show_help: bool,
    pub about_icon: Option<egui::TextureHandle>,
    pub show_changelog: bool,
    pub ui_state: UiState,
    pub selected_id: Option<String>,
    pub dark_mode: bool,
    pub show_gps_plot: bool,
    pub use_simple_plot: bool,
    pub use_street_tiles: bool,
    pub use_satellite_tiles: bool,
    pub map_memory: MapMemory,
    pub last_trip_id: Option<String>,
    pub map_tiles: Option<walkers::HttpTiles>,
    pub satellite_tiles: Option<walkers::HttpTiles>,
    pub plot_state: PlotState,
    _runtime: tokio::runtime::Runtime,
    
    // Help images.
    pub help_image_1: Option<egui::TextureHandle>,
    pub help_image_2: Option<egui::TextureHandle>,
    pub help_image_3: Option<egui::TextureHandle>,
    pub help_image_4: Option<egui::TextureHandle>,
    pub help_image_5: Option<egui::TextureHandle>,
    pub help_image_6: Option<egui::TextureHandle>,
    pub help_image_7: Option<egui::TextureHandle>,
    pub help_image_8: Option<egui::TextureHandle>,
    pub help_image_9: Option<egui::TextureHandle>,
    pub help_image_10: Option<egui::TextureHandle>,
    pub help_image_11: Option<egui::TextureHandle>,
    pub help_image_12: Option<egui::TextureHandle>,
    pub help_image_13: Option<egui::TextureHandle>,
    pub help_image_14: Option<egui::TextureHandle>,
    pub help_image_15: Option<egui::TextureHandle>,
    pub help_image_16: Option<egui::TextureHandle>,
}

impl Default for MyApp {
    fn default() -> Self {
        info!("Creating new instance of MyApp.");

        let runtime = tokio::runtime::Runtime::new().unwrap();

        Self {
            scraper: Scraper::default(),
            show_oot_events: false,
            show_input_events: false,
            show_report_events: false,
            show_debug_events: false,
            show_gps_events: false,
            show_time_series: false,
            show_about: false,
            show_help: false,
            about_icon: None,
            show_changelog: false,
            ui_state: UiState::default(),
            selected_id: Some("".to_string()),
            dark_mode: true,
            show_gps_plot: false,
            use_simple_plot: true,
            use_street_tiles: false,
            use_satellite_tiles: false,
            map_memory: MapMemory::default(),
            last_trip_id: None,
            map_tiles: None,
            satellite_tiles: None,
            plot_state: PlotState::default(),
            _runtime: runtime,

            // Help images.
            help_image_1: None,
            help_image_2: None,
            help_image_3: None,
            help_image_4: None,
            help_image_5: None,
            help_image_6: None,
            help_image_7: None,
            help_image_8: None,
            help_image_9: None,
            help_image_10: None,
            help_image_11: None,
            help_image_12: None,
            help_image_13: None,
            help_image_14: None,
            help_image_15: None,
            help_image_16: None,
        }
    }
}

impl MyApp {
    // Initialize street view tiles when needed.
    pub fn ensure_street_tiles(&mut self, _ctx: &egui::Context) {
        if self.map_tiles.is_none() {
            info!("Initializing street view tiles");
            // HttpTiles needs a context to create itself, so pass it here.
            self.map_tiles = Some(walkers::HttpTiles::new(OpenStreetMap, _ctx.clone()));
        }
    }

    // Initialize satelitte view tiles when needed.
    pub fn ensure_satellite_tiles(&mut self, _ctx: &egui::Context) {
        if self.satellite_tiles.is_none() {
            info!("Initializing satellite view tiles");
            // Use custom satellite tile source.
            self.satellite_tiles = Some(walkers::HttpTiles::new(crate::gps_plot::SatelliteTiles, _ctx.clone()));
        }
    }

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
    // Images get loader early, so that they load quickly when needed.
    pub fn load_help_images(&mut self, ctx: &egui::Context) {
        // Help image 1 - loading log file.
        if self.help_image_1.is_none() {
            let icon_bytes = include_bytes!("../assets/help-1.png");
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

        // Help image 2 - information panel.
        if self.help_image_2.is_none() {
            let icon_bytes = include_bytes!("../assets/help-2.png");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.help_image_2 = Some(ctx.load_texture("help_image_2", color_image, Default::default()));
                }
                Err(e) => info!("Failed to load help image 2: {}", e),
            }
        }

        // Help image 3 - events in trip.
        if self.help_image_3.is_none() {
            let icon_bytes = include_bytes!("../assets/help-3.png");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.help_image_3 = Some(ctx.load_texture("help_image_3", color_image, Default::default()));
                }
                Err(e) => info!("Failed to load help image 3: {}", e),
            }
        }

        // Help image 4 - event details.
        if self.help_image_4.is_none() {
            let icon_bytes = include_bytes!("../assets/help-4.png");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.help_image_4 = Some(ctx.load_texture("help_image_4", color_image, Default::default()));
                }
                Err(e) => info!("Failed to load help image 4: {}", e),
            }
        }

        // Help image 5 - show event options menu.
        if self.help_image_5.is_none() {
            let icon_bytes = include_bytes!("../assets/help-5.png");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.help_image_5 = Some(ctx.load_texture("help_image_5", color_image, Default::default()));
                }
                Err(e) => info!("Failed to load help image 5: {}", e),
            }
        }

        // Help image 6 - optional trip events.
        if self.help_image_6.is_none() {
            let icon_bytes = include_bytes!("../assets/help-6.png");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.help_image_6 = Some(ctx.load_texture("help_image_6", color_image, Default::default()));
                }
                Err(e) => info!("Failed to load help image 6: {}", e),
            }
        }

        // Help image 7 - event gps details.
        if self.help_image_7.is_none() {
            let icon_bytes = include_bytes!("../assets/help-7.png");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.help_image_7 = Some(ctx.load_texture("help_image_7", color_image, Default::default()));
                }
                Err(e) => info!("Failed to load help image 7: {}", e),
            }
        }

        // Help image 8 - out of trip event.
        if self.help_image_8.is_none() {
            let icon_bytes = include_bytes!("../assets/help-8.png");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.help_image_8 = Some(ctx.load_texture("help_image_8", color_image, Default::default()));
                }
                Err(e) => info!("Failed to load help image 8: {}", e),
            }
        }

        // Help image 9 - out of trip event details.
        if self.help_image_9.is_none() {
            let icon_bytes = include_bytes!("../assets/help-9.png");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.help_image_9 = Some(ctx.load_texture("help_image_9", color_image, Default::default()));
                }
                Err(e) => info!("Failed to load help image 9: {}", e),
            }
        }

        // Help image 10 - plot menu.
        if self.help_image_10.is_none() {
            let icon_bytes = include_bytes!("../assets/help-10.png");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.help_image_10 = Some(ctx.load_texture("help_image_10", color_image, Default::default()));
                }
                Err(e) => info!("Failed to load help image 10: {}", e),
            }
        }

        // Help image 11 - naked gps plot.
        if self.help_image_11.is_none() {
            let icon_bytes = include_bytes!("../assets/help-11.png");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.help_image_11 = Some(ctx.load_texture("help_image_11", color_image, Default::default()));
                }
                Err(e) => info!("Failed to load help image 11: {}", e),
            }
        }

        // Help image 12 - gps plot with street view tile background.
        if self.help_image_12.is_none() {
            let icon_bytes = include_bytes!("../assets/help-12.png");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.help_image_12 = Some(ctx.load_texture("help_image_12", color_image, Default::default()));
                }
                Err(e) => info!("Failed to load help image 12: {}", e),
            }
        }

        // Help image 13 - gps plot with satelitte view tile background.
        if self.help_image_13.is_none() {
            let icon_bytes = include_bytes!("../assets/help-13.png");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.help_image_13 = Some(ctx.load_texture("help_image_13", color_image, Default::default()));
                }
                Err(e) => info!("Failed to load help image 13: {}", e),
            }
        }

        // Help image 14 - street view gps plot pan and zoom.
        if self.help_image_13.is_none() {
            let icon_bytes = include_bytes!("../assets/help-14.png");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.help_image_14 = Some(ctx.load_texture("help_image_14", color_image, Default::default()));
                }
                Err(e) => info!("Failed to load help image 14: {}", e),
            }
        }

        // Help image 15 - light and dark mode menu.
        if self.help_image_14.is_none() {
            let icon_bytes = include_bytes!("../assets/help-15.png");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.help_image_15 = Some(ctx.load_texture("help_image_15", color_image, Default::default()));
                }
                Err(e) => info!("Failed to load help image 15: {}", e),
            }
        }

        // Help image 16 - trip and event font sizes.
        if self.help_image_16.is_none() {
            let icon_bytes = include_bytes!("../assets/help-16.png");
            match image::load_from_memory(icon_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.help_image_16 = Some(ctx.load_texture("help_image_16", color_image, Default::default()));
                }
                Err(e) => info!("Failed to load help image 16: {}", e),
            }
        }
    }
}

// Implement the eframe::App trait for MyApp.
impl App for MyApp {

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Apply theme according to menu selection.
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // Add border to the main window.
        self.draw_main_window_border(ctx, frame);

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
        ui::draw_bottom_panel(self, ctx);
        ui::draw_central_panel(self, ctx);

        // Handle modal dialogs.
        ui::draw_about_dialog(self, ctx);
        ui::draw_help_panel(self, ctx);
        ui::draw_changelog(self, ctx);

        // Check if we need to plot gps data.
        if self.show_gps_plot {
            ui::draw_gps_plot_window(self, ctx);
        }

        // Check if we need to plot time series data.
        if self.show_time_series {
            ui::draw_time_series_window(self, ctx);
        }
    }
}

impl MyApp {
    // Draw border around the main window.
    fn draw_main_window_border(&self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let screen_rect = ctx.screen_rect();
        let border_width = 8.0;
        let border_colour = crate::colours::border_colour(self.dark_mode);

        // Use the exact same approach as other window borders.
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("main_border")));

        painter.rect_stroke(
            screen_rect.shrink(border_width / 2.0),
            CornerRadius::same(0),
            egui::Stroke::new(border_width, border_colour),
            egui::epaint::StrokeKind::Outside,
        );
    }
}
