// Structure and methods for UI.

use log::info;

use eframe::{egui};
use egui::epaint::{CornerRadius};

use crate::plots;
use crate::colours;
use crate::app::MyApp;
use crate::help_content;
use crate::changelog_content;
use crate::DETAILS;

// Function to draw the menu bar.
pub fn draw_menu_bar(app: &mut MyApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {

            // File menu.
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    info!("Open file button clicked.");
                    app.scraper.load_file(ctx, &mut app.selected_id);
                    ui.close_menu();
                }
            });

            // Show menu.
            ui.menu_button("Show", |ui| {
                ui.checkbox(&mut app.show_input_events, "Input events");
                ui.checkbox(&mut app.show_report_events, "Report events");
                ui.checkbox(&mut app.show_debug_events, "Debug events");
                ui.separator();                
                ui.checkbox(&mut app.show_gps_events, "GPS event data");
                ui.separator();                
                ui.checkbox(&mut app.show_oot_events, "Out of trip events");
            });

            // Plot menu.
            ui.menu_button("Plot", |ui| {
                if ui.button("GPS Data").clicked() {
                    info!("GPS Data button clicked.");
                    app.show_gps_plot = true;
                    ui.close_menu();
                }
                ui.separator();
                let checkbox_response = ui.checkbox(&mut app.use_osm_tiles, "Use OSM tiles");
                if checkbox_response.hovered() {
                    checkbox_response.on_hover_text("Toggle between OSM tiles and simple plot view");
                }
            });

            // View menu.
            // For toggling dark and light mode.
            ui.menu_button("View", |ui| {
                let (icon, text) = if app.dark_mode { 
                    ("☀", "Light Mode")
                } else { 
                    ("🌙", "Dark Mode")
                };
                
                if ui.button(format!("{} {}", icon, text)).clicked() {
                    app.dark_mode = !app.dark_mode;
                    ui.close_menu();
                }
            });

            // Help menu.
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
                ui.separator();                
                if ui.button("Changelog").clicked() {
                    info!("Changelog button clicked.");
                    app.show_changelog = true;
                    ui.close_menu();
                }
            });

            // Quit menu.
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
                        else {
                            ui.separator();
                            ui.label("Controller ID:");
                            ui.strong("Not defined.");
                        }

                        // Add controller firmware version if available.
                        if !app.scraper.controller_fw.is_empty() {
                            ui.separator();
                            ui.label("Firmware:");
                            ui.strong(format!("{}", app.scraper.controller_fw));
                        }
                        else {
                            ui.separator();
                            ui.label("Firmware:");
                            ui.strong("Not defined.");
                        }
                    } else {
                        ui.label("No file selected.");
                    }
                });
                
                ui.add_space(0.5);

                // Second row: Program status, and selected trip.
                ui.horizontal(|ui| {
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Body,
                        egui::FontId::new(10.0, egui::FontFamily::Proportional)
                    );

                    // Program status.
                    ui.label("Status:");
                    ui.strong(app.scraper.get_processing_status());

                    // Selected trip at any level.
                    ui.separator();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        
                        // Right justified so add labels from right to left.
                        if let Some(id) = &app.selected_id {
                            ui.strong(format!("{:<10}", id));
                        } else {
                            ui.strong(format!("{:>10}", ""));
                        }                   
                        ui.label("Trip: ");
                    });

                    ui.add_space(0.5);
            });
        });
    });
}

// Draw the central panel with the log data.
pub fn draw_central_panel(app: &mut MyApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Update UI state with scraped data if available.
        if !app.scraper.scrapings.is_empty() {
            app.ui_state.update_with_scraped_data(&app.scraper.scrapings);
        }

        // Set the ui height to full available space.
        let available_height = ui.available_height();
        let available_width = ui.available_width();
        
        // Call the rendering function.
        crate::log_display::render_scraped_data(
            ui, 
            &mut app.ui_state, 
            &app.scraper.scrapings, 
            available_height,
            available_width,
            app.show_oot_events,
            app.show_input_events,
            app.show_report_events,
            app.show_debug_events,
            app.show_gps_events,
            &mut app.selected_id,
            app.dark_mode,
        );
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
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .frame(egui::Frame::window(&ctx.style()).stroke(egui::Stroke::new(3.0, colours::border_colour(app.dark_mode))))
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

// Function to draw the help panel.
pub fn draw_help_panel(app: &mut MyApp, ctx: &egui::Context) {

    // Lock the global DETAILS to obtain access to the Details object.
    let details = DETAILS.lock().unwrap().clone();

    if app.show_help {
        // Load help images if not loaded.
        app.load_help_images(ctx);

        // Create a detached window in its own viewport.
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("help_window"),
            egui::ViewportBuilder::default()
                .with_title("Scraper Help")
                .with_inner_size([details.help_win_height, details.help_win_width])
                .with_resizable(false)
                .with_min_inner_size([details.help_win_width, details.min_help_win_height])
                .with_max_inner_size([details.help_win_width, details.max_help_win_height]),
            |ctx, class| {
                assert!(class == egui::ViewportClass::Immediate);

                // Apply theme according to menu selection. This should be inside
                // the closure to ensure it's re-evaluated every frame.
                if app.dark_mode {
                    ctx.set_visuals(egui::Visuals::dark());
                } else {
                    ctx.set_visuals(egui::Visuals::light());
                }
                
                // Check if close was requested via the window's X button.
                if ctx.input(|i| i.viewport().close_requested()) {
                    app.show_help = false;
                }
                
                // Draw border around the help window.
                draw_viewport_border(ctx, app.dark_mode);
                
                // Determine the background color based on dark_mode.
                let background_color = if app.dark_mode {
                    ctx.style().visuals.widgets.noninteractive.bg_fill
                } else {
                    ctx.style().visuals.widgets.noninteractive.bg_fill
                };

                egui::CentralPanel::default()
                    .frame(egui::Frame::default()
                        .stroke(egui::Stroke::new(2.0, colours::border_colour(app.dark_mode)))
                        .inner_margin(egui::Margin::same(8))
                        .fill(background_color)
                    )
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            // Close help menu.
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
            },
        );
    }
}

