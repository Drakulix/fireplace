use super::IntoCallback;
use std::ops::{Deref, DerefMut};

use wlc::*;
#[cfg(feature = "render")]
use wlc::render::*;

/// A trait to express that your struct may split Callback calls onto
/// two children.
///
///
/// Knowing that your amount of children will always be
/// two at maximum opens some options for optimizations and makes
/// some edge cases much easier to implement, as it is more common
/// than you might expect. This is main reason it exists next to
/// [`AsVec`](./trait.AsVec.html).
/// Other than that typing is easier, because both children have different
/// generic types.
///
/// You may acquire a Split struct actually implementing `Callback` via the
/// [`IntoCallback`](./trait.IntoCallback.html) trait.
///
/// Note that fireplace APIs all take `IntoCallback` types instead of
/// `Callback`,
/// so you usually don't need to perform the conversation yourself.
///
///
/// # Example
/// ```norun
/// use ::wlc::{Callback, View};
/// use ::fireplace_lib::callback::{IntoCallback, AsSplit, Split};
///
/// pub struct ViewSplit<C1: Callback + 'static, C2: Callback + 'static>
/// {
///     pub child1: C1,
///     pub child2: C2,
///     even: bool,
/// }
///
/// impl<C1, C2> AsSplit for ViewSplit<C1, C2>
///     where C1: Callback + 'static, C2: Callback + 'static
/// {
///     type Callback1 = C1;
///     type Callback2 = C2;
///
///     fn first_child(&mut self) -> Option<&mut Self::Callback1>
///     {
///         self.child1
///     }
///     fn second_child(&mut self) -> Option<&mut Self::Callback2>
///     {
///         self.child2
///     }
/// }
///
/// impl<C1, C2> Callback for Split<ViewSplit<C1, C2>>
///     where C1: Callback + 'static, C2: Callback + 'static
/// {
///     fn view_created(&mut self, view: &View) -> bool
///     {
///         let result = if self.even {
///             if let Some(child) = self.first_child() {
///                 child.view_created(view)
///             }
///         } else {
///             if let Some(child) = self.second_child() {
///                 child.view_created(view)
///             }
///         };
///         self.even = !self.even;
///         result
///     }
///
///     // All remaining Callback functions are still send to both children.
/// // Here specialization kicks in and uses the `default` implementation
/// of `Split`.
///     //
/// // Note that this example is against the concepts presented in the
/// handler trait
/// // All Callbacks should be able to assume all `view_*` methods they may
/// receive have
///     // a corresponding `view_created` even, that has happend earlier.
///     // Meaning you never receive events for view's you never saw created.
/// }
/// ```
pub trait AsSplit {
    /// Type of the first child
    type Callback1: Callback + 'static;
    /// Type of the second child
    type Callback2: Callback + 'static;
    /// Returns a mutable reference to your first child, when it exists
    fn first_child(&mut self) -> Option<&mut Self::Callback1>;
    /// Returns a mutable reference to your second child, when it exists
    fn second_child(&mut self) -> Option<&mut Self::Callback2>;
}

impl<S: AsSplit> IntoCallback<Split<S>> for S {
    fn into_callback(self) -> Split<S> {
        Split(self)
    }
}

/// Struct wrapping `AsSplit` Implementations
///
/// Because we cannot do specialized implementations for all types implementing
/// `AsSplit`, we need to wrap `AsSplit` types for the actual implementation.
///
/// Take a look at [`AsSplit`](./trait.AsSplit.html) for an example for when to
/// use this.
pub struct Split<T: AsSplit>(T);

impl<T: AsSplit> Deref for Split<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: AsSplit> DerefMut for Split<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: AsSplit> Callback for Split<T> {
    default fn output_created(&mut self, output: &Output) -> bool {
        let mut result = true;
        if let Some(child) = self.0.first_child() {
            result = child.output_created(output) && result;
        }
        if let Some(child) = self.0.second_child() {
            result = child.output_created(output) && result;
        }
        result
    }

    default fn output_destroyed(&mut self, output: &Output) {
        if let Some(child) = self.0.second_child() {
            child.output_destroyed(output);
        }
        if let Some(child) = self.0.first_child() {
            child.output_destroyed(output);
        }
    }

