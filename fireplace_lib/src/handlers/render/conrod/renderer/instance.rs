use conrod::{Dimensions, Ui, UiBuilder};
use conrod::backend;
use conrod::image::Map as ImageMap;
use conrod::text::GlyphCache;
use conrod::text::rt::Rect;

use graphics::Viewport;

use handlers::render::conrod::provider::ConrodProvider;
use handlers::store::Store;
use opengles_graphics::{GlGraphics, Texture};

use slog;

use std::ops::{Deref, DerefMut};
use texture::{Format, TextureSettings, UpdateTexture};

use wlc::render::RenderOutput;

/// Instance to render `ConrodProvider`s
///
/// `ConrodInstance` manages `ConrodProvider`s as well as the
/// concrete `conrod` types used for rendering, like `Ui`, which
/// it also dereferences to for rendering.
pub struct ConrodInstance {
    /// conrod `Ui` used for rendering
    pub ui: Ui,
    /// GL ES `Texture` used for caching rendered glyphs
    pub text_tex: Texture,
    /// Glyph cache for `conrod` rendering
    pub text_cache: GlyphCache,
    /// Map loading images as `Texture`s to be used by `conrod` via their `Id`s
    pub image_map: ImageMap<Texture>,
    provider: Vec<Box<ConrodProvider>>,
    logger: slog::Logger,
}

impl ConrodInstance {
    /// Initializes a new `ConrodInstance`
    pub fn new(dim: Dimensions, logger: slog::Logger) -> ConrodInstance {
        let len = (dim[0] * dim[1] * 4.0) as usize;
        let mut empty: Vec<u8> = Vec::with_capacity(len);
        for _ in 0..len {
            empty.push(0u8);
        }

        ConrodInstance {
            ui: UiBuilder::new(dim).build(),
            text_tex: Texture::from_memory_alpha(&empty,
                                                 dim[0] as u32,
                                                 dim[1] as u32,
                                                 &TextureSettings::new())
                .unwrap(),
            text_cache: GlyphCache::new(dim[0] as u32, dim[1] as u32, 0.1, 0.1),
            image_map: ImageMap::new(),
            provider: Vec::new(),
            logger: logger,
        }
    }

    /// Register a new `ConrodProvider` to be rendered
    pub fn register<P: ConrodProvider + 'static>(&mut self, provider: P) {
        self.provider.push(Box::new(provider));
    }

    /// Obtain a reference to the `ImageMap` to store images
    /// used by `Widget`s
    pub fn image_map(&mut self) -> &mut ImageMap<Texture> {
        &mut self.image_map
    }

    /// Render on a given `RenderOutput`
    pub fn render(&mut self, output: &mut RenderOutput) {

        // Update
        //

        trace!(self.logger, "Updating Widgets");

        {
            let mut cell = self.ui.set_widgets();

            for child in &mut self.provider {
                child.render(output, &mut cell);
            }
        }

        // Render
        //

        trace!(self.logger, "Rendering");

        let res = output.resolution();
        if let Some(gl) = output.get::<GlGraphics>() {

            fn texture_from_image<T>(img: &T) -> &T {
                img
            };
            fn cache_queued_glyphs(_: &mut GlGraphics, tex: &mut Texture, rect: Rect<u32>, data: &[u8]) {
                struct Bytes {
                    b: u8,
                    i: u8,
                }
                impl Iterator for Bytes {
                    type Item = u8;
                    fn next(&mut self) -> Option<Self::Item> {
                        let b = match self.i {
                            0 | 1 | 2 => 255,
                            3 => self.b,
                            _ => return None,
                        };
                        self.i += 1;
                        Some(b)
                    }
                }

                UpdateTexture::update(tex,
                                      &mut (),
                                      Format::Rgba8,
                                      &data.iter().flat_map(|x| Bytes { b: *x, i: 0 }).collect::<Vec<u8>>(),
                                      [rect.min.x, rect.min.y],
                                      [rect.width(), rect.height()])
                    .unwrap()
            }

            let primitives = self.ui.draw();
            let text_tex = &mut self.text_tex;
            let text_cache = &mut self.text_cache;
            let map = &self.image_map;

            gl.write().unwrap().draw(Viewport {
                                         rect: [0, 0, res.w as i32, res.h as i32],
                                         draw_size: [res.w, res.h],
                                         window_size: [res.w, res.h],
                                     },
                                     move |c, g| {
                backend::piston::draw::primitives(primitives,
                                                  c,
                                                  g,
                                                  text_tex,
                                                  text_cache,
                                                  map,
                                                  cache_queued_glyphs,
                                                  texture_from_image);
            });
        }
    }
}

impl Deref for ConrodInstance {
    type Target = Ui;

    fn deref(&self) -> &Ui {
        &self.ui
    }
}

impl DerefMut for ConrodInstance {
    fn deref_mut(&mut self) -> &mut Ui {
        &mut self.ui
    }
}
