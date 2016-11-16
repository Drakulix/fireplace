use wlc::{Callback, Point, View, input};

/// Handler to move the pointer according to mouse movement
///
/// ## Dependencies
///
/// This handler has no dependencies.
///
#[derive(Default)]
pub struct PointerHandler {}

impl Callback for PointerHandler {
    fn pointer_motion(&mut self, _view: Option<&View>, _time: u32, origin: Point) -> bool {
        input::pointer::set_position(origin);
        false
    }
}

impl PointerHandler {
    /// Initialize a new `PointerHandler`
    pub fn new() -> PointerHandler {
        PointerHandler {}
    }
}
