//! # Fireplace configuration
//!

use fireplace_lib::handlers::FocusConfig;
use fireplace_lib::handlers::OutputConfig;
use fireplace_lib::handlers::keyboard::KeyPattern;
#[cfg(feature = "ui")]
use fireplace_lib::handlers::render::screenshot::ScreenshotConfig;
use fireplace_lib::handlers::workspaces::WorkspacesConfig;

use logger::Logging;

use std::collections::HashMap;

mod default;

/// Main configuration struct
///
/// Collects all configuration structs from the various handlers.
/// Can be deserialized by `serde`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// A `HashMap` of an `OutputConfig` for each name of an `Output`.
    /// The name `default` or `generic` might be used for a general
    /// configuration.
    #[serde(default = "::config::default::outputs")]
    pub outputs: HashMap<String, OutputConfig>,
    /// Logging configuration
    #[serde(default)]
    pub logging: Logging,
    /// A `HashMap` of global actions that may be invoked through keys.
    ///
    /// * terminate => End the compositor
    #[serde(default = "::config::default::keys")]
    pub keys: HashMap<String, KeyPattern>,
    /// Configuration of keys related to `View`s
    #[serde(default)]
    pub view: View,
    /// Configuration of program execution by keys.
    #[serde(default)]
    pub exec: Exec,
    /// Configuration for Workspaces
    #[serde(default)]
    pub workspace: WorkspacesConfig,
    /// Configuration for `View` focusing
    #[serde(default)]
    pub focus: FocusConfig,
    /// Configuration for Screenshots
    #[serde(default)]
    #[cfg(feature = "ui")]
    pub screenshot: ScreenshotConfig,
}

#[cfg(feature = "ui")]
impl Default for Config {
    fn default() -> Config {
        Config {
            outputs: default::outputs(),
            keys: default::keys(),
            logging: Logging::default(),
            view: View::default(),
            exec: Exec::default(),
            workspace: WorkspacesConfig::default(),
            focus: FocusConfig::default(),
            screenshot: ScreenshotConfig::default(),
        }
    }
}

#[cfg(not(feature = "ui"))]
impl Default for Config {
    fn default() -> Config {
        Config {
            outputs: default::outputs(),
            keys: default::keys(),
            logging: Logging::default(),
            view: View::default(),
            exec: Exec::default(),
            workspace: WorkspacesConfig::default(),
            focus: FocusConfig::default(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
/// View related configuration options
pub struct View {
    /// A `HashMap` of global actions that may be invoked through keys.
    ///
    /// * close => Close the currently focused `View`
    #[serde(default = "::config::default::view_keys")]
    pub keys: HashMap<String, KeyPattern>,
}

impl Default for View {
    fn default() -> View {
        View { keys: default::view_keys() }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
/// Exec/Launcher related configuration options
pub struct Exec {
    /// A `HashMap` of commands that may be launched through keys.
    ///
    /// The command is executed in a shell context.
    /// Environment Variables passed to Fireplace are available.
    /// Executables available in `$PATH` do not need to be given in full path.
    #[serde(default = "::config::default::exec_keys")]
    pub keys: HashMap<String, KeyPattern>,
}

impl Default for Exec {
    fn default() -> Exec {
        Exec { keys: default::exec_keys() }
    }
}
