//! Utility functions regarding different coordinate systems used in fireplace

/// Convert a wlc x coordinate to its conrod counterpart
#[cfg(feature = "conrod_ui")]
pub fn x_wlc_to_conrod(x: i32, width: u32, total_width: u32) -> f64 {
    x as f64 + (width as f64 / 2.0) - (total_width as f64 / 2.0)
}

/// Convert a wlc y coordinate to its conrod counterpart
#[cfg(feature = "conrod_ui")]
pub fn y_wlc_to_conrod(y: i32, height: u32, total_height: u32) -> f64 {
    (total_height as f64 - y as f64) - (height as f64 / 2.0) - (total_height as f64 / 2.0)
}
