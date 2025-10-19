// Helper functions for time series plots.

 use crate::time_series_plot::TimeSeriesData;
 use crate::time_series_plot::SinglePoint;

// Helper function to convert Unix timestamp to hh:mm:ss format.
pub fn unix_time_to_hms(unix_time: u64) -> String {
    // Convert Unix timestamp to time of day (UTC).
    let seconds_in_day = unix_time % 86400;
    let hours = seconds_in_day / 3600;
    let minutes = (seconds_in_day % 3600) / 60;
    let seconds = seconds_in_day % 60;
    
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

// Helper function to calculate the overall time range across all datasets.
pub fn calculate_time_range(datasets: &[TimeSeriesData]) -> (u64, u64) {
    let mut time_min = u64::MAX;
    let mut time_max = u64::MIN;
    
    for dataset in datasets {
        for point in &dataset.time_series_points {
            time_min = time_min.min(point.unix_time);
            time_max = time_max.max(point.unix_time);
        }
    }
    
    // If no data points, return reasonable defaults.
    if time_min == u64::MAX {
        (0, 1)
    } else {
        (time_min, time_max)
    }
}

// Helper function to calculate Y-axis range for a dataset.
pub fn calculate_y_range(dataset: &TimeSeriesData) -> (f32, f32) {
    if dataset.time_series_points.is_empty() {
        return (0.0, 1.0);
    }
    
    // Special handling for impulse signals.
    if dataset.data_type == "Impulse" {
        // Range comes from actual number of levels in the dataset.
        // There is also the baseline level added.
        if !dataset.levels.is_empty() {
            return (0.0, (dataset.levels.len() + 1) as f32);
        } else {
            // Fallback to default behavior if no levels defined.
            return (0.0, 4.0);
        }
    }
    
    // Special handling for dual digital signals.
    if dataset.data_type == "DualDigital" || dataset.data_type == "ImpulseDigitalCombo" {
        // Range must match how tick marks are calculated
        // Tick marks use: y_ratio = level_value / total_levels
        // So range should be 0 to total_levels.
        if !dataset.levels.is_empty() {
            // The range goes from 0.0 up to the number of levels.
            return (0.0, dataset.levels.len() as f32); 
        } else {
            return (0.0, 2.0);
        }
    }

    let mut y_min = f32::MAX;
    let mut y_max = f32::MIN;
    
    for point in &dataset.time_series_points {
        y_min = y_min.min(point.point_value);
        y_max = y_max.max(point.point_value);
    }
    
    // Add some padding for better visualization.
    let padding = (y_max - y_min) * 0.1;
    if padding == 0.0 {
        // For flat lines (like digital signals), add fixed padding.
        (y_min - 0.1, y_max + 0.1)
    } else {
        (y_min - padding, y_max + padding)
    }
}

// Helper function to convert single event points to pulse data.
pub fn convert_to_pulse_data(ev_points: &[SinglePoint], trip_start: u64, trip_end: u64, data_type: &str) -> Vec<SinglePoint> {
    let mut pulse_points = Vec::new();
    
    // Determine baseline value based on data type.
    let baseline_value = if data_type == "Impulse" {
        // Impulse signsl
        1.0
    } else {
        // Digital signal.
        0.0
    };
    
    // Add starting point at trip start (baseline level).
    pulse_points.push(SinglePoint {
        unix_time: trip_start,
        point_value: baseline_value,
    });
    
    // Convert each event point into a pulse (rising pulse).
    for point in ev_points {
        let event_end_time = point.unix_time;
        let duration_seconds = if data_type == "Digital" {
            // For digital signals, use the point value as duration.
            point.point_value as u64
        } else {
            // For impulse signals, we want instantaneous events, so use 0 duration.
            0
        };
        
        // Calculate when the event started (going back in time by duration).
        let event_start_time = if event_end_time >= duration_seconds {
            event_end_time - duration_seconds
        } else {
            // If duration is longer than time since trip start, use trip start.
            trip_start
        };
        
        // Add point just before signal changes (still at baseline).
        if event_start_time > trip_start {
            pulse_points.push(SinglePoint {
                unix_time: event_start_time,
                point_value: baseline_value,
            });
        }
        
        // Add point where signal becomes active (rising edge).
        pulse_points.push(SinglePoint {
            unix_time: event_start_time,
            point_value: if data_type == "Digital" { 1.0 } else { point.point_value },
        });
        
        // Add point where signal is active (constant at level).
        pulse_points.push(SinglePoint {
            unix_time: event_end_time,
            point_value: if data_type == "Digital" { 1.0 } else { point.point_value },
        });

        // Add point where signal returns to baseline (falling edge).
        pulse_points.push(SinglePoint {
            unix_time: event_end_time,
            point_value: baseline_value,
        });
    }
    
    // Add ending point at trip end (baseline level).
    pulse_points.push(SinglePoint {
        unix_time: trip_end,
        point_value: baseline_value,
    });
    
    pulse_points
}
