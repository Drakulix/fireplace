//! Collection of common Keyboard related types.
//!
//! The `KeyboardHandler` and `KeyHandler` may be used for global
//! key handling, while the `KeyPattern` just describes a common
//! structure of a key action, that may be handled by the compositor.
//! The `KeyPattern` is therefor often used in other handlers reacting
//! to key presses.
//!

use slog;
use slog_scope;

use std::collections::HashMap;
use wlc::{Callback, Key, KeyState, Modifier, Modifiers, View};

/// Describtion of a key combination that might be
/// handled by the compositor.
///
/// Can be matched against arguments of `keyboard_key`
/// of the `Callback` trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyPattern {
    /// If the Pattern should match if the Key was pressed
    /// or when it gets released
    #[serde(default = "::handlers::keyboard::default_state")]
    pub state: KeyState,
    /// What modifiers are expected to be pressed alongside the key
    pub modifiers: Modifier::Flags,
    /// The actual key, that was pressed
    pub key: Key,
}

fn default_state() -> KeyState {
    KeyState::Pressed
}

impl KeyPattern {
    /// Create a new KeyPattern
    pub fn new(state: KeyState, modifiers: Modifier::Flags, key: Key) -> KeyPattern {
        KeyPattern {
            state: state,
            modifiers: modifiers,
            key: key,
        }
    }
}

/// Implement to express you may handle a certain key event
///
/// Note that this trait makes no assumtions about the actual
/// `KeyPattern` matched. It only expresses a single action that may
/// may be started.
///
/// An Implementation for `FnMut` is provided
pub trait KeyHandler {
    /// Handle an event
    ///
    /// ## Arguments
    ///
    /// * time - A value only meaningful in context to other `time` values, the
    /// larger the later an event has happened.
    /// * view - The currently focused `View` if any.
    fn handle_key(&mut self, time: u32, view: Option<&View>);
}

impl<F> KeyHandler for F
    where F: FnMut(u32, Option<&View>)
{
    fn handle_key(&mut self, time: u32, view: Option<&View>) {
        self(time, view)
    }
}

/// Handler to handle global key events
///
/// With this handler you can register custom key events, that have
/// no actual meaning to other handlers or components of the window
/// manager itself, but are helping the user to interact.
///
/// Examples can be both
///
/// - window related functionality like, closing a window.
/// - or external events like spawning a new terminal.
///
/// ## Dependencies
///
/// This handler has no dependencies.
///
pub struct KeyboardHandler {
    global: HashMap<KeyPattern, Box<KeyHandler>>,
    logger: slog::Logger,
}

impl Default for KeyboardHandler {
    fn default() -> KeyboardHandler {
        KeyboardHandler::new()
    }
}

impl KeyboardHandler {
    /// Initialize a new `KeyboardHandler`
    pub fn new() -> KeyboardHandler {
        KeyboardHandler {
            global: HashMap::new(),
            logger: slog_scope::logger().new(o!("instance" => "KeyboardHandler")),
        }
    }

    /// Register a new `KeyPattern` and `KeyHandler` combination,
    /// that triggers the `KeyHandler` once the corresponding
    /// `KeyPattern` was encountered.
    pub fn register<H: KeyHandler + 'static>(&mut self, pattern: KeyPattern, handler: H) {
        let contains = self.global.contains_key(&pattern);
        if contains {
            error!(self.logger,
                   "KeyPattern ({:?}) is already registered. Not registering",
                   pattern);
        } else {
            self.global.insert(pattern, Box::new(handler));
        }
    }
}

impl Callback for KeyboardHandler {
    fn keyboard_key(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, key: Key,
                    state: KeyState)
                    -> bool {
        let pattern = KeyPattern::new(state, modifiers.mods, key);
        if let Some(handler) = self.global.get_mut(&pattern) {
            debug!(self.logger, "Matching KeyPattern ({:?})", pattern);

            handler.handle_key(time, view);
            true
        } else {
            false
        }
    }
}
