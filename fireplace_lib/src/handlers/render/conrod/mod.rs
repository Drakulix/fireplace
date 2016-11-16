//! Types to use the `conrod` crate for UI rendering directly inside the
//! compositor.
//!

use wlc::{Callback, Output, Size};
use wlc::render::RenderOutput;

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
}
