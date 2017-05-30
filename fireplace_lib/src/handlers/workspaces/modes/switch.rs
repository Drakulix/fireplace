use callback::{AsWrapper, IntoCallback, Wrapper};
use handlers::geometry::InitialViewGeometry;
use handlers::keyboard::KeyPattern;
use handlers::store::Store;
use handlers::workspaces::modes::{AnyModeConfig, AnyModeWrap, Mode};
use slog;
use slog_scope;
use wlc::{Callback, Key, KeyState, Modifiers, Output, ResizeEdge, Size, View, WeakView};

/// A `Mode` that lets you conviniently switch between different `Mode`s
pub struct Switch {
    active: Option<usize>,
    modes: Vec<AnyModeWrap>,
    views: Vec<WeakView>,
    keys: KeyPatterns,
    logger: slog::Logger,
}

/// Configuration for `Switch`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct SwitchConfig {
    /// Configuration of switchable `Mode`s
    pub modes: Vec<AnyModeConfig>,
    /// Key configuration
    pub keys: KeyPatterns,
}

/// `KeyPattern`s switching modes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Deserialize)]
pub struct KeyPatterns {
    /// Switch to the previous mode
    pub switch_prev: Option<KeyPattern>,
    /// Switch to the next mode
    pub switch_next: Option<KeyPattern>,
}

/// Convinient type alias for a wrapped `Switch`
pub type SwitchWrap = Wrapper<Switch>;

impl AsWrapper for Switch {
    fn child(&mut self) -> Option<&mut Callback> {
        let index = self.active;
        if let Some(index) = index {
            Some(&mut self.modes[index])
        } else {
            None
        }
    }
}

impl Mode for Wrapper<Switch> {
    type Arguments = SwitchConfig;

    fn new(mut arguments: Self::Arguments) -> Self {
        Switch {
                active: None,
                modes: arguments.modes.drain(..).map(AnyModeWrap::new).collect(),
                views: Vec::new(),
                keys: arguments.keys,
                logger: slog_scope::logger().new(o!("instance" => "Switch")),
            }
            .into_callback()
    }

    fn len(&self) -> usize {
        self.views.len()
    }
}

impl Callback for Wrapper<Switch> {
    fn view_created(&mut self, view: &View) -> bool {
        slog_scope::scope(self.logger.clone(), || {
            self.views.push(view.weak_reference());
            let index = self.active;
            if let Some(index) = index {
                self.modes[index].view_created(view)
            } else {
                false
            }
        })
    }

    fn output_focus(&mut self, output: &Output, focus: bool) {
        for hook in &mut self.modes {
            hook.output_focus(output, focus);
        }
    }

    fn output_resolution(&mut self, output: &Output, from: Size, to: Size) {
        for hook in &mut self.modes {
            hook.output_resolution(output, from, to);
        }
    }

    fn output_context_created(&mut self, output: &Output) {
        for hook in &mut self.modes {
            hook.output_context_created(output);
        }
    }

    fn output_context_destroyed(&mut self, output: &Output) {
        for hook in self.modes.iter_mut().rev() {
            hook.output_context_destroyed(output);
        }
    }

    fn keyboard_key(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, key: Key,
                    state: KeyState)
                    -> bool {
        let active = self.active;
        if let Some(active) = active {
            match Some(KeyPattern::new(state, modifiers.mods, key)) {
                x if x == self.keys.switch_prev => {
                    debug!(self.logger, "Switching to prev mode");
                    let index = if active == 0 {
                        (self.views.len() as i32 - 1) as usize
                    } else {
                        active
                    };
                    let views = self.views.clone();
                    for view in views {
                        view.run(|view| {
                                     self.modes[active].view_destroyed(view);
                                     let initial = view.get::<InitialViewGeometry>().unwrap();
                                     view.set_geometry(ResizeEdge::Null, *initial.read().unwrap());
                                     self.modes[index].view_created(view);
                                 });
                    }
                    self.active = Some(index);
                    true
                }
                x if x == self.keys.switch_next => {
                    debug!(self.logger, "Switching to next mode");
                    let index = if active == self.views.len() {
                        0
                    } else {
                        active + 1
                    };
                    let views = self.views.clone();
                    for view in views {
                        view.run(|view| {
                                     self.modes[active].view_destroyed(view);
                                     let initial = view.get::<InitialViewGeometry>().unwrap();
                                     view.set_geometry(ResizeEdge::Null, *initial.read().unwrap());
                                     self.modes[index].view_created(view);
                                 });
                    }
                    self.active = Some(index);
                    true
                }
                _ => self.modes[active].keyboard_key(view, time, modifiers, key, state),
            }
        } else {
            false
        }
    }

    fn compositor_ready(&mut self) {
        for hook in &mut self.modes {
            hook.compositor_ready()
        }
    }

    fn compositor_terminate(&mut self) {
        for hook in self.modes.iter_mut().rev() {
            hook.compositor_terminate()
        }
    }
}
