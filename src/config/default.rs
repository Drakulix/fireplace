use crate::handler::keyboard::{KeyPattern, KeyState, KeyModifier, KeySyms};

use std::collections::HashMap;

pub fn keys() -> HashMap<String, KeyPattern> {
    let mut map = HashMap::new();
    map.insert(String::from("terminate"),
               KeyPattern::new(KeyState::Pressed,
                               KeyModifier::Logo | KeyModifier::Shift,
                               KeySyms::KEY_Escape));
    map
}

pub fn view_keys() -> HashMap<String, KeyPattern> {
    let mut map = HashMap::new();
    map.insert(String::from("close"),
               KeyPattern::new(KeyState::Pressed, KeyModifier::Logo | KeyModifier::Shift, KeySyms::KEY_Q));
    map
}

pub fn exec_keys() -> HashMap<String, KeyPattern> {
    let mut map = HashMap::new();
    map.insert(String::from("$TERMINAL"),
               KeyPattern::new(KeyState::Pressed, KeyModifier::Logo, KeySyms::KEY_Return));
    map
}
