use font_loader::system_fonts::{FontProperty, FontPropertyBuilder};

/// Structure describing a `Font`
///
/// Implements `Deserialize`
#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct Font {
    /// Font family if desired
    #[serde(default)]
    pub family: Option<String>,
    /// If the selected font shall be monospaced
    #[serde(default)]
    pub monospace: bool,
    /// If the selected font shall be italic
    #[serde(default)]
    pub italic: bool,
    /// If the selected font shall be oblique
    #[serde(default)]
    pub oblique: bool,
    /// If the selected font shall be bold
    #[serde(default)]
    pub bold: bool,
}

impl Font {
    /// Return a `FontProperty` for loading the described `Font`
    pub fn property(&self) -> FontProperty {
        let mut builder = FontPropertyBuilder::new();
        if let Some(ref family) = self.family {
            builder = builder.family(family);
        }
        if self.monospace {
            builder = builder.monospace();
        }
        if self.italic {
            builder = builder.italic();
        }
        if self.oblique {
            builder = builder.oblique();
        }
        if self.bold {
            builder = builder.bold();
        }
        builder.build()
    }
}
