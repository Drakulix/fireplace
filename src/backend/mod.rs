use anyhow::Result;
use smithay::reexports::calloop::EventLoop;

use crate::state::Fireplace;
pub mod render;
pub mod udev;
pub mod winit;
pub mod egl;

pub fn initial_backend_auto(
    event_loop: &mut EventLoop<'static, Fireplace>,
    state: &mut Fireplace,
) -> Result<()> {
    if std::env::var_os("WAYLAND_DISPLAY").is_some()
        || std::env::var_os("WAYLAND_SOCKET").is_some()
        || std::env::var_os("DISPLAY").is_some()
    {
        winit::init_winit(event_loop, state)
    } else {
        udev::init_udev(event_loop, state)
    }
}
