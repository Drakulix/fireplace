//! Types to use the `conrod` crate for UI rendering directly inside the
//! compositor.
//!

use wlc::{Callback, Output, Size, View, Modifiers, Key, KeyState, Button, ButtonState, Point, ScrollAxis, TouchType};
use wlc::render::RenderOutput;

#[doc(hidden)]
pub mod convert;

mod renderer;
pub use self::renderer::*;
pub mod provider;
pub mod deserializer;

use callback::Wrapper;
use handlers::render::gl::GLInit;
use handlers::render::graphics::GraphicsRenderer;
use handlers::store::Store;

/// Handler that initializes a `ConrodRenderer` for every `Output`.
///
/// Can be used for rendering with the `conrod` library.
///
/// ## Dependencies
///
/// - [`StoreHandler`](./struct.StoreHandler.html)
///
pub struct ConrodHandler;

impl ConrodHandler {
    /// Initialize a new `ConrodHandler`
    pub fn new() -> GLInit<Wrapper<GraphicsRenderer<ConrodHandler>>> {
        GraphicsRenderer::new(ConrodHandler)
    }
}

impl Callback for ConrodHandler {
    fn output_context_created(&mut self, output: &Output) {
        let res = output.resolution();
        output.insert::<ConrodRenderer>(ConrodRenderer::new([res.w as f64, res.h as f64]));
    }

    fn output_context_destroyed(&mut self, output: &Output) {
        output.remove::<ConrodRenderer>();
    }

    fn output_resolution(&mut self, output: &Output, from: Size, to: Size) {
        let lock = output.get::<ConrodRenderer>();
        if let Some(mut ui) = lock.as_ref().and_then(|x| x.write().ok()) {
            ui.output_resolution(output, from, to)
        };
    }

    fn output_render_pre(&mut self, output: &mut RenderOutput) {
        let lock = output.get::<ConrodRenderer>();
        if let Some(mut ui) = lock.as_ref().and_then(|x| x.write().ok()) {
            ui.output_render_pre(output)
        };
    }

    fn output_render_post(&mut self, output: &mut RenderOutput) {
        let lock = output.get::<ConrodRenderer>();
        if let Some(mut ui) = lock.as_ref().and_then(|x| x.write().ok()) {
            ui.output_render_post(output)
        };
    }

    fn keyboard_key(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, key: Key,
                    state: KeyState)
                    -> bool {
        Output::with_focused_output(|output| {
            let lock = output.get::<ConrodRenderer>();
            if let Some(mut ui) = lock.as_ref().and_then(|x| x.write().ok()) {
                ui.keyboard_key(view, time, modifiers, key, state)
            } else {
                false
            }
        })
    }

    fn pointer_button(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, button: Button,
                      state: ButtonState, origin: Point)
                      -> bool {
        Output::with_focused_output(|output| {
            let lock = output.get::<ConrodRenderer>();
            if let Some(mut ui) = lock.as_ref().and_then(|x| x.write().ok()) {
                ui.pointer_button(view, time, modifiers, button, state, origin)
            } else {
                false
            }
        })
    }

    fn pointer_scroll(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers,
                      axis: ScrollAxis::Flags, amount: [f64; 2])
                      -> bool {
        Output::with_focused_output(|output| {
            let lock = output.get::<ConrodRenderer>();
            if let Some(mut ui) = lock.as_ref().and_then(|x| x.write().ok()) {
                ui.pointer_scroll(view, time, modifiers, axis, amount)
            } else {
                false
            }
        })
    }

    fn pointer_motion(&mut self, view: Option<&View>, time: u32, origin: Point) -> bool {
        Output::with_focused_output(|output| {
            let lock = output.get::<ConrodRenderer>();
            if let Some(mut ui) = lock.as_ref().and_then(|x| x.write().ok()) {
                ui.pointer_motion(view, time, origin)
            } else {
                false
            }
        })
    }

    fn touch(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, touch_type: TouchType,
             slot: i32, origin: Point)
             -> bool {
        Output::with_focused_output(|output| {
            let lock = output.get::<ConrodRenderer>();
            if let Some(mut ui) = lock.as_ref().and_then(|x| x.write().ok()) {
                ui.touch(view, time, modifiers, touch_type, slot, origin)
            } else {
                false
            }
        })
    }
}
