use conrod::position::Align as ConrodAlign;
use serde::de::{Deserialize, Deserializer, Error, Visitor};

use std::fmt;
use std::ops::Deref;

/// Wrapper around `conrod::Align` that implements `Deserialize`
#[derive(Debug, Clone)]
pub struct Align(pub ConrodAlign);

impl Deref for Align {
    type Target = ConrodAlign;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Align {
    fn default() -> Align {
        Align(ConrodAlign::Start)
    }
}

static VARIANTS: [&'static str; 6] = ["start", "left", "middle", "center", "end", "right"];

impl Deserialize for Align {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct FieldVisitor;

        impl Visitor for FieldVisitor {
            type Value = Align;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid variant of Align")
            }

            fn visit_str<E>(self, value: &str) -> Result<Align, E>
                where E: Error
            {
                match &*value.to_lowercase() {
                        "start" | "left" => Ok(ConrodAlign::Start),
                        "middle" | "center" => Ok(ConrodAlign::Middle),
                        "end" | "right" => Ok(ConrodAlign::End),
                        _ => Err(Error::unknown_field(value, &VARIANTS)),
                    }
                    .map(Align)
            }
        }

        deserializer.deserialize_struct_field(FieldVisitor)
    }
}
