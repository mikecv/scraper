use eframe::{egui, App, Frame};

// Define a struct for your application state
struct MyApp {
    name: String,
    age: u32,
}

// Implement the Default trait to set up initial state
impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "User".to_owned(),
            age: 30,
        }
    }
}

// Implement the eframe::App trait for your application struct
impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // This is where you define your UI elements
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My Modern Windows-Style App");
            ui.horizontal(|ui| {
                ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));

            // Example of a simple menu bar (you'd typically put this in a TopBottomPanel)
            // For a more structured menu, you'd use TopBottomPanel::top and ui.menu_button
            // This is a simplified version for demonstration within the CentralPanel
            ui.separator(); // Separate from other content
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    // Placeholder for Open action
                    println!("Open action triggered!");
                    ui.close_menu();
                }
                if ui.button("Save").clicked() {
                    // Placeholder for Save action
                    println!("Save action triggered!");
                    ui.close_menu();
                }
                if ui.button("Exit").clicked() {
                    // To close the app, you'd typically send a quit event
                    // For simplicity here, we'll just print
                    println!("Exit action triggered!");
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    ui.close_menu();
                }
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    // Configure the native options for the window
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 400.0]), // Initial window size
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "My App Title", // The title of the window
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))), // Create and box your app state
    )
}
