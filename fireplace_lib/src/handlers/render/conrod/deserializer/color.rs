use conrod::color::{self, Color as ConrodColor};
use serde::de::{Deserialize, Deserializer, Error, Visitor};

use std::fmt;
use std::ops::Deref;

/// Wrapper around `conrod::color::Color` that implements `Deserialize`
#[derive(Debug, Clone)]
pub struct Color(pub ConrodColor);

impl Deref for Color {
    type Target = ConrodColor;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Color {
    fn default() -> Color {
        Color(color::TRANSPARENT)
    }
}

static VARIANTS: [&'static str; 33] = ["BLACK",
                                       "BLUE",
                                       "BROWN",
                                       "CHARCOAL",
                                       "DARK_BLUE",
                                       "DARK_BROWN",
                                       "DARK_CHARCOAL",
                                       "DARK_GRAY",
                                       "DARK_GREEN",
                                       "DARK_GREY",
                                       "DARK_ORANGE",
                                       "DARK_PURPLE",
                                       "DARK_RED",
                                       "DARK_YELLOW",
                                       "GRAY",
                                       "GREEN",
                                       "GREY",
                                       "LIGHT_BLUE",
                                       "LIGHT_BROWN",
                                       "LIGHT_CHARCOAL",
                                       "LIGHT_GRAY",
                                       "LIGHT_GREEN",
                                       "LIGHT_GREY",
                                       "LIGHT_ORANGE",
                                       "LIGHT_PURPLE",
                                       "LIGHT_RED",
                                       "LIGHT_YELLOW",
                                       "ORANGE",
                                       "PURPLE",
                                       "RED",
                                       "TRANSPARENT",
                                       "WHITE",
                                       "YELLOW"];

impl Deserialize for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct FieldVisitor;

        impl Visitor for FieldVisitor {
            type Value = Color;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid variant of Color")
            }

            fn visit_str<E>(self, value: &str) -> Result<Color, E>
                where E: Error
            {
                match &*value.to_uppercase() {
                        "BLACK" => Ok(color::BLACK),
                        "BLUE" => Ok(color::BLUE),
                        "BROWN" => Ok(color::BROWN),
                        "CHARCOAL" => Ok(color::CHARCOAL),
                        "DARK_BLUE" => Ok(color::DARK_BLUE),
                        "DARK_BROWN" => Ok(color::DARK_BROWN),
                        "DARK_CHARCOAL" => Ok(color::DARK_CHARCOAL),
                        "DARK_GRAY" => Ok(color::DARK_GRAY),
                        "DARK_GREEN" => Ok(color::DARK_GREEN),
                        "DARK_GREY" => Ok(color::DARK_GREY),
                        "DARK_ORANGE" => Ok(color::DARK_ORANGE),
                        "DARK_PURPLE" => Ok(color::DARK_PURPLE),
                        "DARK_RED" => Ok(color::DARK_RED),
                        "DARK_YELLOW" => Ok(color::DARK_YELLOW),
                        "GRAY" => Ok(color::GRAY),
                        "GREEN" => Ok(color::GREEN),
                        "GREY" => Ok(color::GREY),
                        "LIGHT_BLUE" => Ok(color::LIGHT_BLUE),
                        "LIGHT_BROWN" => Ok(color::LIGHT_BROWN),
                        "LIGHT_CHARCOAL" => Ok(color::LIGHT_CHARCOAL),
                        "LIGHT_GRAY" => Ok(color::LIGHT_GRAY),
                        "LIGHT_GREEN" => Ok(color::LIGHT_GREEN),
                        "LIGHT_GREY" => Ok(color::LIGHT_GREY),
                        "LIGHT_ORANGE" => Ok(color::LIGHT_ORANGE),
                        "LIGHT_PURPLE" => Ok(color::LIGHT_PURPLE),
                        "LIGHT_RED" => Ok(color::LIGHT_RED),
                        "LIGHT_YELLOW" => Ok(color::LIGHT_YELLOW),
                        "ORANGE" => Ok(color::ORANGE),
                        "PURPLE" => Ok(color::PURPLE),
                        "RED" => Ok(color::RED),
                        "TRANSPARENT" => Ok(color::TRANSPARENT),
                        "WHITE" => Ok(color::WHITE),
                        "YELLOW" => Ok(color::YELLOW),
                        x if (x.len() == 6 || x.len() == 8) && u32::from_str_radix(x, 16).is_ok() => {
                            let mut color = String::from(value);
                            if x.len() == 6 {
                                color.push_str("FF");
                            }
                            let rgba = parse_hex_color(&color);
                            Ok(color::rgba(rgba[0], rgba[1], rgba[2], rgba[3]))
                        }
                        x if x.starts_with('#') && (x.len() == 7 || x.len() == 9) &&
                             u32::from_str_radix(&x[1..], 16).is_ok() => {
                            let mut color = String::from(&value[1..]);
                            if x.len() == 7 {
                                color.push_str("FF");
                            }
                            let rgba = parse_hex_color(&color);
                            Ok(color::rgba(rgba[0], rgba[1], rgba[2], rgba[3]))
                        }
                        _ => Err(Error::unknown_field(value, &VARIANTS)),
                    }
                    .map(Color)
            }
        }

        deserializer.deserialize_struct_field(FieldVisitor)
    }
}

pub fn parse_hex_color(color: &str) -> [f32; 4] {
    let value = u32::from_str_radix(color, 16).unwrap();
    let r: u8 = ((value >> 24) & 0xff) as u8;
    let g: u8 = ((value >> 16) & 0xff) as u8;
    let b: u8 = ((value >> 8) & 0xff) as u8;
    let a: u8 = (value & 0xff) as u8;
    [(r as f32 / 255.0), (g as f32 / 255.0), (b as f32 / 255.0), (a as f32 / 255.0)]
}
