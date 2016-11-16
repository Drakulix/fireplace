use super::IntoCallback;
use std::ops::{Deref, DerefMut};

use wlc::*;
#[cfg(feature = "render")]
use wlc::render::*;

/// A trait to express that your struct may forward Callback calls onto
/// a wrapped child.
///
///
/// You may acquire a Wrapper struct actually implementing `Callback` via the
/// [`IntoCallback`](./trait.IntoCallback.html) trait.
///
/// Note that fireplace APIs all take `IntoCallback` types instead of
/// `Callback`,
/// so you usually don't need to perform the conversation yourself.
///
/// # Example
/// ```norun
/// use ::wlc::{Callback, View};
/// use ::fireplace_lib::callback::{IntoCallback, AsWrapper, Wrapper};
///
/// pub struct LargeViewFilter<C: Callback + 'static>(pub C);
///
/// impl<C> AsWrapper for LargeViewFilter<C>
///     where C: Callback + 'static
/// {
///     type Callback = C;
///
///     fn child(&mut self) -> &mut Self::Callback
///     {
///         &mut self.0
///     }
/// }
///
/// impl<C> Callback for Wrapper<LargeViewFilter<C>>
///     where C: Callback + 'static
/// {
///     fn view_created(&mut self, view: &View) -> bool
///     {
///         if view.geometry().size.w * view.geometry().size.h >= 1000000
///         {
///             self.child().view_created(view)
///         }
///     }
///
///     // All remaining Callback functions are still send to the child.
/// // Here specialization kicks in and uses the `default` implementation
/// of `Wrapper`.
///     //
/// // Note that this example is against the concepts presented in the
/// handler trait
/// // All Callbacks should be able to assume all `view_*` methods they may
/// receive have
///     // a corresponding `view_created` even, that has happend earlier.
///     // Meaning you never receive events for view's you never saw created.
/// }
/// ```
pub trait AsWrapper {
    /// Returns a mutable reference to the wrapped child, if one exists
    fn child(&mut self) -> Option<&mut Callback>;
}

impl<W: AsWrapper> IntoCallback<Wrapper<W>> for W {
    fn into_callback(self) -> Wrapper<W> {
        Wrapper(self)
    }
}

/// Struct wrapping `AsWrapper` Implementations
///
/// Because we cannot do specialized implementations for all types implementing
/// `AsWrapper`,
/// we need to wrap `AsWrapper` types for the actual implementation.
///
/// Take a look at [`AsWrapper`](./trait.AsWrapper.html) for an example for
/// when to use this.
pub struct Wrapper<T: AsWrapper>(T);

impl<T: AsWrapper> Deref for Wrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: AsWrapper> DerefMut for Wrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: AsWrapper> Callback for Wrapper<T> {
    default fn output_created(&mut self, output: &Output) -> bool {
        if let Some(child) = self.0.child() {
            child.output_created(output)
        } else {
            true
        }
    }

    default fn output_destroyed(&mut self, output: &Output) {
        if let Some(child) = self.0.child() {
            child.output_destroyed(output)
        }
    }

    default fn output_focus(&mut self, output: &Output, focus: bool) {
        if let Some(child) = self.0.child() {
            child.output_focus(output, focus)
        }
    }

    default fn output_resolution(&mut self, output: &Output, from: Size, to: Size) {
        if let Some(child) = self.0.child() {
            child.output_resolution(output, from, to)
        }
    }

    #[cfg(feature = "render")]
    default fn output_render_pre(&mut self, output: &mut RenderOutput) {
        if let Some(child) = self.0.child() {
            child.output_render_pre(output)
        }
    }

    #[cfg(not(feature = "render"))]
    default fn output_render_pre(&mut self, output: &Output) {
        if let Some(child) = self.0.child() {
            child.output_render_pre(output)
        }
    }

    #[cfg(feature = "render")]
    default fn output_render_post(&mut self, output: &mut RenderOutput) {
        if let Some(child) = self.0.child() {
            child.output_render_post(output)
        }
    }

    #[cfg(not(feature = "render"))]
    default fn output_render_post(&mut self, output: &Output) {
        if let Some(child) = self.0.child() {
            child.output_render_post(output)
        }
    }

