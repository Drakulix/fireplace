//! Workspace handling
//!
//! Workspace handling is split in three parts in this compositor.
//! At first the `WorkspaceHandler` dispatches all incoming events to the
//! relevant workspaces (most of the time, only the currently active one),
//! making them able to behave like they manage a completely
//! seperate `Output`.
//!
//! A workspace is managed by a `Mode`, which describes how `View`s are
//! managed on said workspace.
//!
//! Examples are `Floating` or `BSP` (i3-like **B**inary
//! **S**plit  **P**artitioning). `Mode`s themselves can be combined by
//! special `Mode`s that dispatch events on certain conditions, examples are
//! `Combined`, `Fullscreen` and `Switch`.
//!
//! Using just a `Mode` as an handler without the `WorkspaceHandler` will
//! effectively result in a functional `Output` without any workspaces.
//!

use callback::{IntoCallback, Wrapper};
use handlers::keyboard::KeyPattern;
use handlers::store::{Store, StoreKey};
use linked_hash_map::LinkedHashMap;
use slog;
use slog_scope;

use std::collections::HashMap;
use std::str::FromStr;
use wlc::*;
#[cfg(feature = "render")]
use wlc::render::*;

pub mod modes;
use self::modes::AnyModeConfig;

mod workspace;
use self::workspace::*;

/// Handler that creates and manages `Workspace`s
///
/// ## Dependencies
///
/// - [`StoreHandler`](../struct.StoreHandler.html)
///
/// ### Optional - but must be loaded before to have an effect
///
/// - [`GeometryHandler`](../geometry/struct.GeometryHandler.html)
/// - [`GapsHandler`](../geometry/struct.GapsHandler.html)
/// - [`StatusbarHandler`](../render/conrod/provider/statusbar/struct.
/// StatusbarHandler.html)
///
pub struct WorkspaceHandler {
    mode_arguments: HashMap<u8, (Option<String>, AnyModeConfig)>,
    workspaces: LinkedHashMap<u8, Wrapper<Workspace>>,
    size: Size,
    keys: KeyPatterns,
    logger: slog::Logger,
}

/// Configuration for the `WorkspaceHandler`
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspacesConfig {
    /// A `HashMap` of an `WorkspaceConfig` for each name of
    /// an `Workspace`. The name `default` or `generic` might be used for a
    /// generic configuration.
    #[serde(default = "::handlers::workspaces::default_spaces")]
    pub spaces: HashMap<String, WorkspaceConfig>,
    /// Key configuration
    #[serde(default)]
    pub keys: KeyPatterns,
}

impl Default for WorkspacesConfig {
    fn default() -> WorkspacesConfig {
        WorkspacesConfig {
            spaces: default_spaces(),
            keys: KeyPatterns::default(),
        }
    }
}

fn default_spaces() -> HashMap<String, WorkspaceConfig> {
    let mut map = HashMap::new();
    map.insert(String::from("default"), WorkspaceConfig::default());
    map
}

/// Configuration for a single Workspace
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceConfig {
    /// Name of the workspace, used for Ui
    #[serde(default)]
    pub name: Option<String>,
    /// Configuration for the `Mode` that shall handle the `View`s
    /// of this `Workspace`
    #[serde(default)]
    pub mode: AnyModeConfig,
}

