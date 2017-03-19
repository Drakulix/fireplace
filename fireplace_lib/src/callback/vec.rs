use super::IntoCallback;
use std::ops::{Deref, DerefMut};

use wlc::*;
#[cfg(feature = "render")]
use wlc::render::*;

/// A trait to express that your struct may split Callback calls onto
/// an arbitrary amount of children.
///
///
/// You may acquire a `VecCallback` struct actually implementing `Callback` via
/// the [`IntoCallback`](./trait.IntoCallback.html) trait.
///
/// Note that fireplace APIs all take `IntoCallback` types instead of
/// `Callback`,
/// so you usually don't need to perform the conversation yourself.
///
/// A default implementation for `std::vec::Vec` is provided.
///
/// # Example
/// ```norun
/// use ::wlc::{Callback, View};
/// use ::fireplace_lib::callback::{IntoCallback, AsVecCallback, VecCallback};
///
/// pub struct CountingViewSplit<C: Callback + 'static>
/// {
///     pub children: Vec<C>,
///     pub index: usize,
/// }
///
/// impl<C> AsVecCallback for CountingViewSplit<C>
///     where C: Callback + 'static
/// {
///     type Callback = C;
///
///     fn children(&mut self) -> &mut [Self::Callback]
///     {
///         &mut self.children
///     }
/// }
///
/// impl<C> Callback for VecCallback<CountingViewSplit<C>>
///     where C: Callback + 'static
/// {
///     fn view_created(&mut self, view: &View) -> bool
///     {
///         if self.index >= self.children.len() { self.index = 0 }
///         self.children()[self.index].view_created(view)
///     }
///
///     // All remaining Callback functions are still send to all children.
/// // Here specialization kicks in and uses the `default` implementation
/// of `VecCallback`.
///     //
/// // Note that this example is against the concepts presented in the
/// handler trait
/// // All Callbacks should be able to assume all `view_*` methods they may
/// receive have
///     // a corresponding `view_created` even, that has happend earlier.
///     // Meaning you never receive events for view's you never saw created.
/// }
/// ```
pub trait AsVecCallback {
    /// Type of the children
    type Callback: Callback + 'static;
    /// Returns a mutable slice of the children
    fn children(&mut self) -> &mut [Self::Callback];
}

impl<S: AsVecCallback> IntoCallback<VecCallback<S>> for S {
    fn into_callback(self) -> VecCallback<S> {
        VecCallback(self)
    }
}

/// Struct wrapping `AsVecCallback` Implementations
///
/// Because we cannot do specialized implementations for all types implementing
/// `AsVecCallback`,
/// we need to wrap `AsVecCallback` types for the actual implementation.
///
/// Take a look at [`AsVecCallback`](./trait.AsVecCallback.html) for an example
/// for when to use this.
pub struct VecCallback<T: AsVecCallback>(T);

impl<T: AsVecCallback> Deref for VecCallback<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: AsVecCallback> DerefMut for VecCallback<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: AsVecCallback> Callback for VecCallback<T> {
    default fn output_created(&mut self, output: &Output) -> bool {
        let mut result = true;
        for hook in self.0.children() {
            result = hook.output_created(output) && result;
        }
        result
    }

    default fn output_destroyed(&mut self, output: &Output) {
        for hook in self.0
                .children()
                .iter_mut()
                .rev() {
            hook.output_destroyed(output);
        }
    }

    default fn output_focus(&mut self, output: &Output, focus: bool) {
        for hook in self.0.children() {
            hook.output_focus(output, focus);
        }
    }

    default fn output_resolution(&mut self, output: &Output, from: Size, to: Size) {
        for hook in self.0.children() {
            hook.output_resolution(output, from, to);
        }
    }

    #[cfg(feature = "render")]
    default fn output_render_pre(&mut self, output: &mut RenderOutput) {
        for hook in self.0.children() {
            hook.output_render_pre(output);
        }
    }


    #[cfg(not(feature = "render"))]
    default fn output_render_pre(&mut self, output: &Output) {
        for hook in self.0.children() {
            hook.output_render_pre(output);
        }
    }

    #[cfg(feature = "render")]
    default fn output_render_post(&mut self, output: &mut RenderOutput) {
        for hook in self.0
                .children()
                .iter_mut()
                .rev() {
            hook.output_render_post(output);
        }
    }

    #[cfg(not(feature = "render"))]
    default fn output_render_post(&mut self, output: &Output) {
        for hook in self.0
                .children()
                .iter_mut()
                .rev() {
            hook.output_render_post(output);
        }
    }

