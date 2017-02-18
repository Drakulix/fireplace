use callback::{AsWrapper, IntoCallback, Wrapper};
use handlers::keyboard::KeyPattern;
use handlers::workspaces::modes::{AnyModeConfig, AnyModeWrap, Mode};
use slog;
use slog_scope;
use wlc::{Callback, Geometry, Key, KeyState, Modifiers, Output, Point, ResizeEdge, Size, View, ViewState,
          WeakView};

/// A `Mode` that lets you pull one `View` into fullscreen operation while
/// letting another wrapped `Mode` handle the rest
pub struct Fullscreen {
    active: Option<(WeakView, Geometry)>,
    mode: Box<AnyModeWrap>,
    keys: KeyPatterns,
    logger: slog::Logger,
}

/// Configuration of `Fullscreen`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct FullscreenConfig {
    /// Configuration of the wrapped `Mode`
    pub mode: Box<AnyModeConfig>,
    /// Key configuration
    pub keys: KeyPatterns,
}

/// `KeyPattern`s toggling fullscreen
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Deserialize)]
pub struct KeyPatterns {
    /// Toggle fullscreen operation for the currently focused `View`
    pub toggle: Option<KeyPattern>,
}

/// Convinient type alias for a wrapped `Fullscreen`
pub type FullscreenWrap = Wrapper<Fullscreen>;

impl AsWrapper for Fullscreen {
    fn child(&mut self) -> Option<&mut Callback> {
        Some(&mut self.mode)
    }
}

impl Mode for Wrapper<Fullscreen> {
    type Arguments = FullscreenConfig;

    fn new(arguments: Self::Arguments) -> Self {
        Fullscreen {
                active: None,
                mode: Box::new(AnyModeWrap::new(*arguments.mode)),
                keys: arguments.keys,
                logger: slog_scope::logger().new(o!("instance" => "Fullscreen")),
            }
            .into_callback()
    }

    fn len(&self) -> usize {
        self.mode.len() + self.active.as_ref().map(|_| 1).unwrap_or(0)
    }
}

impl Callback for Wrapper<Fullscreen> {
    fn output_resolution(&mut self, output: &Output, from: Size, to: Size) {
        let current = self.active.clone();
        if self.active.is_some() {
            self.set_fullscreen(None);
        }
        self.mode.output_resolution(output, from, to);
        if let Some((view, _)) = current {
            view.run(|view| self.set_fullscreen(Some(view)));
        }
    }

    fn keyboard_key(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, key: Key,
                    state: KeyState)
                    -> bool {
        if let Some(view) = view {
            match Some(KeyPattern::new(state, modifiers.mods, key)) {
                x if x == self.keys.toggle => {
                    let active = self.has_active();
                    if active {
                        self.set_fullscreen(None);
                    } else {
                        self.set_fullscreen(Some(view));
                    }
                    true
                }
                _ => self.mode.keyboard_key(Some(view), time, modifiers, key, state),
            }
        } else {
            self.mode.keyboard_key(None, time, modifiers, key, state)
        }
    }

    fn view_request_state(&mut self, view: &View, state: ViewState::Flags, toggle: bool) {
        match state {
            x if x.contains(ViewState::Fullscreen) => {
                if self.active.is_some() {
                    self.set_fullscreen(None)
                } else if toggle {
                    self.set_fullscreen(Some(view))
                }
            }
            _ => self.mode.view_request_state(view, state, toggle),
        };
    }
}

impl Fullscreen {
    fn has_active(&self) -> bool {
        self.active.as_ref().is_some()
    }

    fn set_fullscreen(&mut self, view: Option<&View>) {
        if let Some((active, geo)) = self.active.clone() {
            debug!(self.logger, "Un-fullscreening {:?}", active);
            active.run(|active_view| {
                active_view.set_state(ViewState::Fullscreen, false);
                active_view.set_geometry(ResizeEdge::Null, geo);
            });
        }
        self.active = None;

        if let Some(view) = view {
            debug!(self.logger, "Fullscreening {:?}", view);

            let geo = view.geometry();
            view.set_geometry(ResizeEdge::Null,
                              Geometry {
                                  origin: Point { x: 0, y: 0 },
                                  size: view.output().virtual_resolution(),
                              });
            view.set_state(ViewState::Fullscreen, true);
            view.bring_to_front();

            self.active = Some((view.weak_reference(), geo));
        }
    }
}