/// `KeyPattern`s triggering workspace related actions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyPatterns {
    /// Switch to next workspace
    pub next: Option<KeyPattern>,
    /// Switch to previous workspace
    pub prev: Option<KeyPattern>,

    /// Switch to workspace 1
    pub workspace1: Option<KeyPattern>,
    /// Switch to workspace 2
    pub workspace2: Option<KeyPattern>,
    /// Switch to workspace 3
    pub workspace3: Option<KeyPattern>,
    /// Switch to workspace 4
    pub workspace4: Option<KeyPattern>,
    /// Switch to workspace 5
    pub workspace5: Option<KeyPattern>,
    /// Switch to workspace 6
    pub workspace6: Option<KeyPattern>,
    /// Switch to workspace 7
    pub workspace7: Option<KeyPattern>,
    /// Switch to workspace 8
    pub workspace8: Option<KeyPattern>,
    /// Switch to workspace 9
    pub workspace9: Option<KeyPattern>,
    /// Switch to workspace 10
    pub workspace10: Option<KeyPattern>,
    /// Switch to workspace 11
    pub workspace11: Option<KeyPattern>,
    /// Switch to workspace 12
    pub workspace12: Option<KeyPattern>,
    /// Switch to workspace 13
    pub workspace13: Option<KeyPattern>,
    /// Switch to workspace 14
    pub workspace14: Option<KeyPattern>,
    /// Switch to workspace 15
    pub workspace15: Option<KeyPattern>,
    /// Switch to workspace 16
    pub workspace16: Option<KeyPattern>,
    /// Switch to workspace 17
    pub workspace17: Option<KeyPattern>,
    /// Switch to workspace 18
    pub workspace18: Option<KeyPattern>,
    /// Switch to workspace 19
    pub workspace19: Option<KeyPattern>,
    /// Switch to workspace 20
    pub workspace20: Option<KeyPattern>,
    /// Switch to workspace 21
    pub workspace21: Option<KeyPattern>,
    /// Switch to workspace 22
    pub workspace22: Option<KeyPattern>,
    /// Switch to workspace 23
    pub workspace23: Option<KeyPattern>,
    /// Switch to workspace 24
    pub workspace24: Option<KeyPattern>,
    /// Switch to workspace 25
    pub workspace25: Option<KeyPattern>,
    /// Switch to workspace 26
    pub workspace26: Option<KeyPattern>,
    /// Switch to workspace 27
    pub workspace27: Option<KeyPattern>,
    /// Switch to workspace 28
    pub workspace28: Option<KeyPattern>,
    /// Switch to workspace 29
    pub workspace29: Option<KeyPattern>,
    /// Switch to workspace 30
    pub workspace30: Option<KeyPattern>,
    /// Switch to workspace 31
    pub workspace31: Option<KeyPattern>,
    /// Switch to workspace 32
    pub workspace32: Option<KeyPattern>,

    /// Move focused `View` to workspace 1
    pub moveto_workspace1: Option<KeyPattern>,
    /// Move focused `View` to workspace 2
    pub moveto_workspace2: Option<KeyPattern>,
    /// Move focused `View` to workspace 3
    pub moveto_workspace3: Option<KeyPattern>,
    /// Move focused `View` to workspace 4
    pub moveto_workspace4: Option<KeyPattern>,
    /// Move focused `View` to workspace 5
    pub moveto_workspace5: Option<KeyPattern>,
    /// Move focused `View` to workspace 6
    pub moveto_workspace6: Option<KeyPattern>,
    /// Move focused `View` to workspace 7
    pub moveto_workspace7: Option<KeyPattern>,
    /// Move focused `View` to workspace 8
    pub moveto_workspace8: Option<KeyPattern>,
    /// Move focused `View` to workspace 9
    pub moveto_workspace9: Option<KeyPattern>,
    /// Move focused `View` to workspace 10
    pub moveto_workspace10: Option<KeyPattern>,
    /// Move focused `View` to workspace 11
    pub moveto_workspace11: Option<KeyPattern>,
    /// Move focused `View` to workspace 12
    pub moveto_workspace12: Option<KeyPattern>,
    /// Move focused `View` to workspace 13
    pub moveto_workspace13: Option<KeyPattern>,
    /// Move focused `View` to workspace 14
    pub moveto_workspace14: Option<KeyPattern>,
    /// Move focused `View` to workspace 15
    pub moveto_workspace15: Option<KeyPattern>,
    /// Move focused `View` to workspace 16
    pub moveto_workspace16: Option<KeyPattern>,
    /// Move focused `View` to workspace 17
    pub moveto_workspace17: Option<KeyPattern>,
    /// Move focused `View` to workspace 18
    pub moveto_workspace18: Option<KeyPattern>,
    /// Move focused `View` to workspace 19
    pub moveto_workspace19: Option<KeyPattern>,
    /// Move focused `View` to workspace 20
    pub moveto_workspace20: Option<KeyPattern>,
    /// Move focused `View` to workspace 21
    pub moveto_workspace21: Option<KeyPattern>,
    /// Move focused `View` to workspace 22
    pub moveto_workspace22: Option<KeyPattern>,
    /// Move focused `View` to workspace 23
    pub moveto_workspace23: Option<KeyPattern>,
    /// Move focused `View` to workspace 24
    pub moveto_workspace24: Option<KeyPattern>,
    /// Move focused `View` to workspace 25
    pub moveto_workspace25: Option<KeyPattern>,
    /// Move focused `View` to workspace 26
    pub moveto_workspace26: Option<KeyPattern>,
    /// Move focused `View` to workspace 27
    pub moveto_workspace27: Option<KeyPattern>,
    /// Move focused `View` to workspace 28
    pub moveto_workspace28: Option<KeyPattern>,
    /// Move focused `View` to workspace 29
    pub moveto_workspace29: Option<KeyPattern>,
    /// Move focused `View` to workspace 30
    pub moveto_workspace30: Option<KeyPattern>,
    /// Move focused `View` to workspace 31
    pub moveto_workspace31: Option<KeyPattern>,
    /// Move focused `View` to workspace 32
    pub moveto_workspace32: Option<KeyPattern>,
}