    default fn output_context_created(&mut self, output: &Output) {
        for hook in self.0.children() {
            hook.output_context_created(output);
        }
    }

    default fn output_context_destroyed(&mut self, output: &Output) {
        for hook in self.0
                .children()
                .iter_mut()
                .rev() {
            hook.output_context_destroyed(output);
        }
    }

    default fn view_created(&mut self, view: &View) -> bool {
        let mut result = true;
        for hook in self.0.children() {
            result = hook.view_created(view) && result;
        }
        result
    }

    default fn view_destroyed(&mut self, view: &View) {
        for hook in self.0
                .children()
                .iter_mut()
                .rev() {
            hook.view_destroyed(view);
        }
    }

    default fn view_focus(&mut self, view: &View, focus: bool) {
        for hook in self.0.children() {
            hook.view_focus(view, focus);
        }
    }

    default fn view_move_to_output(&mut self, view: &View, from: &Output, to: &Output) {
        for hook in self.0.children() {
            hook.view_move_to_output(view, from, to);
        }
    }

    default fn view_request_geometry(&mut self, view: &View, geometry: Geometry) {
        for hook in self.0.children() {
            hook.view_request_geometry(view, geometry);
        }
    }

    default fn view_request_state(&mut self, view: &View, state: ViewState::Flags, toggle: bool) {
        for hook in self.0.children() {
            hook.view_request_state(view, state, toggle);
        }
    }

    default fn view_request_move(&mut self, view: &View, origin: Point) {
        for hook in self.0.children() {
            hook.view_request_move(view, origin);
        }
    }

    default fn view_request_resize(&mut self, view: &View, edges: ResizeEdge::Flags, origin: Point) {
        for hook in self.0.children() {
            hook.view_request_resize(view, edges, origin);
        }
    }

    #[cfg(feature = "render")]
    default fn view_render_pre(&mut self, view: &mut RenderView) {
        for hook in self.0.children() {
            hook.view_render_pre(view);
        }
    }

    #[cfg(not(feature = "render"))]
    default fn view_render_pre(&mut self, view: &View) {
        for hook in self.0.children() {
            hook.view_render_pre(view);
        }
    }

    #[cfg(feature = "render")]
    default fn view_render_post(&mut self, view: &mut RenderView) {
        for hook in self.0
                .children()
                .iter_mut()
                .rev() {
            hook.view_render_post(view);
        }
    }

    #[cfg(not(feature = "render"))]
    default fn view_render_post(&mut self, view: &View) {
        for hook in self.0
                .children()
                .iter_mut()
                .rev() {
            hook.view_render_post(view);
        }
    }

    default fn view_properties_updated(&mut self, view: &View, mask: ViewPropertyUpdate::Flags) {
        for hook in self.0.children() {
            hook.view_properties_updated(view, mask);
        }
    }

    default fn keyboard_key(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, key: Key,
                            state: KeyState)
                            -> bool {
        let mut result = false;
        for hook in self.0.children() {
            result = hook.keyboard_key(view, time, modifiers, key, state) || result;
        }
        result
    }

    default fn pointer_button(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers,
                              button: Button, state: ButtonState, origin: Point)
                              -> bool {
        let mut result = false;
        for hook in self.0.children() {
            result = hook.pointer_button(view, time, modifiers, button, state, origin) || result;
        }
        result
    }

    default fn pointer_scroll(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers,
                              axis: ScrollAxis::Flags, amount: [f64; 2])
                              -> bool {
        let mut result = false;
        for hook in self.0.children() {
            result = hook.pointer_scroll(view, time, modifiers, axis, amount) || result;
        }
        result
    }

    default fn pointer_motion(&mut self, view: Option<&View>, time: u32, origin: Point) -> bool {
        let mut result = false;
        for hook in self.0.children() {
            result = hook.pointer_motion(view, time, origin) || result;
        }
        result
    }

    default fn touch(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers,
                     touch_type: TouchType, slot: i32, origin: Point)
                     -> bool {
        let mut result = false;
        for hook in self.0.children() {
            result = hook.touch(view, time, modifiers, touch_type, slot, origin) || result;
        }
        result
    }

    default fn compositor_ready(&mut self) {
        for hook in self.0.children() {
            hook.compositor_ready()
        }
    }

    default fn compositor_terminate(&mut self) {
        for hook in self.0
                .children()
                .iter_mut()
                .rev() {
            hook.compositor_terminate()
        }
    }
}

impl<C: Callback + 'static> AsVecCallback for Vec<C> {
    type Callback = C;

    fn children(&mut self) -> &mut [Self::Callback] {
        self.as_mut_slice()
    }
}
