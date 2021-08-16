use smithay::{
    reexports::{
        wayland_protocols::xdg_shell::server::xdg_toplevel::ResizeEdge,
        wayland_server::protocol::wl_surface::WlSurface,
    },
    utils::{Logical, Point, Rectangle, Size},
    wayland::{
        seat::{GrabStartData, Seat},
        shell::xdg::ToplevelConfigure,
        Serial,
    },
};
use std::sync::atomic::AtomicUsize;

use super::window::Kind;

mod floating;
pub use self::floating::Floating;

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub trait Layout {
    fn id(&self) -> usize;
    fn new_toplevel(&mut self, surface: Kind);
    fn move_request(
        &mut self,
        surface: Kind,
        seat: &Seat,
        serial: Serial,
        start_data: GrabStartData,
    );
    fn resize_request(
        &mut self,
        surface: Kind,
        seat: &Seat,
        serial: Serial,
        start_data: GrabStartData,
        edges: ResizeEdge,
    );
    fn ack_configure(&mut self, surface: WlSurface, configure: ToplevelConfigure);
    fn commit(&mut self, surface: Kind);
    fn fullscreen_request(&mut self, surface: Kind, state: bool);
    fn maximize_request(&mut self, surface: Kind, state: bool);
    fn minimize_request(&mut self, surface: Kind);
    fn remove_toplevel(&mut self, surface: Kind);
    fn on_focus(&mut self, surface: &WlSurface);
    //TODO: fn window_options(&mut self, surface: Kind) -> Vec<String>;

    fn is_empty(&self) -> bool;
    fn rearrange(&mut self, size: &Size<i32, Logical>);

    fn surface_under(
        &mut self,
        point: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<i32, Logical>)>;
    fn focused_window(&self) -> Option<Kind>;
    fn windows<'a>(&'a self) -> Box<dyn Iterator<Item = Kind> + 'a>;
    fn windows_from_bottom_to_top<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = (Kind, Point<i32, Logical>, Rectangle<i32, Logical>)> + 'a>;

    /// Sends the frame callback to all the subsurfaces in this
    /// window that requested it
    fn send_frames(&self, time: u32) {
        use crate::shell::SurfaceData;
        use smithay::wayland::compositor::{with_surface_tree_downward, TraversalAction};

        for w in self.windows() {
            if let Some(wl_surface) = w.get_surface() {
                with_surface_tree_downward(
                    wl_surface,
                    (),
                    |_, _, &()| TraversalAction::DoChildren(()),
                    |_, states, &()| {
                        // the surface may not have any user_data if it is a subsurface and has not
                        // yet been commited
                        SurfaceData::send_frame(&mut *states.cached_state.current(), time)
                    },
                    |_, _, &()| true,
                );
            }
        }
    }
}

impl PartialEq for Box<dyn Layout> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
