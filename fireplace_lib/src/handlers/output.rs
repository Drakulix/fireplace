

use handlers::geometry::GapsConfig;
#[cfg(feature = "conrod_ui")]
use handlers::render::conrod::provider::BackgroundConfig;
#[cfg(feature = "conrod_ui")]
use handlers::render::conrod::provider::StatusbarConfig;
use handlers::store::{Store, StoreKey};
use slog;
use slog_scope;

use std::collections::HashMap;

use wlc::{Callback, Output};

/// Handler to apply output configurations to every newly created
/// `Output` automatically.
///
/// ## Dependencies
///
/// - [`StoreHandler`](./struct.StoreHandler.html)
///
pub struct OutputConfigHandler {
    outputs: HashMap<String, OutputConfig>,
    logger: slog::Logger,
}

/// Output configuration
///
/// Contains properties that may be applied for an `Output`
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputConfig {
    /// Scale of the `Output`
    pub scale: u32,
    /// Ui Configuration of the `Output`
    pub ui: UiConfig,
}

/// Ui Configuration mostly only available by enabling the `ui` feature
#[derive(Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct UiConfig {
    /// The kind of background to be drawn
    #[serde(default)]
    #[cfg(feature = "conrod_ui")]
    pub background: BackgroundConfig,
    /// Configuration of gaps between windows
    #[serde(default)]
    pub gaps: GapsConfig,
    /// Statusbar configuraion
    #[serde(default)]
    #[cfg(feature = "conrod_ui")]
    pub statusbar: StatusbarConfig,
}
impl StoreKey for UiConfig {
    type Value = UiConfig;
}

impl Default for OutputConfig {
    fn default() -> OutputConfig {
        OutputConfig {
            scale: 1,
            ui: UiConfig::default(),
        }
    }
}

impl OutputConfigHandler {
    /// Initialize a new `OutputConfigHandler`
    ///
    /// # Arguments
    /// *outputs - A `HashMap` of an `OutputConfig` for each name of
    /// an `Output`. The name `default` or `generic` might be used for a
    /// generic configuration.
    pub fn new(outputs: HashMap<String, OutputConfig>) -> OutputConfigHandler {
        OutputConfigHandler {
            outputs: outputs,
            logger: slog_scope::logger().new(o!("handler" => "OutputConfig")),
        }
    }
}

impl Callback for OutputConfigHandler {
    fn output_created(&mut self, output: &Output) -> bool {
        if let Some(config) = self.outputs.get(&*output.name()) {
            info!(self.logger,
                  "Setting scale {:?} for output {:?}",
                  config.scale,
                  output.name());
            let res = output.resolution();
            output.set_resolution(res, config.scale);
            output.insert::<UiConfig>(config.ui.clone());
        } else if let Some(config) = self.outputs.get("default") {
            info!(self.logger,
                  "Setting scale {:?} for output {:?}",
                  config.scale,
                  output.name());
            let res = output.resolution();
            output.set_resolution(res, config.scale);
            output.insert::<UiConfig>(config.ui.clone());
        } else if let Some(config) = self.outputs.get("generic") {
            info!(self.logger,
                  "Setting scale {:?} for output {:?}",
                  config.scale,
                  output.name());
            let res = output.resolution();
            output.set_resolution(res, config.scale);
            output.insert::<UiConfig>(config.ui.clone());
        }
        output.set_sleeping(false);
        true
    }
}
