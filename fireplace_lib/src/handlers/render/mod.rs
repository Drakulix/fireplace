//! Collection of handlers related to render/draw directly inside
//! of the compositor.
//!
//! Note that `wlc` already [provides
//! functionality](../../../wlc/render/trait.Renderer.html)
//! to read and write parts of an rendered `Output` through receiving
//! and setting pixels.
//!
//! That is an easy to use and useful functionality, but can be a real
//! performance penalty when used regulary for rendering UI elements.
//! See [this](https://github.
//! com/SirCmpwn/sway/issues/1004#issuecomment-268894678) for example.
//!
//! These handlers offer more advanced rendering utilizing OpenGL ES for
//! drawing just like the compositor does to render `View`s itself, which
//! offers far better performance and possibilities although is a bit more
//! complicated of course.
//!

#[cfg(feature = "gl")]
mod gl;
#[cfg(feature = "gl")]
pub use self::gl::GLInit;

#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
pub use self::graphics::GraphicsRenderer;

#[cfg(feature = "conrod_ui")]
pub mod conrod;
