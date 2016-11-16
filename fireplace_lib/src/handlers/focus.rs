use slog;
use slog_scope;
use wlc::{Button, ButtonState, Callback, Modifiers, Output, Point, View, ViewState};

/// Handler to direct window focus by given properties
///
/// ## Dependencies
///
/// This handler has no dependencies.
///
pub struct FocusHandler {
    config: FocusConfig,
    logger: slog::Logger,
}

impl Default for FocusHandler {
    fn default() -> FocusHandler {
        FocusHandler::new(FocusConfig::default())
    }
}

/// Configuration for a `FocusHandler`
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FocusConfig {
    /// Always focus a newly created `View`
    #[serde(default = "::handlers::focus::default_on_creation")]
    pub on_creation: bool,
    /// Switch focus upon clicking onto an unfocused `View`
    #[serde(default = "::handlers::focus::default_on_click")]
    pub on_click: bool,
    /// Switch focus when the mouse enters a previously unfocused `View`
    #[serde(default = "::handlers::focus::default_follows_mouse")]
    pub follows_mouse: bool,
}

impl Default for FocusConfig {
    fn default() -> FocusConfig {
        FocusConfig {
            on_creation: default_on_creation(),
            on_click: default_on_click(),
            follows_mouse: default_follows_mouse(),
        }
    }
}

fn default_on_creation() -> bool {
    true
}

fn default_on_click() -> bool {
    true
}

fn default_follows_mouse() -> bool {
    false
}

impl FocusHandler {
    /// Initialize a new `FocusHandler` with a given `FocusConfig`
    pub fn new(config: FocusConfig) -> FocusHandler {
        FocusHandler {
            config: config,
            logger: slog_scope::logger().new(o!("handler" => "Focus")),
        }
    }
}

impl Callback for FocusHandler {
    fn view_created(&mut self, view: &View) -> bool {
        debug!(self.logger.new(o!("view" => format!("{:?}", view))),
               "Focusing");
        if self.config.on_creation {
            view.focus();
        }
        true
    }

    fn view_focus(&mut self, view: &View, focus: bool) {
        debug!(self.logger.new(o!("view" => format!("{:?}", view))),
               "Focus changed: {}",
               focus);
        view.set_state(ViewState::Activated, focus);
    }

    fn view_destroyed(&mut self, view: &View) {
        if view.state().contains(ViewState::Activated) {
            debug!(self.logger.new(o!("view" => format!("{:?}", view))),
                   "Finding new focus");
            if let Some(view) = view.output().views().last_mut() {
                view.focus()
            }
        }
    }

    fn pointer_motion(&mut self, view: Option<&View>, _time: u32, origin: Point) -> bool {
        if self.config.follows_mouse {
            Output::with_focused_output(|focused_output| if let Some(new_focus_view) =
                focused_output.views()
                    .iter()
                    .filter(|view| {
                        let geometry = view.geometry();
                        view.visibility() == focused_output.visibility() && origin.x > geometry.origin.x &&
                        origin.y > geometry.origin.y &&
                        origin.x < geometry.origin.x + geometry.size.w as i32 &&
                        origin.y < geometry.origin.y + geometry.size.h as i32
                    })
                    .last() {
                if view.and_then(|view| view.parent()) != Some(new_focus_view) {
                    new_focus_view.focus();
                }
            });
        }
        false
    }

    fn pointer_button(&mut self, _view: Option<&View>, _time: u32, _modifiers: Modifiers, button: Button,
                      _state: ButtonState, origin: Point)
                      -> bool {
        if self.config.on_click && button == Button::Left {
            Output::with_focused_output(|focused_output| if let Some(new_focus_view) =
                focused_output.views()
                    .iter()
                    .filter(|view| {
                        let geometry = view.geometry();
                        view.visibility() == focused_output.visibility() && origin.x > geometry.origin.x &&
                        origin.y > geometry.origin.y &&
                        origin.x < geometry.origin.x + geometry.size.w as i32 &&
                        origin.y < geometry.origin.y + geometry.size.h as i32
                    })
                    .last() {
                if !new_focus_view.state().contains(ViewState::Activated) {
                    new_focus_view.focus();
                    true
                } else {
                    false
                }
            } else {
                false
            })
        } else {
            false
        }
    }

    fn view_request_state(&mut self, view: &View, state: ViewState::Flags, toggle: bool) {
        match state {
            x if x.contains(ViewState::Activated) && toggle => {
                view.focus();
            }
            _ => {}
        }
    }
}
