// Structure and methods for UI.

// use log::info;

use eframe::{egui};

use crate::app::MyApp;

// Function to draw the menu bar.
pub fn draw_menu_bar(_app: &mut MyApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Exit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    ui.close_menu();
                }
            });
        });
    });
}

// Function to draw the main content area
pub fn draw_central_panel(_app: &mut MyApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Web Scraper Control Panel");
    });
}
