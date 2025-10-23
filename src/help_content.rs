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
        ui.label("The trips are identified by the trip number, and the date and time of the start of the trip.");
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
        ui.label("Some less common events can be shown in the trip events list by selecting options from the 'Show' menu item as illustrated below.");
    });

    // Optional events display.
    if let Some(texture) = &app.help_image_5 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("2.6 Trip events showing optional unsupported events", |ui| {
        ui.label("There may be some events that are not supported (although they may be supported in the future).");
        ui.label("Unsupported event titles will be shown in the trip/event listing in a special colour as illustrated in purple in the figure below.");
        ui.label("Note that for unsupported events there is no event attribute data included; other than gps data.");
    });

    // Optional unsupported events displayed.
    if let Some(texture) = &app.help_image_17 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("2.7 Trip events showing optional REPORT events", |ui| {
        ui.label("The following figure shows that REPORT events can be included with the other trip events by selecting 'Report events' from the 'Show' menu.");
    });

    // Optional report events displayed.
    if let Some(texture) = &app.help_image_6 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("2.8 Trip events showing GPS data", |ui| {
        ui.label("Most trip events include some GPS data in the event string.");
        ui.label("By checking the 'GPS event data' from the 'Show' menu option, GPS data is appended to the event data for each event as illustrated in the figure below.");
        ui.label("Note that GPS data relies on a gps fix for data to be accurate or useful.");
    });

    // Optional gps data included with event data.
    if let Some(texture) = &app.help_image_7 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("2.9 Out of trip events", |ui| {
        ui.label("By checking the 'Show Out of trip events' item from the 'Show' menu the trip events list will include any (supported) events that occured out of trip, where out of trip means after an end of trip or before a start of trip.");
    });

    // Optional out of trip events.
    if let Some(texture) = &app.help_image_8 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("2.10 Out of trip event data", |ui| {
        ui.label("Out of trip events, if displayed, can be expanded to show event data as illustrated below.");
    });

    // Out of trip evet data.
    if let Some(texture) = &app.help_image_9 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("3.0 GPS breadcrumb plots", |ui| {
        ui.label("As described in earlier sections event data has collected gps data associated with it.");
        ui.label("As events are recorded in sequence, when plotted, the gps data is a breadcrumb trail of the machine in question.");
        ui.label("From the Plot menu the user can select to show the gps plot for the currently selected trip (if there is one selected).");
        ui.label("The Plot menu allows the user to select a simple plot (no background), or one with an Open street (OSM), or ESRI satelitte view tile background as illustrated belwo.");
    });

    // GPS plot menu.
    if let Some(texture) = &app.help_image_10 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("3.1 Simple GPS plot", |ui| {
        ui.label("The standard option is no backgound to the plot as illustrated below.");
        ui.label("Note that the plain background will match the current light/dark application background setting.");
        ui.label("Note that the plot displayed is for the currently selected trip, if one is selected.");
    });

    // Plain GPS plot.
    if let Some(texture) = &app.help_image_11 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("3.2 Open Street Maps (OSM) GPS plot", |ui| {
        ui.label("If the 'Street View' option is selected from the 'Plot' menu the current and future gps plots will use a street view background as illustrated below.");
        ui.label("Note that if a map is already displayed in plain mode when the street view menu item is checked, the gps plot will automatically be updated to use the OSM street view background.");
        ui.label("Note that the gps breadcrumbs are colour coded to indicate the gps speed of the vehicle at the time - refer to the legend below the plot.");
    });

    // Street view GPS plot.
    if let Some(texture) = &app.help_image_12 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("3.3 Satelite Maps GPS plot", |ui| {
        ui.label("If the 'Satellite View' option is selected from the 'Plot' menu the current and future gps plots will use a satellite view background as illustrated below.");
        ui.label("Note that if a map is already displayed in plain mode when the satellite view menu item is checked, the gps plot will automatically be updated to use the ESRI satellite view background.");
        ui.label("Note that the gps breadcrumbs are colour coded to indicate the gps speed of the vehicle at the time - refer to the legend below the plot.");
    });

    // Satellite view GPS plot.
    if let Some(texture) = &app.help_image_13 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    // Satellite view GPS plot (pan and zoom).
    if let Some(texture) = &app.help_image_14 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("4.0 Time series data plots", |ui| {
        ui.label("As data is collected per trip (discounting out of trip events) it is possible to plot this time series data.");
        ui.label("From the Plot menu the user can select to show time series plots for the currently selected trip (if there is one selected).");
        ui.label("On selecting the Time Series Data option from the Plot menu a separate application window will be displayed.");
        ui.label("The window will be blank or complete with plots depending on whether or not there is a current trip selected.");
        ui.label("In the figure below a trip is already selected so time series events from the trip are displayed.");
        ui.label("\nNote that unsupported events are not displayed in the time series plots, and currently only digital, analog, and impulse events are plotted.");
    });

    // Time series data plotting.
    if let Some(texture) = &app.help_image_18 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("4.1 Time series data plot window scrolling", |ui| {
        ui.label("As illustrated in the figure above all the time series data is plotted, one event above the other.");
        ui.label("In the case above the window needs to be scrolled down to see the event plot at the bottom, as shown in the figure below.");
    });

    // Time series data window scrolling.
    if let Some(texture) = &app.help_image_19 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("4.2 Time series data Pan and Zoom", |ui| {
        ui.label("My selecting the Pan/Zoom button it is possible to pan and zoom the time series data.");
        ui.label("Note that panning and zooming can only be done in the X (time) direction.");
        ui.label("Note also the all plots will pan and zoom together, i.e. the time scale of the plots remain in alignment.");
        ui.label("Panning and zooming of the time series data is illustrated in the figure below.");
    });

    // Time series data pan and zoom.
    if let Some(texture) = &app.help_image_20 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("4.3 Cancelling Pan and Zoom", |ui| {
        ui.label("By pressing the Pan/Zoom button again, it will turn panning and zooming off, but the current state will remain.");
        ui.label("If desired, select the Reset View button to cancel the panning and zooming to return to the original state.");
    });

    ui.collapsing("4.4 Time series data Time Cursor", |ui| {
        ui.label("My selecting the Cursor button it is possible to dispay a time cursor on each of the time series plots.");
        ui.label("Click on a time series plot, and then move the cursor line to time left or right, doing so will move the cursor line on all plots together.");
        ui.label("At the bottom of each cursor line is the plot time corresponding to the time at that location.");
        ui.label("An example of using the time cursor is illustrated in the figure below.");
    });

    // Time series data time cursor.
    if let Some(texture) = &app.help_image_21 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("4.5 Cancelling the time cursor", |ui| {
        ui.label("By pressing the Cursor button again, it will remove the time cursor from all plots.");
    });

    ui.collapsing("5.0 User settings", |ui| {
        ui.label("There are some program related settings that the user can select or use independent of the log scraping functionality.");
        ui.label("These include the dark/light program setting, and the font size of trip and event data information, on the main display.");
        ui.label("These settings will be described in the following sections.");
    });

    ui.collapsing("5.1 Background settings", |ui| {
        ui.label("From the 'View' menu, the user can select between light and dark mode as illustrated below.");
        ui.label("Note that menu selection is effectively a toggle button.");
        ui.label("Note also that this will change the background to all the application windows.");
    });

    // Light and dark mode setting.
    if let Some(texture) = &app.help_image_15 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("5.2 Trip and event list font size", |ui| {
        ui.label("There is an option to change the font size used for Trip and Event data.");
        ui.label("This is achieved via the 'settings.yml' file, an example of which is shown in the following example (these are also the default values).");
    });

    // Trip and Event font size setting.
    if let Some(texture) = &app.help_image_16 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }

    ui.collapsing("Trip and event list font constraints", |ui| {
        ui.add_space(10.0);
        ui.label("Font sizes are limited to the range 12.0 to 20.0.");
        ui.add_space(10.0);
        ui.label("If the range limits above are exceeded the default values shown above will be used.");
        ui.label("Note that the width of the main screen can be varied within limits, but using too large a font may result in longer strings being clipped.");
    });

    ui.collapsing("5.3 Logging settings", |ui| {
        ui.label("There is various logging, at different levels that can be performed by the application.");
        ui.label("If there is no logging configuration file in the top level directory, a default one will be created on application start.");
        ui.label("As illustrated in the following figure the default configuration level is 'debug'.");
        ui.label("For logging at a different level change the level to 'info' or 'warn'.");
    });

    // Logging configuration file.
    if let Some(texture) = &app.help_image_22 {
        ui.add_space(10.0);
        ui.add(egui::Image::new(texture).max_width(400.0));
        ui.add_space(10.0);
    }
}
