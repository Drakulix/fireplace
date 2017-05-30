//! Types to render a background via `conrod` on the compositor

use conrod::{Positionable, Sizeable, Widget};

use conrod::UiCell;
use conrod::color;
use conrod::image::{Id as ImageId, Map as ImageMap};
use conrod::widget::id::Id;
use conrod::widget::primitive::image::Image as ConrodImage;
use conrod::widget::primitive::shape::rectangle::Rectangle;
use handlers::UiConfig;
use handlers::render::conrod::ConrodRenderer;
use handlers::render::conrod::deserializer::{Color, Image};
use handlers::render::conrod::provider::ConrodProvider;

use handlers::store::Store;
use opengles_graphics::{Texture, TextureSettings};

use wlc::{Callback, Output};

/// Handler that initializes a `Background` provider for every created `Output`
///
/// ## Dependencies
///
/// - [`StoreHandler`](../../../../struct.StoreHandler.html)
/// - [`OutputConfigHandler`](../../../../struct.OutputConfigHandler.html)
/// - [`ConrodHandler`](../../struct.ConrodHandler.html)
///
#[derive(Default)]
pub struct BackgroundHandler;

/// Configuration for a `BackgroundHandler` describing what
/// kind of background shall be set.
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub enum BackgroundConfig {
    /// A solid color as backgound
    #[serde(rename = "color")]
    Color(Color),
    /// An image rendered as background
    #[serde(rename = "image")]
    Image(Image),
}

impl Default for BackgroundConfig {
    fn default() -> BackgroundConfig {
        BackgroundConfig::Color(Color(color::CHARCOAL))
    }
}

impl BackgroundHandler {
    /// Initialize a new `BackgroundHandler` from a given configuration
    pub fn new() -> BackgroundHandler {
        BackgroundHandler
    }
}

impl Callback for BackgroundHandler {
    fn output_context_created(&mut self, output: &Output) {
        if let Some(lock) = output.get::<UiConfig>() {
            let conf = lock.read().unwrap();
            if let Some(lock) = output.get::<ConrodRenderer>() {
                let mut ui = lock.write().unwrap();
                let id = ui.background.widget_id_generator().next();
                let background = Background::new(id,
                                                 (*conf).background.clone(),
                                                 &mut ui.background.image_map());
                ui.background.register(background);
            }
        }
    }
}

/// A provider rendering a desktop background
pub struct Background {
    id: Id,
    tex_id: Option<ImageId>,
    data: BackgroundConfig,
}

impl Background {
    fn new(id: Id, data: BackgroundConfig, image_map: &mut ImageMap<Texture>) -> Background {

        let tex_id = if let BackgroundConfig::Image(ref rgba) = data {
            let texture = Texture::from_image(&**rgba, &TextureSettings::new());
            Some(image_map.insert(texture))
        } else {
            None
        };

        Background {
            id: id,
            tex_id: tex_id,
            data: data,
        }
    }
}

impl ConrodProvider for Background {
    fn render(&mut self, _output: &Output, ui: &mut UiCell) {
        match self.data {
            BackgroundConfig::Color(ref color) => {
                Rectangle::fill_with(ui.window_dim(), **color)
                    .xy([0.0, 0.0])
                    .set(self.id, ui);
            }
            BackgroundConfig::Image(_) => {
                ConrodImage::new(self.tex_id.unwrap())
                    .xy([0.0, 0.0])
                    .wh(ui.window_dim())
                    .set(self.id, ui);
            }
        }
    }
}