/// Key for receiving the currently active workspace's number and name
/// from any `Output`s [`Store`](../trait.Store.html).
pub struct ActiveWorkspace {
    num: u8,
    name: String,
}

impl ActiveWorkspace {
    /// Returns the currently active workspace's number
    pub fn num(&self) -> u8 {
        self.num
    }

    /// Returns the currently active workspace's name
    ///
    /// This name can be empty, but is otherwise unique.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl StoreKey for ActiveWorkspace {
    type Value = ActiveWorkspace;
}

/// Key for receiving the currently workspace's number and name
/// of any `View`s [`Store`](../trait.Store.html).
pub struct ViewWorkspace {
    num: u8,
    name: String,
}

impl ViewWorkspace {
    /// Returns the currently active workspace's number
    pub fn num(&self) -> u8 {
        self.num
    }

    /// Returns the currently active workspace's name
    ///
    /// This name can be empty, but is otherwise unique.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl StoreKey for ViewWorkspace {
    type Value = ViewWorkspace;
}

impl WorkspaceHandler {
    /// Initialize a new `WorkspaceHandler` with a given `WorkspaceHandler`
    pub fn new(mut arguments: WorkspacesConfig) -> WorkspaceHandler {
        WorkspaceHandler {
            mode_arguments: {
                let mut new_map = HashMap::new();
                for (key, value) in arguments.spaces.drain() {
                    if match u8::from_str(&key) {
                        Ok(key) => new_map.insert(key, (value.name, value.mode)).is_some(),
                        Err(x) => {
                            match &*key {
                                "default" | "generic" => {
                                    new_map.insert(0, (value.name, value.mode)).is_some()
                                }
                                _ => {
                                    error!(slog_scope::logger(),
                                           "{} not a valid Workspace. Needs to be a number (> 0)
                                           or \"default\" / \"generic\". {}",
                                           key,
                                           x);
                                    true
                                }
                            }
                        }
                    } {
                        error!(slog_scope::logger(), "Workspace used twice! {}", key);
                    }
                }
                new_map
            },
            workspaces: LinkedHashMap::new(),
            size: Size { w: 0, h: 0 },
            keys: arguments.keys,
            logger: slog_scope::logger().new(o!("handler" => "Workspace")),
        }
    }

    fn next_available(&mut self, output: &Output) -> Option<u8> {
        for i in 1..32 {
            let contains = self.workspaces.contains_key(&i);
            if contains {
                let space = self.workspaces.get(&i).unwrap();
                if !space.active() {
                    return Some(space.number);
                }
            } else {
                self.create_workspace(output, i);
                return Some(i);
            }
        }
        None
    }

    fn create_workspace(&mut self, output: &Output, index: u8) {
        debug!(self.logger,
               "Creating Workspace {:?} for Output {:?}",
               index,
               output);
        match self.mode_arguments.get(&index) {
            Some(&(ref name, ref arg)) => {
                self.workspaces.insert(index,
                                       Workspace::new(index, name.clone().unwrap_or_default(), arg.clone())
                                           .into_callback())
            }
            None => {
                let (name, arg) = self.mode_arguments[&0].clone();
                self.workspaces.insert(index,
                                       Workspace::new(index, name.unwrap_or_default(), arg).into_callback())
            }
        };
        self.workspaces.get_mut(&index).unwrap().output_resolution(output, self.size, self.size);
    }

    fn destroy_workspace(&mut self, index: u8) {
        debug!(self.logger, "Destroying empty Workspace {:?}", index);
        self.workspaces.remove(&index);
    }

