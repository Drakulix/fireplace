//! Collection of `Deserialize` implementations for `conrod`
//! and related data types

mod image;
mod color;
mod align;
mod font;
mod justify;

pub use self::align::Align;
pub use self::color::Color;
pub use self::font::Font;
pub use self::image::Image;
pub use self::justify::Justify;