    default fn output_context_created(&mut self, output: &Output) {
        if let Some(child) = self.0.child() {
            child.output_context_created(output)
        }
    }

    default fn output_context_destroyed(&mut self, output: &Output) {
        if let Some(child) = self.0.child() {
            child.output_context_destroyed(output)
        }
    }

    default fn view_created(&mut self, view: &View) -> bool {
        if let Some(child) = self.0.child() {
            child.view_created(view)
        } else {
            true
        }
    }

    default fn view_destroyed(&mut self, view: &View) {
        if let Some(child) = self.0.child() {
            child.view_destroyed(view)
        }
    }

    default fn view_focus(&mut self, view: &View, focus: bool) {
        if let Some(child) = self.0.child() {
            child.view_focus(view, focus)
        }
    }

    default fn view_move_to_output(&mut self, view: &View, from: &Output, to: &Output) {
        if let Some(child) = self.0.child() {
            child.view_move_to_output(view, from, to)
        }
    }

    default fn view_request_geometry(&mut self, view: &View, geometry: Geometry) {
        if let Some(child) = self.0.child() {
            child.view_request_geometry(view, geometry)
        }
    }

    default fn view_request_state(&mut self, view: &View, state: ViewState::Flags, toggle: bool) {
        if let Some(child) = self.0.child() {
            child.view_request_state(view, state, toggle)
        }
    }

    default fn view_request_move(&mut self, view: &View, origin: Point) {
        if let Some(child) = self.0.child() {
            child.view_request_move(view, origin)
        }
    }

    default fn view_request_resize(&mut self, view: &View, edges: ResizeEdge::Flags, origin: Point) {
        if let Some(child) = self.0.child() {
            child.view_request_resize(view, edges, origin)
        }
    }

    #[cfg(feature = "render")]
    default fn view_render_pre(&mut self, view: &mut RenderView) {
        if let Some(child) = self.0.child() {
            child.view_render_pre(view)
        }
    }

    #[cfg(not(feature = "render"))]
    default fn view_render_pre(&mut self, view: &View) {
        if let Some(child) = self.0.child() {
            child.view_render_pre(view)
        }
    }

    #[cfg(feature = "render")]
    default fn view_render_post(&mut self, view: &mut RenderView) {
        if let Some(child) = self.0.child() {
            child.view_render_post(view)
        }
    }

    #[cfg(not(feature = "render"))]
    default fn view_render_post(&mut self, view: &View) {
        if let Some(child) = self.0.child() {
            child.view_render_post(view)
        }
    }

    default fn view_properties_updated(&mut self, view: &View, mask: ViewPropertyUpdate::Flags) {
        if let Some(child) = self.0.child() {
            child.view_properties_updated(view, mask)
        }
    }

    default fn keyboard_key(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, key: Key,
                            state: KeyState)
                            -> bool {
        if let Some(child) = self.0.child() {
            child.keyboard_key(view, time, modifiers, key, state)
        } else {
            false
        }
    }

    default fn pointer_button(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers,
                              button: Button, state: ButtonState, origin: Point)
                              -> bool {
        if let Some(child) = self.0.child() {
            child.pointer_button(view, time, modifiers, button, state, origin)
        } else {
            false
        }
    }

    default fn pointer_scroll(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers,
                              axis: ScrollAxis::Flags, amount: [f64; 2])
                              -> bool {
        if let Some(child) = self.0.child() {
            child.pointer_scroll(view, time, modifiers, axis, amount)
        } else {
            false
        }
    }

    default fn pointer_motion(&mut self, view: Option<&View>, time: u32, origin: Point) -> bool {
        if let Some(child) = self.0.child() {
            child.pointer_motion(view, time, origin)
        } else {
            false
        }
    }

    default fn touch(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers,
                     touch_type: TouchType, slot: i32, origin: Point)
                     -> bool {
        if let Some(child) = self.0.child() {
            child.touch(view, time, modifiers, touch_type, slot, origin)
        } else {
            false
        }
    }

    default fn compositor_ready(&mut self) {
        if let Some(child) = self.0.child() {
            child.compositor_ready()
        }
    }

    default fn compositor_terminate(&mut self) {
        if let Some(child) = self.0.child() {
            child.compositor_terminate()
        }
    }
}