    /// Move the given `View` to the `Workspace` of the given `index`
    pub fn moveto_workspace(&mut self, view: &View, index: u8) {
        if !self.workspaces.contains_key(&index) {
            self.create_workspace(view.output(), index);
        }

        debug!(self.logger,
               "Moving View {:?} to Workspace {:?}",
               view,
               index);
        {
            let lock = view.get::<ViewWorkspace>();
            match lock.as_ref().and_then(|x| x.read().ok()) {
                Some(active) => {
                    let mut space = self.workspaces.get_mut(&active.num()).unwrap();
                    space.view_destroyed(view);
                    space.restore_focus();
                }
                None => unreachable!(),
            };
        }

        view.insert::<ViewWorkspace>(ViewWorkspace {
            num: index,
            name: self.workspaces[&index].name.clone(),
        });
        self.workspaces[&index].view_created(view);
    }

    /// Returns an array of currently active `Workspace` indicies
    pub fn active_spaces(&self) -> Vec<u8> {
        self.workspaces.keys().cloned().collect()
    }

    /// Switch the currently focused `Output` to the workspace of the given
    /// `index`
    pub fn switch_workspace(&mut self, index: u8) {
        Output::with_focused_output(move |output| {
            // Create workspace if necessary
            if !self.workspaces.contains_key(&index) {
                self.create_workspace(output, index);
            }

            // Switch output if necessary
            let last_output = self.workspaces.get(&index).unwrap().output();
            if last_output != Some(output.weak_reference()) {
                if let Some(new_output) = last_output {
                    new_output.run(|output| {
                        output.focus();
                        if let Some(new_focus_view) =
                            output.views()
                                .iter()
                                .filter(|view| {
                                    let geometry = view.geometry();
                                    let origin = input::pointer::position();
                                    view.visibility() == output.visibility() &&
                                    origin.x > geometry.origin.x &&
                                    origin.y > geometry.origin.y &&
                                    origin.x < geometry.origin.x + geometry.size.w as i32 &&
                                    origin.y < geometry.origin.y + geometry.size.h as i32
                                })
                                .last() {
                            new_focus_view.focus();
                        }
                    });
                    return;
                }
            }

            // Else swap workspace on this monitor
            let name = {
                let space = self.workspaces.get(&index).unwrap();
                space.name.clone()
            };

            match output.insert::<ActiveWorkspace>(ActiveWorkspace {
                num: index,
                name: name,
            }) {
                Some(old) => {
                    if old.num == index {
                        return;
                    }

                    {
                        let old_output =
                            self.workspaces.get_mut(&old.num).expect("ActiveWorkspace was invalid");
                        old_output.output_context_destroyed(output);
                        old_output.output_destroyed(output);
                    }

                    if self.workspaces.get(&old.num).expect("ActiveWorkspace was invalid").is_empty() {
                        self.destroy_workspace(old.num);
                    }
                }
                None => {
                    warn!(slog_scope::logger(),
                          "Output {:?} was previously not assigned a workspace",
                          output)
                }
            };

            debug!(self.logger,
                   "Switching Workspace {:?} on {:?}",
                   index,
                   output);

            let space = self.workspaces.get_mut(&index).unwrap();
            space.output_created(output);
            space.output_context_created(output);
            space.restore_focus();
        })
    }
}

impl Callback for WorkspaceHandler {
    fn output_created(&mut self, output: &Output) -> bool {
        let num = match self.next_available(output) {
            Some(num) => num,
            None => {
                warn!(slog_scope::logger(), "No free workspace. Destroying");
                return false;
            }
        };
        info!(slog_scope::logger(),
              "New output: {:?}. Setting workspace {}",
              output,
              num);

        output.insert::<ActiveWorkspace>(ActiveWorkspace {
            num: num,
            name: self.workspaces[&num].name.clone(),
        });

        self.workspaces[&num].output_created(output);
        true
    }

    fn output_destroyed(&mut self, output: &Output) {
        let lock = output.get::<ActiveWorkspace>();
        if let Some(active) = lock.as_ref().and_then(|x| x.read().ok()) {
            (*self).workspaces[&active.num].output_destroyed(output);
            let is_empty = (*self).workspaces[&active.num].is_empty();
            if is_empty {
                debug!(slog_scope::logger(),
                       "Cleaning up empty workspace {}",
                       &active.num);
                (*self).workspaces.remove(&active.num);
            };
        };
    }

    fn output_focus(&mut self, output: &Output, focus: bool) {
        let lock = output.get::<ActiveWorkspace>();
        if let Some(active) = lock.as_ref().and_then(|x| x.read().ok()) {
            (*self).workspaces[&active.num].output_focus(output, focus);
        };
    }

    fn output_resolution(&mut self, output: &Output, from: Size, to: Size) {
        self.size = to;
        let lock = output.get::<ActiveWorkspace>();
        if let Some(active) = lock.as_ref().and_then(|x| x.read().ok()) {
            (*self).workspaces[&active.num].output_resolution(output, from, to);
        };
    }

