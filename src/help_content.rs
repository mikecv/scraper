// Place for help content.
// Refer to ui.rs for associated ui definitions.

use eframe::egui;

use crate::app::MyApp;

// Render all help content using eframe calls.
pub fn draw_help_content(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.heading("Scraper Help");
    ui.separator();
    
    ui.collapsing("1.0 Getting Started", |ui| {
        ui.label("From the main menu select 'File' / 'Open' to select a log file to process.");
        ui.label("Alternatively, drag and drop a log file onto the application window.");
    });
    ui.collapsing("2.0 Scraped Data", |ui| {
        ui.label("On load, a scraped log file will list the trips in the file as illusrated below.");
    });

    // Loaded file top level.
    if let Some(texture) = &app.help_image_1 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("2.1 Initial View", |ui| {
        ui.label("At the top of the image above we see that the log file contains 3 trips.");
        ui.label("The trip is identified by the trip number, and the date and time of the start of the trip.");
        ui.label("Notice the small arrow at the start of each trip - this indicates that the trip has collapsed data associated with it.");
        ui.label("Clicking on the trip label will expand 1 level below.");
    });
    
    ui.collapsing("2.2 Program info and status", |ui| {
        ui.label("At the bottom of the screen, information about the file, and status of the processing is shown as illustrated below.");
        ui.label("Also in the bottom panel is the detected controller ID, and the firmware version running on the controller.");
        ui.label("Note that the controller ID and firmware version is only the first of each record encountered in the file.");
        ui.label("At the far right is the trip ID of the currently selected trip (if one is selected).");
    });

    // Loaded file info panel.
    if let Some(texture) = &app.help_image_2 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("2.3 Trip event listing", |ui| {
        ui.label("Clicking on a trip in the trip listing will expand the trip down one level to show the events in the trip as illustrated below.");
        ui.label("In the program info and status bar the trip number of the selected trip will be displayed.");
    });

    // Trip expanded to show events.
    if let Some(texture) = &app.help_image_3 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("2.4 Event details", |ui| {
        ui.label("Clicking on an event in the trip listing will expand the event down to show the details associated with the event as illustrated below.");
        ui.label("Note that selecting the trip again will collapse the event back one level.");
        ui.label("Note also that this applies to the trip, i.e. selecting an expanded trip will collapse the trip to the top level.");
        ui.label("Any number of trips and/or events can be expanded at a time as the window supports vertical scrolling as required.");
    });

    // Event expanded to show event details.
    if let Some(texture) = &app.help_image_4 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("2.5 Optional Trip Data", |ui| {
        ui.label("Some less common events can be shown in the trip events list by selecting options from the main menu.");
        ui.label("From the main menu check from the optional event types as illustrated below.");
    });

    // Optional events display.
    if let Some(texture) = &app.help_image_5 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("2.6 Trip events showing optional REPORT events", |ui| {
        ui.label("The following figure shows REPORT events included in the previous trip events list with showing report events enabled.");
    });

    // Optional report events displayed.
    if let Some(texture) = &app.help_image_6 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("2.7 Trip events showing GPS data", |ui| {
        ui.label("Most trip events include some GPS data in the event string.");
        ui.label("By checking the Show GPS data menu option GPS data is apended to the event data for each event as illustrated in below.");
        ui.label("Note that GPS data relies on a gps fix for data to be accurate or useful.");
    });

    // Optional gps data included with event data.
    if let Some(texture) = &app.help_image_7 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("2.8 Out of trip events", |ui| {
        ui.label("By checking the Show Out of Trip Event menu option the trip events list will include any (supported) events that occured out of trip, where out of trip means after an end of trip and before a start of trip.");
    });

    // Optional out of trip events.
    if let Some(texture) = &app.help_image_8 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("2.9 Out of trip event data", |ui| {
        ui.label("Out of trip events, if displayed, can be expanded to show event data as illustrated below.");
    });

    // Out of trip evet data.
    if let Some(texture) = &app.help_image_9 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("3.0 GPS breadcrumb plots", |ui| {
        ui.label("As described in early sections event data has collected gps data associated with it.");
        ui.label("As events are recorded in sequence, when plotted, the gps data is a breadcrumn trail of the machine in question.");
        ui.label("From the Plot menu the user can select to show the gps plot for the currently selected trip (if there is one selected).");
        ui.label("The Plot men allows the user to select a naked plot, or one with an Open Street Map (OSM) tile background as illustrated belwo.");
    });

    // GPS plot menu.
    if let Some(texture) = &app.help_image_10 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("3.1 Plain GPS plot", |ui| {
        ui.label("The standard option is no backgound to the plot as illustrated below.");
        ui.label("Note that the plain background will match the current light/dark application background setting.");
        ui.label("Note that plot displayed is for the currently selected machine if one is selected.");
    });

    // Plain GPS plot.
    if let Some(texture) = &app.help_image_11 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("3.2 Open Street Maps (SM) GPS plot", |ui| {
        ui.label("The gps plot with OSM menu item is selected the current or future gps plots will use an OSM background as illustrated below.");
        ui.label("Note that if a map is already displayed in plain mode when the OSM menu item is checked, the gps plot will automatically be updated to use the GPS background.");
        ui.label("Note that the gps breadcrumbs are colour coded to indicate the gps speed of the vehicle at the time - refer to the legend below the plot.");
});

    // OSM GPS plot.
    if let Some(texture) = &app.help_image_12 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("3.3 OSM GPS gps plot options", |ui| {
        ui.label("Built into the gps with OSM plots is the abilty to pan and zoom using the familiar mouse, or Ctrl mouse opens.");
        ui.label("Ilustrated in the following figure is the previous map that has been panned and zoomed as a demonstration.");
        ui.label("Note that an internet connection is required to plot with OSM backgrounds.");
        ui.label("Note that partial links between consecutive gps points is  only shown if both points are visible.");
        ui.label("Note that the gps breadcrumbs are colour coded to indicate the gps speed of the vehicle at the time - refer to the legend below the plot.");
    });

    // OSM GPS plot (pan and zoom).
    if let Some(texture) = &app.help_image_13 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("4.0 User settings", |ui| {
        ui.label("There are some program related settings that the user can select or use independent of the log scraping functionality.");
        ui.label("These include the dark/light program setting, and the font size of trip and event data information on the main display.");
        ui.label("These settings will be described in the following sections.");
    });

    ui.collapsing("4.1 Background settings", |ui| {
        ui.label("From the View menu, the user can select between light and dark mode as illustrated below.");
        ui.label("Note that menu selection is effectively a toggle button.");
        ui.label("Note also that this will change the background to all the application windows.");
    });

    // Light and dark mode setting.
    if let Some(texture) = &app.help_image_14 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("4.2 Trip and event list font size", |ui| {
        ui.label("There is an option to change the font size used for Trip and Event data.");
        ui.label("This is achieved via the 'settings.yml' file, an example of which is shown in the following example (actually the default values).");
    });

    // Trip and Event font size setting.
    if let Some(texture) = &app.help_image_15 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("Trip and event list font constraints", |ui| {
        ui.add_space(10.0);
        ui.label("Font sizes are limited to the range 12.0 to 20.0.");
        ui.add_space(10.0);
        ui.label("If the range limits above are exceeded the default values shown above will be used.");
        ui.label("Note that the width of the main screen can be varied within limits, but using too large a font may result in longer strings from being clipped.");
    });
}
