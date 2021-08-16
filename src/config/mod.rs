//! # Fireplace configuration
//!
use crate::{
    logger::Logging,
    handler::keyboard::KeyPattern,
};

use std::collections::HashMap;
use serde::Deserialize;

mod default;

/// Main configuration struct
///
/// Collects all configuration structs from the various handlers.
/// Can be deserialized by `serde`.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Logging configuration
    #[serde(default)]
    pub logging: Logging,
    /// A `HashMap` of global actions that may be invoked through keys.
    ///
    /// * terminate => End the compositor
    #[serde(default = "crate::config::default::keys")]
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
}

impl Default for Config {
    fn default() -> Config {
        Config {
            logging: Logging::default(),
            keys: default::keys(),
            view: View::default(),
            exec: Exec::default(),
            workspace: WorkspacesConfig::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
/// View related configuration options
pub struct View {
    /// A `HashMap` of global actions that may be invoked through keys.
    ///
    /// * close => Close the currently focused `View`
    #[serde(default = "crate::config::default::view_keys")]
    pub keys: HashMap<String, KeyPattern>,
}

impl Default for View {
    fn default() -> View {
        View { keys: default::view_keys() }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
/// Exec/Launcher related configuration options
pub struct Exec {
    /// A `HashMap` of commands that may be launched through keys.
    ///
    /// The command is executed in a shell context.
    /// Environment Variables passed to Fireplace are available.
    /// Executables available in `$PATH` do not need to be given in full path.
    #[serde(default = "crate::config::default::exec_keys")]
    pub keys: HashMap<String, KeyPattern>,
}

impl Default for Exec {
    fn default() -> Exec {
        Exec { keys: default::exec_keys() }
    }
}

/// Configuration for the `WorkspaceHandler`
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct WorkspacesConfig {
    /// Key configuration
    #[serde(default = "crate::config::default::workspace_keys")]
    pub keys: HashMap<String, KeyPattern>,
}

impl Default for WorkspacesConfig {
    fn default() -> WorkspacesConfig {
        WorkspacesConfig { keys: default::workspace_keys() }
    }
}