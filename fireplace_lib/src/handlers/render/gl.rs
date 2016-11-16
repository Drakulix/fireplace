use callback::{AsWrapper, IntoCallback, Wrapper};
use egli;
use opengles_graphics::gl;
use slog;
use slog_scope;
use wlc::{Callback, Output};

/// Handler to load GL functions via `egl` at runtime for all
/// `Output` contexts to make GL calls possible.
///
/// Please note that you should restore any GL state you may modify
/// to let `wlc` continue to correctly render any `View`s.
///
/// `wlc` has currently no way to let use render `View`s ourselves, so
/// modifing the state directly influences `wlc`s internal rendering functions,
/// which is likely to render garbage on the screen.
///
/// They don't expose any stable API, so even currently working modifications
/// may completely break in the future and should always be avoided.
///
/// Take a look at the [`GraphicsHandler`](./struct.GraphicsHandler.html)
/// for a stable 2D rendering API
///
/// ## Dependencies
///
/// This handler has no dependencies.
///
#[derive(Debug)]
pub struct GLInit<C: Callback + 'static> {
    child: C,
    logger: slog::Logger,
}

impl<C: Callback + 'static> AsWrapper for GLInit<C> {
    fn child(&mut self) -> Option<&mut Callback> {
        Some(&mut self.child)
    }
}

impl<C: Callback + 'static> Callback for Wrapper<GLInit<C>> {
    fn output_context_created(&mut self, output: &Output) {
        gl::load_with(|s| egli::egl::get_proc_address(s) as *const _);
        debug!(self.logger, "Loaded GL Function Pointers");
        self.child.output_context_created(output)
    }
}

impl<C: Callback + 'static> GLInit<C> {
    /// Create a new GLInit handler. Exposes some unstable APIs.
    pub unsafe fn new<I: IntoCallback<C>>(renderer: I) -> GLInit<C> {
        GLInit {
            child: renderer.into_callback(),
            logger: slog_scope::logger().new(o!("handler" => "GLInit")),
        }
    }
}