    default fn output_focus(&mut self, output: &Output, focus: bool) {
        if let Some(child) = self.0.first_child() {
            child.output_focus(output, focus);
        }
        if let Some(child) = self.0.second_child() {
            child.output_focus(output, focus);
        }
    }

    default fn output_resolution(&mut self, output: &Output, from: Size, to: Size) {
        if let Some(child) = self.0.first_child() {
            child.output_resolution(output, from, to);
        }
        if let Some(child) = self.0.second_child() {
            child.output_resolution(output, from, to);
        }
    }

    #[cfg(feature = "render")]
    default fn output_render_pre(&mut self, output: &mut RenderOutput) {
        if let Some(child) = self.0.first_child() {
            child.output_render_pre(output);
        }
        if let Some(child) = self.0.second_child() {
            child.output_render_pre(output);
        }
    }

    #[cfg(not(feature = "render"))]
    default fn output_render_pre(&mut self, output: &Output) {
        if let Some(child) = self.0.first_child() {
            child.output_render_pre(output);
        }
        if let Some(child) = self.0.second_child() {
            child.output_render_pre(output);
        }
    }

    #[cfg(feature = "render")]
    default fn output_render_post(&mut self, output: &mut RenderOutput) {
        if let Some(child) = self.0.second_child() {
            child.output_render_post(output);
        }
        if let Some(child) = self.0.first_child() {
            child.output_render_post(output);
        }
    }

    #[cfg(not(feature = "render"))]
    default fn output_render_post(&mut self, output: &Output) {
        if let Some(child) = self.0.second_child() {
            child.output_render_post(output);
        }
        if let Some(child) = self.0.first_child() {
            child.output_render_post(output);
        }
    }

    default fn output_context_created(&mut self, output: &Output) {
        if let Some(child) = self.0.first_child() {
            child.output_context_created(output);
        }
        if let Some(child) = self.0.second_child() {
            child.output_context_created(output);
        }
    }

    default fn output_context_destroyed(&mut self, output: &Output) {
        if let Some(child) = self.0.second_child() {
            child.output_context_destroyed(output);
        }
        if let Some(child) = self.0.first_child() {
            child.output_context_destroyed(output);
        }
    }

    default fn view_created(&mut self, view: &View) -> bool {
        let mut result = true;
        if let Some(child) = self.0.first_child() {
            result = child.view_created(view) && result;
        }
        if let Some(child) = self.0.second_child() {
            result = child.view_created(view) && result;
        }
        result
    }

    default fn view_destroyed(&mut self, view: &View) {
        if let Some(child) = self.0.first_child() {
            child.view_destroyed(view);
        }
        if let Some(child) = self.0.second_child() {
            child.view_destroyed(view);
        }
    }

    default fn view_focus(&mut self, view: &View, focus: bool) {
        if let Some(child) = self.0.first_child() {
            child.view_focus(view, focus);
        }
        if let Some(child) = self.0.second_child() {
            child.view_focus(view, focus);
        }
    }

    default fn view_move_to_output(&mut self, view: &View, from: &Output, to: &Output) {
        if let Some(child) = self.0.first_child() {
            child.view_move_to_output(view, from, to);
        }
        if let Some(child) = self.0.second_child() {
            child.view_move_to_output(view, from, to);
        }
    }

    default fn view_request_geometry(&mut self, view: &View, geometry: Geometry) {
        if let Some(child) = self.0.first_child() {
            child.view_request_geometry(view, geometry);
        }
        if let Some(child) = self.0.second_child() {
            child.view_request_geometry(view, geometry);
        }
    }

    default fn view_request_state(&mut self, view: &View, state: ViewState::Flags, toggle: bool) {
        if let Some(child) = self.0.first_child() {
            child.view_request_state(view, state, toggle);
        }
        if let Some(child) = self.0.second_child() {
            child.view_request_state(view, state, toggle);
        }
    }

    default fn view_request_move(&mut self, view: &View, origin: Point) {
        if let Some(child) = self.0.first_child() {
            child.view_request_move(view, origin);
        }
        if let Some(child) = self.0.second_child() {
            child.view_request_move(view, origin);
        }
    }

