use image::{self, RgbaImage};
use serde::de::{Deserialize, Deserializer, Error, Unexpected, Visitor};

use std::fmt;
use std::ops::Deref;

/// Wrapper around `image::RgbaImage` that implements `Deserialize`
#[derive(Clone)]
pub struct Image(pub RgbaImage);

impl Deref for Image {
    type Target = RgbaImage;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deserialize for Image {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct FieldVisitor;

        impl Visitor for FieldVisitor {
            type Value = Image;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid path containing an image file")
            }

            fn visit_str<E>(self, value: &str) -> Result<Image, E>
                where E: Error
            {
                image::open(value)
                    .map(|x| Image(x.to_rgba()))
                    .map_err(|_| Error::invalid_value(Unexpected::Str(value), &self))
            }
        }
        deserializer.deserialize_struct_field(FieldVisitor)
    }
}
