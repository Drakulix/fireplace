use crate::config::Config;
use crate::shell::{
    workspace::Workspaces,
    window::PopupKind,
};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};
use smithay::{
    wayland::{
        data_device::{init_data_device, default_action_chooser},
        seat::{Seat, Keysym},
        shell::xdg::ShellState as XdgShellState,
        shm::init_shm_global,
        output::xdg::init_xdg_output_manager,
    },
    reexports::wayland_server::{
        Display,
    },
};

pub struct Fireplace {
    pub config: Config,
    pub display: Rc<RefCell<Display>>,
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
}

impl Fireplace {
    pub fn new(config: Config, display: Display) -> Self {
        let display = Rc::new(RefCell::new(display));

        init_shm_global(&mut (*display).borrow_mut(), vec![], None);
        let shell = crate::shell::init_shell(display.clone());
        init_xdg_output_manager(&mut display.borrow_mut(), None);
        let initial_seat = crate::handler::add_seat(&mut *display.borrow_mut(), "seat-1".into());
        init_data_device(
            &mut display.borrow_mut(),
            |_dnd_event| { /* TODO */ },
            default_action_chooser,
            None
        );

        Fireplace {
            config,
            display,
            start_time: std::time::Instant::now(),
            should_stop: false,
            xdg_state: shell.xdg_state,
            workspaces: shell.workspaces,
            popups: shell.popups,
            seats: vec![initial_seat.clone()],
            last_active_seat: initial_seat,
            suppressed_keys: Vec::new(),
        }
    }
}

