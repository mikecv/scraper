// Structure and methods for UI.

use log::info;

use eframe::{egui};

use crate::app::MyApp;

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
            ui.menu_button("About", |ui| {
                if ui.button("About").clicked() {
                    info!("About button clicked.");
                    ui.close_menu();
                }
                if ui.button("Help").clicked() {
                    info!("Help button clicked.");
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
pub fn draw_central_panel(_app: &mut MyApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |_ui| {
    });
}
