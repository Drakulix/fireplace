use anyhow::Result;
use calloop::EventLoop;

use crate::state::Fireplace;
mod render;
mod winit;

pub fn initial_backend_auto(
    event_loop: &mut EventLoop<Fireplace>,
    state: &Fireplace,
) -> Result<()> {
    if std::env::var_os("WAYLAND_DISPLAY").is_some()
        || std::env::var_os("WAYLAND_SOCKET").is_some()
        || std::env::var_os("DISPLAY").is_some()
    {
        winit::init_winit(event_loop, state)
    } else {
        unimplemented!("Current this only run nested");
    }
}
