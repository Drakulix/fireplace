extern crate serde_json;
extern crate serde;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate slog;
extern crate slog_scope;
extern crate slog_html;
extern crate slog_journald;
extern crate slog_stdlog;
extern crate slog_stream;
extern crate slog_term;

extern crate wlc;
#[macro_use]
extern crate fireplace_lib;

use fireplace_lib::callback::IntoCallback;
use fireplace_lib::handlers::*;
use fireplace_lib::handlers::keyboard::KeyHandler;

use std::env;
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

use wlc::{Callback, View};

mod logger;
mod config;
pub use self::config::Config;

fn main() {

    // Parse configuration and Initialize logger

    let mut config: Config = serde_json::from_str(&env::home_dir()
            .map(|x| {
                env::var("XDG_CONFIG_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| x.join(".config"))
                    .join(".fireplace.json")
            })
            .and_then(|x| fs::create_dir_all(x.parent().unwrap()).ok().map(|_| x))
            .and_then(|x| {
                OpenOptions::new()
                    .read(true)
                    .open(x)
                    .ok()
            })
            .and_then(|mut x| {
                let mut contents = String::new();
                x.read_to_string(&mut contents).ok().map(|_| contents)
            })
            .unwrap_or_else(|| String::from("{}")))
        .unwrap();

    logger::init(config.logging);


    // Initialize the key combinations

    let mut keyboard_handler = KeyboardHandler::new();

    for (command, pattern) in config.keys.drain() {
        keyboard_handler.register(pattern, {
            struct GlobalKeyHandler {
                command: String,
            }
            impl KeyHandler for GlobalKeyHandler {
                fn handle_key(&mut self, _time: u32, _view: Option<&View>) {
                    match &*self.command {
                        #[cfg(feature = "conrod_ui")]
                        "terminate" => wlc::terminate(),
                        x => {
                            warn!(slog_scope::logger(),
                                  "Unknown command {}. Ignoring KeyBinding",
                                  x)
                        }
                    };
                }
            }

            GlobalKeyHandler { command: command }
        });
    }

    for (command, pattern) in config.view.keys.drain() {
        keyboard_handler.register(pattern, {
            struct ViewKeyHandler {
                command: String,
            }
            impl KeyHandler for ViewKeyHandler {
                fn handle_key(&mut self, _time: u32, view: Option<&View>) {
                    if let Some(view) = view {
                        match &*self.command {
                            "close" => view.close(),
                            x => {
                                warn!(slog_scope::logger(),
                                      "Unknown command {}. Ignoring KeyBinding",
                                      x)
                            }
                        };
                    }
                }
            }

            ViewKeyHandler { command: command }
        });
    }

    for (command, pattern) in config.exec.keys.drain() {
        keyboard_handler.register(pattern, {
            struct ExecKeyHandler {
                command: String,
            }
            impl KeyHandler for ExecKeyHandler {
                fn handle_key(&mut self, _time: u32, _view: Option<&View>) {
                    use std::process;

                    match Command::new(env::var("SHELL").unwrap_or_else(|_| String::from("sh")))
                        .arg("-c")
                        .arg(self.command.clone())
                        .stdin(process::Stdio::null())
                        .stdout(process::Stdio::null())
                        .stderr(process::Stdio::null())
                        .spawn() {
                        Ok(_) => {}
                        Err(x) => {
                            warn!(slog_scope::logger(),
                                  "Command {} failed with {}",
                                  self.command,
                                  x)
                        }
                    };
                }
            }

            ExecKeyHandler { command: command }
        });
    }


    // load the other handlers are start the compositor

    let mut handlers: Vec<Box<Callback>> = Vec::new();

    handlers.push(Box::new(StoreHandler::new().into_callback()));
    handlers.push(Box::new(OutputConfigHandler::new(config.outputs)));
    handlers.push(Box::new(geometry::GeometryHandler::new().into_callback()));
    handlers.push(Box::new(geometry::GapsHandler::new(config.ui.gaps).into_callback()));

    #[cfg(feature = "conrod_ui")]
    {
        handlers.push(Box::new(render::conrod::ConrodHandler::new().into_callback()));

        handlers.push(Box::new(render::conrod::provider::background::BackgroundHandler::new(config.ui
                .background)
            .into_callback()));

        handlers.push(Box::new(render::conrod::provider::statusbar::StatusbarHandler::new(config.ui
                .statusbar)
            .into_callback()));

        handlers.push(Box::new(render::ScreenshotHandler::new(config.screenshot)));
    }

    handlers.push(Box::new(workspaces::WorkspaceHandler::new(config.workspace).into_callback()));
    handlers.push(Box::new(FocusHandler::new(config.focus).into_callback()));
    handlers.push(Box::new(PointerHandler::new()));
    handlers.push(Box::new(keyboard_handler.into_callback()));

    wlc::init(handlers.into_callback()).unwrap();
}
