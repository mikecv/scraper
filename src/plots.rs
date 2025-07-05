// plots.rs
use log::info;
use chrono::{DateTime, NaiveDateTime, Utc, ParseError};
use crate::scraper::{Scraper, ScrapedData};

// Move PlotPoint here and make it public
#[derive(Debug, Clone)]
pub struct PlotPoint {
    pub timestamp: DateTime<Utc>,
    pub trip_num: String,
    pub lat: f64,
    pub lon: f64,
    pub speed: u32,
    pub rssi: u32,
}

impl From<&ScrapedData> for PlotPoint {
    fn from(data: &ScrapedData) -> Self {
        Self {
            timestamp: parse_datetime(&data.date_time).unwrap(),
            trip_num : data.trip_num.clone(),
            lat: data.gps_locn.lat,
            lon: data.gps_locn.long,
            speed: data.gps_speed,
            rssi: data.gps_rssi,
        }
    }
}

// Make this public so it can be used by the From implementation.
pub fn parse_datetime(date_str: &str) -> Result<DateTime<Utc>, ParseError> {
    let naive = NaiveDateTime::parse_from_str(date_str, "%d/%m/%Y %H:%M:%S")?;
    Ok(naive.and_utc())
}

// Function to plot gps data.
pub fn plot_gps_data(scraper: &Scraper, selected_id: &Option<String>) {
    info!("Initiating GPS plotting.");

    // Check to see if there is a current trip selectd.
    if selected_id.as_deref() == Some(&"") {
       info!("No trip selected.");
    }
    else {
        // Get the trip number selected by the user.
        let selected_trip = selected_id.as_ref().unwrap();
        info!("Selected trip number: {:}", selected_trip);

        // Get all the plotting points.
        // Filter out bad gps points, i.e lat and long = 0;
        let plot_points: Vec<PlotPoint> = scraper.scrapings.iter()
            .filter(|scraped| scraped.trip_num == *selected_trip)
            .filter(|scraped| scraped.gps_locn.lat != 0.0 && scraped.gps_locn.long != 0.0)
            .map(PlotPoint::from)
            .collect();
        info!("Number of plot points in trip: {:}", plot_points.len());
    }
}

