use conrod::Dimensions;
use conrod::event::Input;
use conrod::text::GlyphCache;

use opengles_graphics::Texture;

use slog;
use slog_scope;
use texture::TextureSettings;

use wlc::{Callback, Output, Size, View, Modifiers, Key, KeyState, Button, ButtonState, Point, ScrollAxis, TouchType};
use wlc::render::RenderOutput;

mod instance;
pub use self::instance::*;

use handlers::store::StoreKey;

/// Renderer created by `ConrodHandler`
///
/// Contains two `ConrodInstance`s which can be used for
/// actual rendering.
///
/// The `background` instance renders behind any `Views`.
/// The `foreground` instance renders before any `Views`.
///
/// *Currently* no input is directed to the `Widget`s.
///
pub struct ConrodRenderer {
    /// Foreground instance. Renders behind any `View`.
    pub foreground: ConrodInstance,
    /// Background instance. Renders before any `View`.
    pub background: ConrodInstance,
    logger: slog::Logger,
}

impl StoreKey for ConrodRenderer {
    type Value = ConrodRenderer;
}

impl ConrodRenderer {
    /// Initialize a new `ConrodRenderer` for given screen `Dimensions`
    pub fn new(dim: Dimensions) -> ConrodRenderer {
        let logger = slog_scope::logger().new(o!("instance" => "ConrodRenderer"));
        debug!(logger, "Initializing");
        ConrodRenderer {
            foreground: ConrodInstance::new(dim, logger.new(o!("renderer" => "Foreground"))),
            background: ConrodInstance::new(dim, logger.new(o!("renderer" => "Background"))),
            logger: logger,
        }
    }
}

impl Callback for ConrodRenderer {
    fn output_resolution(&mut self, output: &Output, _from: Size, to: Size) {
        debug!(self.logger, "Resizing UI");

        let len = (to.w * to.h * 4) as usize;
        let mut empty: Vec<u8> = Vec::with_capacity(len);
        for _ in 0..len {
            empty.push(0u8);
        }

        self.foreground.ui.handle_event(Input::Resize(to.w, to.h));
        self.foreground.text_tex = Texture::from_memory_alpha(&empty, to.w, to.h, &TextureSettings::new())
            .unwrap();
        self.foreground.text_cache = GlyphCache::new(to.w,
                                                     to.h,
                                                     0.1 / output.scale() as f32,
                                                     0.1 / output.scale() as f32);

        self.background.ui.handle_event(Input::Resize(to.w, to.h));
        self.background.text_tex = Texture::from_memory_alpha(&empty, to.w, to.h, &TextureSettings::new())
            .unwrap();
        self.background.text_cache = GlyphCache::new(to.w,
                                                     to.h,
                                                     0.1 / output.scale() as f32,
                                                     0.1 / output.scale() as f32);
    }

    fn output_render_pre(&mut self, output: &mut RenderOutput) {
        self.background.render(output);
    }

    fn output_render_post(&mut self, output: &mut RenderOutput) {
        self.foreground.render(output);
    }

    fn keyboard_key(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, key: Key,
                    state: KeyState)
                    -> bool {
        if !self.foreground.keyboard_key(view, time, modifiers, key, state) {
            if view.is_some() {
                false
            } else {
                self.background.keyboard_key(view, time, modifiers, key, state)
            }
        } else {
            true
        }
    }

    fn pointer_button(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, button: Button,
                      state: ButtonState, origin: Point)
                      -> bool {
        if !self.foreground.pointer_button(view, time, modifiers, button, state, origin) {
            if view.is_some() {
                false
            } else {
                self.background.pointer_button(view, time, modifiers, button, state, origin)
            }
        } else {
            true
        }
    }

    fn pointer_scroll(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers,
                      axis: ScrollAxis::Flags, amount: [f64; 2])
                      -> bool {
        if !self.foreground.pointer_scroll(view, time, modifiers, axis, amount) {
            if view.is_some() {
                false
            } else {
                self.background.pointer_scroll(view, time, modifiers, axis, amount)
            }
        } else {
            true
        }
    }

    fn pointer_motion(&mut self, view: Option<&View>, time: u32, origin: Point) -> bool {
        if !self.foreground.pointer_motion(view, time, origin) {
            if view.is_some() {
                false
            } else {
                self.background.pointer_motion(view, time, origin)
            }
        } else {
            true
        }
    }

    fn touch(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, touch_type: TouchType,
             slot: i32, origin: Point)
             -> bool {
        if !self.foreground.touch(view, time, modifiers, touch_type, slot, origin) {
            if view.is_some() {
                false
            } else {
                self.background.touch(view, time, modifiers, touch_type, slot, origin)
            }
        } else {
            true
        }
    }
}
