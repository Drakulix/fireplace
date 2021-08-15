use std::{
    cell::RefCell,
    fs::OpenOptions,
    path::PathBuf,
    rc::Rc,
};

use anyhow::{Context, Result};
use smithay::{
    reexports::{
        calloop::{EventLoop, Interest, Mode, PostAction, generic::Generic},
        wayland_server::Display,
    },
};

mod backend;
mod config;
mod logger;
mod handler;
mod shell;
mod state;
pub use self::config::Config;
pub use self::state::Fireplace;

fn try_config_locations(paths: &[PathBuf]) -> (Option<PathBuf>, Config) {
    for path in paths {
        if path.exists() {
            return (Some(path.clone()), serde_yaml::from_reader(OpenOptions::new().read(true).open(path).unwrap())
                       .expect("Malformed config file"));
        }
    }
    (None, Config::default())
}

fn main() -> Result<()> {
    // Parse configuration
    let mut locations = if let Ok(base) = xdg::BaseDirectories::new() {
        base.list_config_files_once("fireplace.yaml")
    } else {
        Vec::with_capacity(3)
    };
    if cfg!(debug_assertions) {
        if let Ok(mut cwd) = std::env::current_dir() {
            cwd.push("fireplace.yaml");
            locations.push(cwd);
        }
    }
    locations.push(PathBuf::from("/etc/fireplace/fireplace.yaml"));
    locations.push(PathBuf::from("/etc/fireplace.yaml"));
    let (config_path, config) = try_config_locations(&locations);

    // Initialize logger
    let _guard = logger::init(&config.logging);

    slog_scope::info!("Version: {}", std::env!("CARGO_PKG_VERSION"));
    slog_scope::debug!("Debug build ({})", std::env!("GIT_HASH"));
    slog_scope::info!("Fireplace starting up with {}.",
        config_path.map(|x| format!("config at {}", x.display()))
        .unwrap_or(String::from("default config")));
    
    let mut event_loop = EventLoop::try_new().with_context(|| "Failed to initialize event loop")?;
    let mut display = Display::new();
    display.add_socket_auto()?;
    event_loop.handle()
        .insert_source(
            Generic::from_fd(display.get_poll_fd(), Interest::READ, Mode::Level),
            move |_, _, state: &mut Fireplace| {
                let display = state.display.clone();
                let mut display = display.borrow_mut();
                match display.dispatch(std::time::Duration::from_millis(0), state) {
                    Ok(_) => {
                        Ok(PostAction::Continue)
                    },
                    Err(e) => {
                        slog_scope::error!("I/O error on the Wayland display: {}", e);
                        state.should_stop = true;
                        Err(e)
                    }
                }
            },
        )
        .expect("Failed to init the wayland event source.");

    let mut state = Fireplace::new(config, display);
    backend::initial_backend_auto(&mut event_loop, &state)?;
    // TODO, flusing clients should be an idle thing
    event_loop.run(std::time::Duration::from_millis(16), &mut state, |state| {
        let display = state.display.clone();
        display.borrow_mut().flush_clients(state);
    })?;

    Ok(())
}
