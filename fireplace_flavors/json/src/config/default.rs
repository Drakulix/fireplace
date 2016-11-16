use fireplace_lib::handlers::OutputConfig;
use fireplace_lib::handlers::keyboard::KeyPattern;

use std::collections::HashMap;
use wlc::{Key, KeyState, Modifier};

pub fn outputs() -> HashMap<String, OutputConfig> {
    let mut map = HashMap::new();
    map.insert(String::from("default"), OutputConfig::default());
    map
}

pub fn keys() -> HashMap<String, KeyPattern> {
    let mut map = HashMap::new();
    map.insert(String::from("terminate"),
               KeyPattern::new(KeyState::Pressed,
                               Modifier::Logo | Modifier::Shift,
                               Key::Esc));
    map
}

pub fn view_keys() -> HashMap<String, KeyPattern> {
    let mut map = HashMap::new();
    map.insert(String::from("close"),
               KeyPattern::new(KeyState::Pressed, Modifier::Logo | Modifier::Shift, Key::Q));
    map
}

pub fn exec_keys() -> HashMap<String, KeyPattern> {
    let mut map = HashMap::new();
    map.insert(String::from("$TERMINAL"),
               KeyPattern::new(KeyState::Pressed, Modifier::Logo, Key::Enter));
    map
}
