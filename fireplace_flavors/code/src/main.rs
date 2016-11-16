#[macro_use]
extern crate slog;
extern crate slog_html;
extern crate slog_journald;
extern crate slog_scope;
extern crate slog_stdlog;
extern crate slog_stream;
extern crate slog_term;

extern crate fireplace_lib;
extern crate wlc;


use fireplace_lib::callback::IntoCallback;
use fireplace_lib::handlers::*;
use fireplace_lib::handlers::keyboard::KeyPattern;
use fireplace_lib::handlers::workspaces::modes;
use std::collections::HashMap;

use wlc::{Callback, Key, KeyState, Modifier, View};

mod logger;

fn main() {
    logger::init(logger::Logging::default());

    let mut keyboard_handler = KeyboardHandler::new();
    keyboard_handler.register(KeyPattern::new(KeyState::Pressed, Modifier::Logo, Key::Esc),
                              |_, _: Option<&_>| { wlc::terminate(); });
    keyboard_handler.register(KeyPattern::new(KeyState::Pressed, Modifier::Logo | Modifier::Shift, Key::Q),
                              |_, view: Option<&View>| if let Some(x) = view {
                                  x.close();
                              });

    let handlers: Vec<Box<Callback + 'static>> = vec![
        Box::new(StoreHandler::new().into_callback()),
        Box::new(geometry::GeometryHandler::new().into_callback()),
        Box::new(geometry::GapsHandler::default().into_callback()),
        Box::new(render::conrod::ConrodHandler::new().into_callback()),
        Box::new(render::conrod::provider::BackgroundHandler::default()
            .into_callback()),
        Box::new(render::conrod::provider::StatusbarHandler::new(
            render::conrod::provider::StatusbarConfig::default()
        ).into_callback()),
        Box::new(workspaces::WorkspaceHandler::new(
            workspaces::WorkspacesConfig {
                spaces: {
                    let mut map = HashMap::new();
                    map.insert(String::from("default"),
                        workspaces::WorkspaceConfig {
                            name: None,
                            mode: modes::AnyModeConfig::Bsp(modes::bsp::BSPConfig {
                                starting_orientation: modes::bsp::Orientation::Horizontal,
                                keys: modes::bsp::KeyPatterns {
                                    focus_left: Some(KeyPattern::new(
                                        KeyState::Pressed, Modifier::Ctrl, Key::Left
                                    )),
                                    focus_right: Some(KeyPattern::new(
                                        KeyState::Pressed, Modifier::Ctrl, Key::Right
                                    )),
                                    focus_up: Some(KeyPattern::new(
                                        KeyState::Pressed, Modifier::Ctrl, Key::Up
                                    )),
                                    focus_down: Some(KeyPattern::new(
                                        KeyState::Pressed, Modifier::Ctrl, Key::Down
                                    )),
                                    move_left: Some(KeyPattern::new(
                                        KeyState::Pressed, Modifier::Logo | Modifier::Ctrl, Key::Left
                                    )),
                                    move_right: Some(KeyPattern::new(
                                        KeyState::Pressed, Modifier::Logo | Modifier::Ctrl, Key::Right
                                    )),
                                    move_up: Some(KeyPattern::new(
                                        KeyState::Pressed, Modifier::Logo | Modifier::Ctrl, Key::Up
                                    )),
                                    move_down: Some(KeyPattern::new(
                                        KeyState::Pressed, Modifier::Logo | Modifier::Ctrl, Key::Down
                                    )),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                        });
                    map
                },
                ..Default::default()
            }
        ).into_callback()),
        Box::new(FocusHandler::default().into_callback()),
        Box::new(keyboard_handler.into_callback()),
    ];

    wlc::init(handlers.into_callback()).unwrap();
}
