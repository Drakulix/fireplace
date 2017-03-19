//! Collection of different `Mode`s
//!
//! `Mode`s handle how `View`s on an `Output` or `Workspace` are layouted
//! and behave. They handle positioning, sizing, reacting to `View`'s requests
//! or may choose to ignore them.
//!
//! They can be used individually as handlers, but are normally indirectly
//! created and managed by workspaces via their matching configurations.
//!

use callback::{AsWrapper, IntoCallback, Wrapper};
use wlc::Callback;

pub mod bsp;
pub use self::bsp::{BSP, BSPConfig, BSPWrap, KeyPatterns as BSPKeyPatterns};

mod combined;
pub use self::combined::{Combined, CombinedConfig, CombinedWrap, KeyPatterns as CombinedKeyPatterns,
                         Predicate as CombinedPredicate};

mod floating;
pub use self::floating::*;

mod switch;
pub use self::switch::{KeyPatterns as SwitchKeyPatterns, Switch, SwitchConfig, SwitchWrap};

mod fullscreen;
pub use self::fullscreen::{Fullscreen, FullscreenConfig};
pub use self::fullscreen::{FullscreenWrap, KeyPatterns as FullscreenKeyPatterns};

/// Common interface for `Mode`s mostly used by workspaces
pub trait Mode: Callback {
    /// Arguments to the `new` constructor
    type Arguments;
    /// Common constructor for every `Mode`
    fn new(arguments: Self::Arguments) -> Self;
    /// Number of `View`s managed
    fn len(&self) -> usize;
    /// If this `Mode` currently manages no `View`s
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Enumeration of all possible `Mode`s, makes storing any mode much easier
#[cfg_attr(feature = "cargo-clippy", allow(large_enum_variant))]
pub enum AnyMode {
    /// Bsp `Mode`
    Bsp(bsp::BSPWrap),
    /// Floating `Mode`
    Floating(floating::Floating),
    /// Combined `Mode`
    Combined(combined::CombinedWrap),
    /// Switch `Mode`
    Switch(switch::SwitchWrap),
    /// Fullscreen `Mode`
    Fullscreen(fullscreen::FullscreenWrap),
}

/// Configuration for `AnyMode`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[cfg_attr(feature = "cargo-clippy", allow(large_enum_variant))]
pub enum AnyModeConfig {
    /// Configuration for Bsp `Mode`
    #[serde(rename = "bsp")]
    Bsp(bsp::BSPConfig),
    /// Configuration for Floating `Mode`
    #[serde(rename = "floating")]
    Floating(floating::FloatingConfig),
    /// Configuration for Combined `Mode`
    #[serde(rename = "combined")]
    Combined(combined::CombinedConfig),
    /// Configuration for Switch `Mode`
    #[serde(rename = "switch")]
    Switch(switch::SwitchConfig),
    /// Configuration for Fullscreen `Mode`
    #[serde(rename = "fullscreen")]
    Fullscreen(fullscreen::FullscreenConfig),
}

impl Default for AnyModeConfig {
    fn default() -> AnyModeConfig {
        AnyModeConfig::Fullscreen(fullscreen::FullscreenConfig {
            mode: Box::new(AnyModeConfig::Floating(floating::FloatingConfig::default())),
            keys: fullscreen::KeyPatterns { toggle: None },
        })
    }
}

impl AsWrapper for AnyMode {
    fn child(&mut self) -> Option<&mut Callback> {
        Some(match *self {
                 AnyMode::Bsp(ref mut mode) => mode as &mut Callback,
                 AnyMode::Floating(ref mut mode) => mode as &mut Callback,
                 AnyMode::Combined(ref mut mode) => mode as &mut Callback,
                 AnyMode::Switch(ref mut mode) => mode as &mut Callback,
                 AnyMode::Fullscreen(ref mut mode) => mode as &mut Callback,
             })
    }
}

/// Convinient type alias for a wrapped `AnyMode`
pub type AnyModeWrap = Wrapper<AnyMode>;

impl Mode for Wrapper<AnyMode> {
    type Arguments = AnyModeConfig;

    fn new(arguments: Self::Arguments) -> Self {
        match arguments {
                AnyModeConfig::Bsp(config) => AnyMode::Bsp(bsp::BSPWrap::new(config)),
                AnyModeConfig::Floating(config) => AnyMode::Floating(floating::Floating::new(config)),
                AnyModeConfig::Combined(config) => AnyMode::Combined(combined::CombinedWrap::new(config)),
                AnyModeConfig::Switch(config) => AnyMode::Switch(switch::SwitchWrap::new(config)),
                AnyModeConfig::Fullscreen(config) => {
                    AnyMode::Fullscreen(fullscreen::FullscreenWrap::new(config))
                }
            }
            .into_callback()
    }

    fn len(&self) -> usize {
        match **self {
            AnyMode::Bsp(ref mode) => mode.len(),
            AnyMode::Floating(ref mode) => mode.len(),
            AnyMode::Combined(ref mode) => mode.len(),
            AnyMode::Switch(ref mode) => mode.len(),
            AnyMode::Fullscreen(ref mode) => mode.len(),
        }
    }
}