    #[cfg(feature = "render")]
    fn output_render_pre(&mut self, output: &mut RenderOutput) {
        let lock = output.get::<ActiveWorkspace>();
        if let Some(active) = lock.as_ref().and_then(|x| x.read().ok()) {
            (*self).workspaces[&active.num].output_render_pre(output);
        };
    }

    #[cfg(not(feature = "render"))]
    fn output_render_pre(&mut self, output: &Output) {
        let lock = output.get::<ActiveWorkspace>();
        if let Some(active) = lock.as_ref().and_then(|x| x.read().ok()) {
            (*self).workspaces[&active.num].output_render_pre(output);
        };
    }

    #[cfg(feature = "render")]
    fn output_render_post(&mut self, output: &mut RenderOutput) {
        let lock = output.get::<ActiveWorkspace>();
        if let Some(active) = lock.as_ref().and_then(|x| x.read().ok()) {
            (*self).workspaces[&active.num].output_render_post(output);
        };
    }

    #[cfg(not(feature = "render"))]
    fn output_render_post(&mut self, output: &Output) {
        let lock = output.get::<ActiveWorkspace>();
        if let Some(active) = lock.as_ref().and_then(|x| x.read().ok()) {
            (*self).workspaces[&active.num].output_render_post(output);
        };
    }

    fn output_context_created(&mut self, output: &Output) {
        let lock = output.get::<ActiveWorkspace>();
        if let Some(active) = lock.as_ref().and_then(|x| x.read().ok()) {
            (*self).workspaces[&active.num].output_context_created(output);
        };
    }

    fn output_context_destroyed(&mut self, output: &Output) {
        let lock = output.get::<ActiveWorkspace>();
        if let Some(active) = lock.as_ref().and_then(|x| x.read().ok()) {
            (*self).workspaces[&active.num].output_context_destroyed(output);
        };
    }

    fn view_created(&mut self, view: &View) -> bool {
        Output::with_focused_output(|focused_output| {
            view.set_output(focused_output);
            let lock = view.output().get::<ActiveWorkspace>();
            let result = match lock.as_ref().and_then(|x| x.read().ok()) {
                Some(active) => {
                    (*self).workspaces[&active.num].view_created(view);
                    view.insert::<ViewWorkspace>(ViewWorkspace {
                        name: active.name.clone(),
                        num: active.num,
                    });
                    true
                }
                None => {
                    error!(slog_scope::logger(), "No active workspace. Illegal state");
                    false
                }
            };
            result
        })
    }

    fn view_destroyed(&mut self, view: &View) {
        let lock = view.get::<ViewWorkspace>();
        match lock.as_ref().and_then(|x| x.read().ok()) {
            Some(space) => {
                self.workspaces.get_mut(&space.num).expect("ViewWorkspace was invalid").view_destroyed(view);
                let lock = view.output().get::<ActiveWorkspace>();
                match lock.as_ref().and_then(|x| x.read().ok()) {
                    Some(active) => {
                        if active.num != space.num &&
                           self.workspaces.get(&space.num).expect("ViewWorkspace was invalid").is_empty() {
                            self.destroy_workspace(space.num);
                        }
                    }
                    None => error!(slog_scope::logger(), "No active workspace. Illegal state"),
                };
            }
            None => {
                warn!(self.logger,
                      "Destroyed View {:?} was not assigned to a workspace",
                      view)
            }
        };
    }

    fn view_focus(&mut self, view: &View, focus: bool) {
        let lock = view.get::<ViewWorkspace>();
        if let Some(space) = lock.as_ref().and_then(|x| x.read().ok()) {
            self.workspaces.get_mut(&space.num).expect("ViewWorkspace was invalid").view_focus(view, focus)
        };
    }

    fn view_move_to_output(&mut self, view: &View, from: &Output, to: &Output) {
        let lock = view.get::<ViewWorkspace>();
        if let Some(space) = lock.as_ref().and_then(|x| x.read().ok()) {
            self.workspaces
                .get_mut(&space.num)
                .expect("ViewWorkspace was invalid")
                .view_move_to_output(view, from, to)
        };
    }

    fn view_request_geometry(&mut self, view: &View, geometry: Geometry) {
        let lock = view.get::<ViewWorkspace>();
        if let Some(space) = lock.as_ref().and_then(|x| x.read().ok()) {
            self.workspaces
                .get_mut(&space.num)
                .expect("ViewWorkspace was invalid")
                .view_request_geometry(view, geometry)
        };
    }