// New function to draw the GPS plot window as a separate viewport.
pub fn draw_gps_plot_window(app: &mut MyApp, ctx: &egui::Context) {
    if app.show_gps_plot {

        // Lock the global DETAILS to obtain access settings.
        let details = DETAILS.lock().unwrap().clone();

        // Ensure map tiles are initialized if OSM is selected.
        if app.use_osm_tiles {
            app.ensure_map_tiles(ctx);
        }

        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("gps_plot_window"),
            egui::ViewportBuilder::default()
                .with_title("GPS Data Plot")
                .with_inner_size([details.gps_win_height, details.gps_win_width])
                .with_resizable(false),
            |ctx, class| {
                assert!(class == egui::ViewportClass::Immediate);

                // Apply theme based on main app setting.
                if app.dark_mode {
                    ctx.set_visuals(egui::Visuals::dark());
                } else {
                    ctx.set_visuals(egui::Visuals::light());
                }

                // Check if the viewport's native close button was clicked.
                if ctx.input(|i| i.viewport().close_requested()) {
                    // Set app state to false when window is closed.
                    app.show_gps_plot = false;
                }

                // Draw border around the gps plot window.
                draw_viewport_border(ctx, app.dark_mode);

                // Determine the background colour based on dark_mode.
                let background_color = if app.dark_mode {
                    ctx.style().visuals.widgets.noninteractive.bg_fill
                } else {
                    ctx.style().visuals.widgets.noninteractive.bg_fill
                };

                egui::CentralPanel::default()
                    .frame(egui::Frame::default()
                        .stroke(egui::Stroke::new(2.0, colours::border_colour(app.dark_mode)))
                        .inner_margin(egui::Margin::same(8))
                        .fill(background_color)
                    )
                    .show(ctx, |ui| {
                        ui.vertical(|ui| {
                            // Header section.
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Close").clicked() {
                                        app.show_gps_plot = false;
                                    }
                                });
                            });

                            ui.separator();

                            // Main plotting area.
                            ui.allocate_ui_with_layout(
                                egui::Vec2::new(ui.available_width(), ui.available_height()),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    // Call the appropriate plot function based on OSM tiles setting.
                                    if app.use_osm_tiles {
                                        // Pass the Option<HttpTiles> directly, and unwrap it safely within the function.
                                        // You need to ensure app.map_tiles is Some(HttpTiles) when this path is taken.
                                        // The ensure_map_tiles call earlier handles this.
                                        if let Some(map_tiles) = &mut app.map_tiles {
                                            plots::plot_gps_data_with_osm(ui, &app.scraper, &app.selected_id, &mut app.map_memory, map_tiles, &mut app.last_trip_id);
                                        } else {
                                            ui.label("Error: OSM tiles not initialized.");
                                        }
                                    } else {
                                        plots::plot_gps_data(ui, &app.scraper, &app.selected_id);
                                    }
                                }
                            );
                        });
                    });
            },
        );
    }
}

// Function to draw changelog.
pub fn draw_changelog(app: &mut MyApp, ctx: &egui::Context) {

    // Lock the global DETAILS to obtain access to the Details object.
    let details = DETAILS.lock().unwrap().clone();

    if app.show_changelog {
        // Create a detached window in its own viewport.
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("changlog_window"),
            egui::ViewportBuilder::default()
                .with_title("Changelog")
                .with_inner_size([details.changelog_win_height, details.changelog_win_width])
                .with_resizable(false),
            |ctx, class| {
                assert!(class == egui::ViewportClass::Immediate);

                // Apply theme according to menu selection. This should be inside
                // the closure to ensure it's re-evaluated every frame.
                if app.dark_mode {
                    ctx.set_visuals(egui::Visuals::dark());
                } else {
                    ctx.set_visuals(egui::Visuals::light());
                }
                
                // Check if close was requested via the window's X button.
                if ctx.input(|i| i.viewport().close_requested()) {
                    app.show_changelog = false;
                }
                
                // Draw border around the changelog window.
                draw_viewport_border(ctx, app.dark_mode);
                
                // Determine the background colour based on dark_mode.
                let background_color = if app.dark_mode {
                    ctx.style().visuals.widgets.noninteractive.bg_fill
                } else {
                    ctx.style().visuals.widgets.noninteractive.bg_fill
                };

                egui::CentralPanel::default()
                    .frame(egui::Frame::default()
                        .stroke(egui::Stroke::new(2.0, colours::border_colour(app.dark_mode)))
                        .inner_margin(egui::Margin::same(8))
                        .fill(background_color)
                    )
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            // Close changelog menu.
                            ui.separator();
                            if ui.button("Close").clicked() {
                                app.show_changelog = false;
                            }
                        });
                        ui.separator();
                        
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            changelog_content::draw_changelog_content(ui, app);
                        });
                    });
            },
        );
    }
}

// Helper function to draw border around viewport windows.
fn draw_viewport_border(ctx: &egui::Context, dark_mode: bool) {
    let screen_rect = ctx.screen_rect();
    let border_width = 3.0;
    let border_colour = colours::border_colour(dark_mode);

    // Draw the border using rect_stroke for a cleaner look.
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("viewport_border")));

    painter.rect_stroke(
        screen_rect.shrink(border_width / 2.0),
        CornerRadius::same(0),
        egui::Stroke::new(border_width, border_colour),
        egui::epaint::StrokeKind::Inside,
    );
}
