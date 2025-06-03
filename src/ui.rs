// Structure and methods for UI.

use log::info;

use eframe::{egui};

use crate::app::MyApp;
use crate::help_content;
use crate::DETAILS;

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

// Function to draw the bottom status panel.
// This is a strip at the bottom of the screen to show
// controller details.
pub fn draw_bottom_panel(app: &mut MyApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("bottom_panel")
        .min_height(30.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(0.5);
                
                // First row: File and Controller ID.
                ui.horizontal(|ui| {
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Body,
                        egui::FontId::new(10.0, egui::FontFamily::Proportional)
                    );
                    
                    // Display selected file info if available.
                    if let Some(filename) = app.scraper.get_selected_filename() {
                        ui.label("File:");
                        ui.strong(filename);
                        
                        // Add controller ID if available.
                        if !app.scraper.controller_id.is_empty() {
                            ui.separator();
                            ui.label("Controller ID:");
                            ui.strong(format!("{:0>6}", app.scraper.controller_id));
                        }

                        // Add controller firmware version if available.
                        if !app.scraper.controller_fw.is_empty() {
                            ui.separator();
                            ui.label("Firmware Version:");
                            ui.strong(format!("{:?}", app.scraper.controller_fw));
                        }
                    } else {
                        ui.label("No file selected");
                    }
                });
                
                ui.add_space(0.5);
                
                // Second row: Status
                ui.horizontal(|ui| {
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Body,
                        egui::FontId::new(10.0, egui::FontFamily::Proportional)
                    );
                    
                    ui.label("Status:");
                    ui.label(app.scraper.get_processing_status());
                });
                
                ui.add_space(0.5);
            });
        });
}

// Function to draw the main content area.
pub fn draw_central_panel(app: &mut MyApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Calculate available height for the scrollable area
        let available_height = ui.available_height();
        
        // Call the demo table renderer
        crate::ui_demo::render_event_table(ui, &mut app.ui_state, available_height);
    });
}

// Function to draw the About dialog.
pub fn draw_about_dialog(app: &mut MyApp, ctx: &egui::Context) {
    if app.show_about {

        // Try to load the icon if it hasn't been loaded yet.
        app.load_about_icon(ctx);

        // Lock the global DETAILS to obtain access to the Details object.
        let details = DETAILS.lock().unwrap().clone();

        egui::Window::new("About Scraper")
            .collapsible(false)
            .resizable(false)
            .default_width(300.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    // Try to display the loaded icon, fallback to the default one if not available.
                    if let Some(texture) = &app.about_icon {
                        ui.image((texture.id(), egui::Vec2::new(64.0, 64.0)));
                    } else {
                        // Fallback to a simple svg coded image.
                        let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(64.0, 64.0), egui::Sense::hover());
                        ui.painter().circle_filled(rect.center(), 32.0, egui::Color32::from_rgb(70, 130, 180));
                        ui.painter().circle_filled(rect.center(), 28.0, egui::Color32::from_rgb(100, 160, 210));
                        ui.painter().circle_filled(rect.center(), 20.0, egui::Color32::from_rgb(130, 190, 240));
                    }

                    // About application details.
                    ui.heading(details.program_name);
                    ui.separator();                  
                    ui.label(format!("Version: {:?}", details.program_ver));
                    ui.label(format!("Devs: {:?}", details.program_devs));
                    ui.label(format!("Build Date: {:?}", details.program_date));
                    ui.label(format!("Web: {:?}", details.program_web));
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

// Function to draw the detachable Help panel/window.
pub fn draw_help_panel(app: &mut MyApp, ctx: &egui::Context) {
    if app.show_help {

        // Load help images if not loaded.
        app.load_help_images(ctx);

        if app.help_detached {
            // Create a detached window in its own viewport.
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("help_window"),
                egui::ViewportBuilder::default()
                    .with_title("Help - Scraper")
                    .with_inner_size([600.0, 500.0])
                    .with_resizable(true),
                |ctx, class| {
                    assert!(class == egui::ViewportClass::Immediate);
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            // Show help attached to main window.
                            if ui.button("Help").clicked() {
                                info!("Help button clicked.");
                                app.show_help = true;
                                app.help_detached = true;
                                ui.close_menu();
                            }
                            // Attach help dialog to main window.
                            if ui.button("Attach to Main Window").clicked() {
                                app.help_detached = false;
                            }
                            // Close help menu.
                            ui.separator();
                            if ui.button("Close").clicked() {
                                app.show_help = false;
                                app.help_detached = false;
                            }

                        });
                        ui.separator();
                        
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            help_content::draw_help_content(ui, app);
                        });
                    });
                },
            );
        } else {
            // Regular attached window.
            egui::Window::new("Help")
                .default_width(500.0)
                .default_height(400.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Detach Window").clicked() {
                            app.help_detached = true;
                        }
                        ui.separator();
                        if ui.button("Close").clicked() {
                            app.show_help = false;
                        }
                    });
                    ui.separator();
                    
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        help_content::draw_help_content(ui, app);
                    });
                });
        }
    }
}
