// Draw the time series data plots to a separate UI.

use log::debug;

use eframe::egui;

use crate::scraper::{Scraper, ScrapedData};

// SinglePoint struct.
#[derive(Debug, Clone)]
pub struct SinglePoint {
    pub unix_time: u64,
    pub point_value: f32,
}

// TimeSeriesData struct.
#[derive(Debug, Clone)]
pub struct TimeSeriesData {
    pub series_name: String,
    pub units: String,
    pub time_series_points: Vec<SinglePoint>,
}

// SinglePoint struct instantiated from scraped data.
impl From<&ScrapedData> for SinglePoint {
    fn from(data: &ScrapedData) -> Self {
        Self {
            unix_time: data.unix_time,
            point_value: data.gps_speed as f32,
        }
    }
}

// Function to plot time series data using custom drawing.
pub fn plot_time_series_data(ui: &mut egui::Ui, scraper: &Scraper, selected_id: &Option<String>) {

    // Get id of selected trip, or show prompt if no trip selected.
    let selected_trip = match selected_id.as_ref() {
        Some(id) if !id.is_empty() => id,
        _ => {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("Please select a trip to plot time series data.");
            });
            return;
        }
    };

    // Get all the plotting points.
    // Ultimately I want to scan scrapped data for different event types that can be time series plotted.
    // And then construct are list of time series plots (see struct TimeSeriesData).
    // For now we can hard code speed time series points

    let plot_points: Vec<SinglePoint> = scraper.scrapings.iter()
        .filter(|scraped| scraped.trip_num == *selected_trip)
        .map(SinglePoint::from)
        .collect();

    // Check if no points to plot.
    if plot_points.is_empty() {
        ui.label("No valid time series points found for this trip.");
        return;
    }

    // Create a speed time series set of data.
    // Instantiate the struct.
    let mut speed_series = TimeSeriesData {
        series_name: "Speed".to_string(),
        units: "Volts".to_string(),
        time_series_points: vec![],
    };

    // Populate with time series speed data.
    for point in plot_points.iter() {
        speed_series.time_series_points.push(point.clone());
    }

    // Time series plotting can go here.
    // If we get this to work can make set of time series data sets for plotting.
    // The plots should be stacked vertically with a common time bottom axis.
    // Each time series plot will be stacked vertically, and will use the bottom time axis.
    // Ideally panning and zooming if available will be applied to all of the stacked plots.

    // Temporary check of time series points.
    ui.horizontal(|ui| {
        ui.label("Time series points:");
        ui.strong(format!("{}", plot_points.len()));
    });
}
