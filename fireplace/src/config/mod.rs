//! # Fireplace configuration
//!

use fireplace_lib::handlers::FocusConfig;
use fireplace_lib::handlers::OutputConfig;
use fireplace_lib::handlers::geometry::GapsConfig;
use fireplace_lib::handlers::keyboard::KeyPattern;
#[cfg(feature = "ui")]
use fireplace_lib::handlers::render::conrod::provider::BackgroundConfig;
#[cfg(feature = "ui")]
use fireplace_lib::handlers::render::conrod::provider::StatusbarConfig;
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
    /// Ui related configuration
    #[serde(default)]
    pub ui: Ui,
}

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
            ui: Ui::default(),
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

/// Ui Configuration mostly only available by enabling the `ui` feature
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Ui {
    /// The kind of background to be drawn
    #[serde(default)]
    #[cfg(feature = "ui")]
    pub background: BackgroundConfig,
    /// Configuration of gaps between windows
    #[serde(default)]
    pub gaps: GapsConfig,
    /// Statusbar configuraion
    #[serde(default)]
    #[cfg(feature = "ui")]
    pub statusbar: StatusbarConfig,
}