    default fn view_request_resize(&mut self, view: &View, edges: ResizeEdge::Flags, origin: Point) {
        if let Some(child) = self.0.first_child() {
            child.view_request_resize(view, edges, origin);
        }
        if let Some(child) = self.0.second_child() {
            child.view_request_resize(view, edges, origin);
        }
    }

    #[cfg(feature = "render")]
    default fn view_render_pre(&mut self, view: &mut RenderView) {
        if let Some(child) = self.0.first_child() {
            child.view_render_pre(view);
        }
        if let Some(child) = self.0.second_child() {
            child.view_render_pre(view);
        }
    }

    #[cfg(not(feature = "render"))]
    default fn view_render_pre(&mut self, view: &View) {
        if let Some(child) = self.0.first_child() {
            child.view_render_pre(view);
        }
        if let Some(child) = self.0.second_child() {
            child.view_render_pre(view);
        }
    }

    #[cfg(feature = "render")]
    default fn view_render_post(&mut self, view: &mut RenderView) {
        if let Some(child) = self.0.second_child() {
            child.view_render_post(view);
        }
        if let Some(child) = self.0.first_child() {
            child.view_render_post(view);
        }
    }

    #[cfg(not(feature = "render"))]
    default fn view_render_post(&mut self, view: &View) {
        if let Some(child) = self.0.second_child() {
            child.view_render_post(view);
        }
        if let Some(child) = self.0.first_child() {
            child.view_render_post(view);
        }
    }

    default fn view_properties_updated(&mut self, view: &View, mask: ViewPropertyUpdate::Flags) {
        if let Some(child) = self.0.first_child() {
            child.view_properties_updated(view, mask);
        }
        if let Some(child) = self.0.second_child() {
            child.view_properties_updated(view, mask);
        }
    }

    default fn keyboard_key(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, key: Key,
                            state: KeyState)
                            -> bool {
        let mut result = false;
        if let Some(child) = self.0.first_child() {
            result = child.keyboard_key(view, time, modifiers, key, state) || result;
        }
        if let Some(child) = self.0.second_child() {
            result = child.keyboard_key(view, time, modifiers, key, state) || result;
        }
        result
    }

    default fn pointer_button(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers,
                              button: Button, state: ButtonState, origin: Point)
                              -> bool {
        let mut result = false;
        if let Some(child) = self.0.first_child() {
            result = child.pointer_button(view, time, modifiers, button, state, origin) || result;
        }
        if let Some(child) = self.0.second_child() {
            result = child.pointer_button(view, time, modifiers, button, state, origin) || result;
        }
        result
    }

    default fn pointer_scroll(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers,
                              axis: ScrollAxis::Flags, amount: [f64; 2])
                              -> bool {
        let mut result = false;
        if let Some(child) = self.0.first_child() {
            result = child.pointer_scroll(view, time, modifiers, axis, amount) || result;
        }
        if let Some(child) = self.0.second_child() {
            result = child.pointer_scroll(view, time, modifiers, axis, amount) || result;
        }
        result
    }

    default fn pointer_motion(&mut self, view: Option<&View>, time: u32, origin: Point) -> bool {
        let mut result = false;
        if let Some(child) = self.0.first_child() {
            result = child.pointer_motion(view, time, origin) || result;
        }
        if let Some(child) = self.0.second_child() {
            result = child.pointer_motion(view, time, origin) || result;
        }
        result
    }

    default fn touch(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers,
                     touch_type: TouchType, slot: i32, origin: Point)
                     -> bool {
        let mut result = false;
        if let Some(child) = self.0.first_child() {
            result = child.touch(view, time, modifiers, touch_type, slot, origin) || result;
        }
        if let Some(child) = self.0.second_child() {
            result = child.touch(view, time, modifiers, touch_type, slot, origin) || result;
        }
        result
    }

    default fn compositor_ready(&mut self) {
        if let Some(child) = self.0.first_child() {
            child.compositor_ready();
        }
        if let Some(child) = self.0.second_child() {
            child.compositor_ready();
        }
    }

    default fn compositor_terminate(&mut self) {
        if let Some(child) = self.0.first_child() {
            child.compositor_terminate();
        }
        if let Some(child) = self.0.second_child() {
            child.compositor_terminate();
        }
    }
}