    fn view_request_state(&mut self, view: &View, state: ViewState::Flags, toggle: bool) {
        let lock = view.get::<ViewWorkspace>();
        if let Some(space) = lock.as_ref().and_then(|x| x.read().ok()) {
            self.workspaces
                .get_mut(&space.num)
                .expect("ViewWorkspace was invalid")
                .view_request_state(view, state, toggle)
        };
    }

    fn view_request_move(&mut self, view: &View, origin: Point) {
        let lock = view.get::<ViewWorkspace>();
        if let Some(space) = lock.as_ref().and_then(|x| x.read().ok()) {
            self.workspaces
                .get_mut(&space.num)
                .expect("ViewWorkspace was invalid")
                .view_request_move(view, origin)
        };
    }

    fn view_request_resize(&mut self, view: &View, edges: ResizeEdge::Flags, origin: Point) {
        let lock = view.get::<ViewWorkspace>();
        if let Some(space) = lock.as_ref().and_then(|x| x.read().ok()) {
            self.workspaces
                .get_mut(&space.num)
                .expect("ViewWorkspace was invalid")
                .view_request_resize(view, edges, origin)
        };
    }

    #[cfg(feature = "render")]
    fn view_render_pre(&mut self, view: &mut RenderView) {
        let lock = view.get::<ViewWorkspace>();
        if let Some(space) = lock.as_ref().and_then(|x| x.read().ok()) {
            self.workspaces.get_mut(&space.num).expect("ViewWorkspace was invalid").view_render_pre(view)
        };
    }

    #[cfg(not(feature = "render"))]
    fn view_render_pre(&mut self, view: &View) {
        let lock = view.get::<ViewWorkspace>();
        if let Some(space) = lock.as_ref().and_then(|x| x.read().ok()) {
            self.workspaces.get_mut(&space.num).expect("ViewWorkspace was invalid").view_render_pre(view)
        };
    }

    #[cfg(feature = "render")]
    fn view_render_post(&mut self, view: &mut RenderView) {
        let lock = view.get::<ViewWorkspace>();
        if let Some(space) = lock.as_ref().and_then(|x| x.read().ok()) {
            self.workspaces.get_mut(&space.num).expect("ViewWorkspace was invalid").view_render_post(view)
        };
    }

    #[cfg(not(feature = "render"))]
    fn view_render_post(&mut self, view: &View) {
        let lock = view.get::<ViewWorkspace>();
        if let Some(space) = lock.as_ref().and_then(|x| x.read().ok()) {
            self.workspaces.get_mut(&space.num).expect("ViewWorkspace was invalid").view_render_post(view)
        };
    }

    fn view_properties_updated(&mut self, view: &View, mask: ViewPropertyUpdate::Flags) {
        let lock = view.get::<ViewWorkspace>();
        if let Some(space) = lock.as_ref().and_then(|x| x.read().ok()) {
            self.workspaces
                .get_mut(&space.num)
                .expect("ViewWorkspace was invalid")
                .view_properties_updated(view, mask)
        };
    }

