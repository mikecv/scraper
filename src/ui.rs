// Structure and methods for UI.

use log::info;

use eframe::{egui};

use crate::app::MyApp;
use crate::help_content;

// Function to draw the menu bar.
pub fn draw_menu_bar(app: &mut MyApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    info!("Open file button clicked.");
                    app.scraper.load_file(ctx);
                    ui.close_menu();
                }
            });
            ui.menu_button("Help", |ui| {
                if ui.button("Help").clicked() {
                    info!("Help button clicked.");
                    app.show_help = true;
                    ui.close_menu();
                }
                if ui.button("About").clicked() {
                    info!("About button clicked.");
                    app.show_about = true;
                    ui.close_menu();
                }
            });
            ui.menu_button("Quit", |ui| {
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    ui.close_menu();
                }
            });
        });
    });
}

// Function to draw the main content area.
pub fn draw_central_panel(app: &mut MyApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Check for file dialog results.
        app.scraper.check_file_dialog();
        
        // Display selected file info if available.
        // If not loaded then default image will be displayed.
        if let Some(filename) = app.scraper.get_selected_filename() {
            ui.label(format!("Selected file: {}", filename));
        }
    });
}

// Function to draw the About dialog.
pub fn draw_about_dialog(app: &mut MyApp, ctx: &egui::Context) {
    if app.show_about {
        // Try to load the icon if it hasn't been loaded yet.
        app.load_about_icon(ctx);
        
        egui::Window::new("About Scraper")
            .collapsible(false)
            .resizable(false)
            .default_width(300.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    // Try to display the loaded icon, fallback to circles if not available.
                    if let Some(texture) = &app.about_icon {
                        ui.image((texture.id(), egui::Vec2::new(64.0, 64.0)));
                    } else {
                        // Fallback to the circle design.
                        let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(64.0, 64.0), egui::Sense::hover());
                        ui.painter().circle_filled(rect.center(), 32.0, egui::Color32::from_rgb(70, 130, 180));
                        ui.painter().circle_filled(rect.center(), 28.0, egui::Color32::from_rgb(100, 160, 210));
                        ui.painter().circle_filled(rect.center(), 20.0, egui::Color32::from_rgb(130, 190, 240));
                    }

                    ui.heading("Scraper");
                    ui.separator();                  
                    ui.label("Version: 0.0.1");
                    ui.label("Devs: MDC");
                    ui.label("Build Date: 2025");
                    ui.label("Built with Rust & eframe");
                    ui.separator();
                    ui.label("A tool for scraping and presenting log data.");
                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() {
                            app.show_about = false;
                        }
                    });
                });
            });
    }
}

// Function to draw the Help panel/window - now uses separated content
pub fn draw_help_panel(app: &mut MyApp, ctx: &egui::Context) {
    if app.show_help {

        // Try to load the help images if they haven't been loaded yet.
        app.load_help_images(ctx);

        egui::Window::new("Help")
            .default_width(500.0)
            .default_height(400.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Use the separated help content.
                    help_content::draw_help_content(ui, app);
                });
            });
    }
}
