use crate::{
    backend::udev::RenderSurface,
    config::Config,
    shell::{window::PopupKind, workspace::Workspaces},
};
use smithay::{
    backend::renderer::gles2::{Gles2Renderer, Gles2Texture},
    reexports::{
        drm::control::crtc,
        calloop::RegistrationToken,
        nix::sys::stat::dev_t,
        wayland_server::Display,
    },
    wayland::{
        data_device::{default_action_chooser, init_data_device},
        output::xdg::init_xdg_output_manager,
        seat::{Keysym, Seat},
        shell::xdg::ShellState as XdgShellState,
        shm::init_shm_global,
    },
    utils::{
        Size, Physical,
        signaling::SignalToken
    },
};
use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::OsString,
    rc::Rc,
    sync::{Arc, Mutex},
};

pub struct Fireplace {
    pub config: Config,
    pub display: Rc<RefCell<Display>>,
    pub socket_name: OsString,
    pub start_time: std::time::Instant,
    pub should_stop: bool,

    // shell
    pub xdg_state: Arc<Mutex<XdgShellState>>,
    pub workspaces: Rc<RefCell<Workspaces>>,
    pub popups: Rc<RefCell<Vec<PopupKind>>>,

    // input
    pub seats: Vec<Seat>,
    pub last_active_seat: Seat,
    pub suppressed_keys: Vec<Keysym>,

    // backend
    pub tokens: Vec<RegistrationToken>,
    pub udev: HashMap<dev_t, BackendData>,
}

pub struct BackendData {
    pub _restart_token: SignalToken,
    pub registration_token: RegistrationToken,
    pub surfaces: HashMap<crtc::Handle, SurfaceData>,
    pub pointer: crate::backend::udev::Cursor,
    pub pointer_images: Vec<(xcursor::parser::Image, Gles2Texture)>,
    //fps_texture: Gles2Texture,
    pub renderer: Gles2Renderer,
}

pub struct SurfaceData {
    pub output: String,
    pub size: Size<i32, Physical>,
    pub surface: RenderSurface,
    //fps: fps_ticker::Fps,
}

impl Fireplace {
    pub fn new(config: Config, display: Display, socket_name: OsString) -> Self {
        let display = Rc::new(RefCell::new(display));

        init_shm_global(&mut (*display).borrow_mut(), vec![], None);
        let shell = crate::shell::init_shell(display.clone());
        init_xdg_output_manager(&mut display.borrow_mut(), None);
        let initial_seat = crate::handler::add_seat(&mut *display.borrow_mut(), "seat-1".into());
        init_data_device(
            &mut display.borrow_mut(),
            |_dnd_event| { /* TODO */ },
            default_action_chooser,
            None,
        );

        Fireplace {
            config,
            display,
            socket_name,
            start_time: std::time::Instant::now(),
            should_stop: false,
            xdg_state: shell.xdg_state,
            workspaces: shell.workspaces,
            popups: shell.popups,
            seats: vec![initial_seat.clone()],
            last_active_seat: initial_seat,
            suppressed_keys: Vec::new(),
            tokens: Vec::new(),
            udev: HashMap::new(),
        }
    }
}
