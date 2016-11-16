//! Providers for `Widget`s to be rendered by `ConrodInstance`
//!

use conrod::UiCell;
use wlc::Output;

pub mod background;
pub use self::background::{BackgroundConfig, BackgroundHandler};
pub mod statusbar;
pub use self::statusbar::{StatusbarConfig, StatusbarHandler};

/// An Interface for dealing with types, that want to render on a
/// `ConrodInstance`
pub trait ConrodProvider {
    /// Render all `Widget`s managed by this provider via the provided
    /// `Cell` with the given context by `output`.
    fn render(&mut self, output: &Output, ui: &mut UiCell);
}