    #[cfg_attr(feature = "cargo-clippy", allow(cyclomatic_complexity))]
    fn keyboard_key(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, key: Key,
                    state: KeyState)
                    -> bool {
        let active = Output::with_focused_output(move |output| {
            let lock = output.get::<ActiveWorkspace>();
            let result = lock.as_ref().and_then(|x| x.read().ok()).expect("ActiveWorkspace was invalid").num;
            result
        });

        match Some(KeyPattern::new(state, modifiers.mods, key)) {
            x if x == self.keys.next => self.switch_workspace(if active == 32 { 0 } else { active + 1 }),
            x if x == self.keys.prev => self.switch_workspace(if active == 0 { 32 } else { active - 1 }),

            x if x == self.keys.workspace1 => self.switch_workspace(1),
            x if x == self.keys.workspace2 => self.switch_workspace(2),
            x if x == self.keys.workspace3 => self.switch_workspace(3),
            x if x == self.keys.workspace4 => self.switch_workspace(4),
            x if x == self.keys.workspace5 => self.switch_workspace(5),
            x if x == self.keys.workspace6 => self.switch_workspace(6),
            x if x == self.keys.workspace7 => self.switch_workspace(7),
            x if x == self.keys.workspace8 => self.switch_workspace(8),
            x if x == self.keys.workspace9 => self.switch_workspace(9),
            x if x == self.keys.workspace10 => self.switch_workspace(10),
            x if x == self.keys.workspace11 => self.switch_workspace(11),
            x if x == self.keys.workspace12 => self.switch_workspace(12),
            x if x == self.keys.workspace13 => self.switch_workspace(13),
            x if x == self.keys.workspace14 => self.switch_workspace(14),
            x if x == self.keys.workspace15 => self.switch_workspace(15),
            x if x == self.keys.workspace16 => self.switch_workspace(16),
            x if x == self.keys.workspace17 => self.switch_workspace(17),
            x if x == self.keys.workspace18 => self.switch_workspace(18),
            x if x == self.keys.workspace19 => self.switch_workspace(19),
            x if x == self.keys.workspace20 => self.switch_workspace(20),
            x if x == self.keys.workspace21 => self.switch_workspace(21),
            x if x == self.keys.workspace22 => self.switch_workspace(22),
            x if x == self.keys.workspace23 => self.switch_workspace(23),
            x if x == self.keys.workspace24 => self.switch_workspace(24),
            x if x == self.keys.workspace25 => self.switch_workspace(25),
            x if x == self.keys.workspace26 => self.switch_workspace(26),
            x if x == self.keys.workspace27 => self.switch_workspace(27),
            x if x == self.keys.workspace28 => self.switch_workspace(28),
            x if x == self.keys.workspace29 => self.switch_workspace(29),
            x if x == self.keys.workspace30 => self.switch_workspace(30),
            x if x == self.keys.workspace31 => self.switch_workspace(31),
            x if x == self.keys.workspace32 => self.switch_workspace(32),

            x if x == self.keys.moveto_workspace1 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 1)
                }
            }
            x if x == self.keys.moveto_workspace2 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 2)
                }
            }
            x if x == self.keys.moveto_workspace3 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 3)
                }
            }
            x if x == self.keys.moveto_workspace4 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 4)
                }
            }
            x if x == self.keys.moveto_workspace5 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 5)
                }
            }
            x if x == self.keys.moveto_workspace6 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 6)
                }
            }
            x if x == self.keys.moveto_workspace7 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 7)
                }
            }
            x if x == self.keys.moveto_workspace8 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 8)
                }
            }
            x if x == self.keys.moveto_workspace9 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 9)
                }
            }
            x if x == self.keys.moveto_workspace10 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 10)
                }
            }
            x if x == self.keys.moveto_workspace11 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 11)
                }
            }
            x if x == self.keys.moveto_workspace12 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 12)
                }
            }
            x if x == self.keys.moveto_workspace13 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 13)
                }
            }
            x if x == self.keys.moveto_workspace14 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 14)
                }
            }
            x if x == self.keys.moveto_workspace15 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 15)
                }
            }
            x if x == self.keys.moveto_workspace16 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 16)
                }
            }
            x if x == self.keys.moveto_workspace17 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 17)
                }
            }
            x if x == self.keys.moveto_workspace18 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 18)
                }
            }
            x if x == self.keys.moveto_workspace19 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 19)
                }
            }
            x if x == self.keys.moveto_workspace20 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 20)
                }
            }
            x if x == self.keys.moveto_workspace21 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 21)
                }
            }
            x if x == self.keys.moveto_workspace22 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 22)
                }
            }
            x if x == self.keys.moveto_workspace23 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 23)
                }
            }
            x if x == self.keys.moveto_workspace24 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 24)
                }
            }
            x if x == self.keys.moveto_workspace25 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 25)
                }
            }
            x if x == self.keys.moveto_workspace26 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 26)
                }
            }
            x if x == self.keys.moveto_workspace27 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 27)
                }
            }
            x if x == self.keys.moveto_workspace28 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 28)
                }
            }
            x if x == self.keys.moveto_workspace29 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 29)
                }
            }
            x if x == self.keys.moveto_workspace30 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 30)
                }
            }
            x if x == self.keys.moveto_workspace31 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 31)
                }
            }
            x if x == self.keys.moveto_workspace32 => {
                if let Some(view) = view {
                    self.moveto_workspace(view, 32)
                }
            }

            _ => {
                return if let Some(view) = view {
                    let lock = view.get::<ViewWorkspace>();
                    let result = match lock.as_ref().and_then(|x| x.read().ok()) {
                        Some(space) => {
                            self.workspaces
                                .get_mut(&space.num)
                                .expect("ViewWorkspace was invalid")
                                .keyboard_key(Some(view), time, modifiers, key, state)
                        }
                        None => false,
                    };
                    result
                } else {
                    Output::with_focused_output(|output| {
                        let lock = output.get::<ActiveWorkspace>();
                        let result = match lock.as_ref().and_then(|x| x.read().ok()) {
                            Some(active) => {
                                self.workspaces
                                    .get_mut(&active.num)
                                    .expect("ActiveWorkspace was invalid")
                                    .keyboard_key(None, time, modifiers, key, state)
                            }
                            None => false,
                        };
                        result
                    })
                };
            }
        }
        true
    }

    fn pointer_button(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, button: Button,
                      state: ButtonState, origin: Point)
                      -> bool {
        if let Some(view) = view {
            let lock = view.get::<ViewWorkspace>();
            let result = match lock.as_ref().and_then(|x| x.read().ok()) {
                Some(space) => {
                    self.workspaces
                        .get_mut(&space.num())
                        .expect("ViewWorkspace was invalid")
                        .pointer_button(Some(view), time, modifiers, button, state, origin)
                }
                None => false,
            };
            result
        } else {
            Output::with_focused_output(|output| {
                let lock = output.get::<ActiveWorkspace>();
                let result = match lock.as_ref().and_then(|x| x.read().ok()) {
                    Some(active) => {
                        self.workspaces
                            .get_mut(&active.num)
                            .expect("ActiveWorkspace was invalid")
                            .pointer_button(None, time, modifiers, button, state, origin)
                    }
                    None => false,
                };
                result
            })
        }
    }

    fn pointer_scroll(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers,
                      axis: ScrollAxis::Flags, amount: [f64; 2])
                      -> bool {
        if let Some(view) = view {
            let lock = view.get::<ViewWorkspace>();
            let result = match lock.as_ref().and_then(|x| x.read().ok()) {
                Some(space) => {
                    self.workspaces
                        .get_mut(&space.num())
                        .expect("ViewWorkspace was invalid")
                        .pointer_scroll(Some(view), time, modifiers, axis, amount)
                }
                None => false,
            };
            result
        } else {
            Output::with_focused_output(|output| {
                let lock = output.get::<ActiveWorkspace>();
                let result = match lock.as_ref().and_then(|x| x.read().ok()) {
                    Some(active) => {
                        self.workspaces
                            .get_mut(&active.num)
                            .expect("ActiveWorkspace was invalid")
                            .pointer_scroll(None, time, modifiers, axis, amount)
                    }
                    None => false,
                };
                result
            })
        }
    }

    fn pointer_motion(&mut self, view: Option<&View>, time: u32, origin: Point) -> bool {
        if let Some(view) = view {
            let lock = view.get::<ViewWorkspace>();
            let result = match lock.as_ref().and_then(|x| x.read().ok()) {
                Some(space) => {
                    self.workspaces
                        .get_mut(&space.num)
                        .expect("ViewWorkspace was invalid")
                        .pointer_motion(Some(view), time, origin)
                }
                None => false,
            };
            result
        } else {
            Output::with_focused_output(|output| {
                let lock = output.get::<ActiveWorkspace>();
                let result = match lock.as_ref().and_then(|x| x.read().ok()) {
                    Some(active) => {
                        self.workspaces
                            .get_mut(&active.num)
                            .expect("ActiveWorkspace was invalid")
                            .pointer_motion(None, time, origin)
                    }
                    None => false,
                };
                result
            })
        }
    }

    fn touch(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, touch_type: TouchType,
             slot: i32, origin: Point)
             -> bool {
        if let Some(view) = view {
            let lock = view.get::<ViewWorkspace>();
            let result = match lock.as_ref().and_then(|x| x.read().ok()) {
                Some(space) => {
                    self.workspaces
                        .get_mut(&space.num)
                        .expect("ViewWorkspace was invalid")
                        .touch(Some(view), time, modifiers, touch_type, slot, origin)
                }
                None => false,
            };
            result
        } else {
            Output::with_focused_output(|output| {
                let lock = output.get::<ActiveWorkspace>();
                let result = match lock.as_ref().and_then(|x| x.read().ok()) {
                    Some(active) => {
                        self.workspaces
                            .get_mut(&active.num)
                            .expect("ActiveWorkspace was invalid")
                            .touch(None, time, modifiers, touch_type, slot, origin)
                    }
                    None => false,
                };
                result
            })
        }
    }

    fn compositor_ready(&mut self) {}
    fn compositor_terminate(&mut self) {}
}
