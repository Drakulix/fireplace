use conrod::text::Justify as ConrodJustify;
use serde::de::{Deserialize, Deserializer, Error, Visitor};

use std::fmt;
use std::ops::Deref;

/// Wrapper around `conrod::Justify` that implements `Deserialize`
#[derive(Debug, Clone)]
pub struct Justify(pub ConrodJustify);

impl Deref for Justify {
    type Target = ConrodJustify;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Justify {
    fn default() -> Justify {
        Justify(ConrodJustify::Left)
    }
}

static VARIANTS: [&'static str; 6] = ["left", "status", "center", "middle", "right", "end"];

impl Deserialize for Justify {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct FieldVisitor;

        impl Visitor for FieldVisitor {
            type Value = Justify;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid variant of Justify")
            }

            fn visit_str<E>(self, value: &str) -> Result<Justify, E>
                where E: Error
            {
                match &*value.to_lowercase() {
                        "start" | "left" => Ok(ConrodJustify::Left),
                        "middle" | "center" => Ok(ConrodJustify::Center),
                        "end" | "right" => Ok(ConrodJustify::Right),
                        _ => Err(Error::unknown_field(value, &VARIANTS)),
                    }
                    .map(Justify)
            }
        }

        deserializer.deserialize_struct_field(FieldVisitor)
    }
}
