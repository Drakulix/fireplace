use slog;
use slog_scope;

use std::collections::HashMap;

use wlc::{Callback, Output};

/// Handler to apply output configurations to every newly created
/// `Output` automatically.
///
/// ## Dependencies
///
/// This handler has no dependencies.
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
}

impl Default for OutputConfig {
    fn default() -> OutputConfig {
        OutputConfig { scale: 1 }
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
        } else if let Some(config) = self.outputs.get("default") {
            info!(self.logger,
                  "Setting scale {:?} for output {:?}",
                  config.scale,
                  output.name());
            let res = output.resolution();
            output.set_resolution(res, config.scale);
        } else if let Some(config) = self.outputs.get("generic") {
            info!(self.logger,
                  "Setting scale {:?} for output {:?}",
                  config.scale,
                  output.name());
            let res = output.resolution();
            output.set_resolution(res, config.scale);
        }
        output.set_sleeping(false);
        true
    }
}
