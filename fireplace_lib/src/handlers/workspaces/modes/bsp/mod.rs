//! i3-like **B**inary **S**plit  **P**artitioning `Mode` and corresponding
//! handlers and types
//!

use callback::{AsWrapper, IntoCallback, Wrapper};
use handlers::geometry::{UsableScreenGeometry, UsableViewGeometry};
use handlers::keyboard::KeyPattern;
use handlers::store::{Store, StoreKey};
use handlers::workspaces::modes::Mode;

use id_tree::{Node, NodeBuilder, NodeId, Tree};
use id_tree::InsertBehavior::*;
use id_tree::MoveBehavior::*;
use id_tree::RemoveBehavior::*;
use id_tree::SwapBehavior::*;

use slog;
use slog_scope;
use std::ops::Not;
use wlc::{Callback, Geometry, Key, KeyState, Modifiers, Output, Point, ResizeEdge, Size, View, WeakView};

#[cfg(feature = "conrod_ui")]
mod indicators;

#[cfg(feature = "conrod_ui")]
pub use self::indicators::{IndicatorConfig, IndicatorsHandler};

/// A tiling `Mode` that organises `View`s in a binary tree structure
/// and displays them by deviding the available space
#[cfg(not(feature = "conrod_ui"))]
pub struct BSP {
    tree: Tree<Data>,
    tiling_root: Option<WeakView>,
    size: Geometry,
    next_orientation: Orientation,
    keys: KeyPatterns,
    logger: slog::Logger,
}

/// A tiling `Mode` that organises `View`s in a binary tree structure
/// and displays them by deviding the available space
#[cfg(feature = "conrod_ui")]
pub struct BSP {
    tree: Tree<Data>,
    tiling_root: Option<WeakView>,
    size: Geometry,
    next_orientation: Orientation,
    keys: KeyPatterns,
    gui: IndicatorsHandler,
    logger: slog::Logger,
}

/// Convinient type alias for a wrapped `BSP`
pub type BSPWrap = Wrapper<BSP>;

/// Configuration for `BSP`
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BSPConfig {
    /// Which `Orientation` to split by the `Mode` shall start with
    pub starting_orientation: Orientation,
    /// Key configuration
    pub keys: KeyPatterns,
    /// Supporting Ui configuration, if desired
    #[cfg(feature = "conrod_ui")]
    pub ui: Option<Ui>,
}

/// `BSP` supporting Ui configuration
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg(feature = "conrod_ui")]
pub struct Ui {
    /// `View` indicators configuration
    pub indicator: Option<IndicatorConfig>,
}

/// `KeyPatterns` reorganising the internal tree
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyPatterns {
    /// Switch focus to the closest container on the left
    pub focus_left: Option<KeyPattern>,
    /// Switch focus to the closest container on the right
    pub focus_right: Option<KeyPattern>,
    /// Switch focus to the closest container above
    pub focus_up: Option<KeyPattern>,
    /// Switch focus to the closest container below
    pub focus_down: Option<KeyPattern>,

    /// Move the current container one tile left
    pub move_left: Option<KeyPattern>,
    /// Move the current container one tile right
    pub move_right: Option<KeyPattern>,
    /// Move the current container one tile up
    pub move_up: Option<KeyPattern>,
    /// Move the current container one tile down
    pub move_down: Option<KeyPattern>,

    /// Resize the current container by moving it's border leftwards
    pub resize_left: Option<KeyPattern>,
    /// Resize the current container by moving it's border rightwards
    pub resize_right: Option<KeyPattern>,
    /// Resize the current container by moving it's border upwards
    pub resize_up: Option<KeyPattern>,
    /// Resize the current container by moving it's border downwards
    pub resize_down: Option<KeyPattern>,

    /// Toggle the orientation used to split the current container on a new
    /// `View` between `Horizontal` and `Vertical`
    pub toggle_orientation: Option<KeyPattern>,
    /// Switch the orientation used to split the current container on a new
    /// `View` to `Horizontal`
    pub horizontal_orientation: Option<KeyPattern>,
    /// Switch the orientation used to split the current container on a new
    /// `View` to `Vertical`
    pub vertical_orientation: Option<KeyPattern>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Leaf {
    view: WeakView,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Split {
    orientation: Orientation,
    ratio: f64,
}

#[derive(Clone, Debug, PartialEq)]
enum Data {
    Split(Split),
    Leaf(Leaf),
}

/// Orientation of a container used for splitting on new `View`s
#[cfg_attr(rustfmt, rustfmt_skip)]
enum_str!(pub enum Orientation
{
    Horizontal,
    Vertical,
});

impl Default for Orientation {
    fn default() -> Orientation {
        Orientation::Vertical
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
enum_str!(enum Direction
{
    Left,
    Right,
});

impl Default for Direction {
    fn default() -> Direction {
        Direction::Right
    }
}

impl Not for Orientation {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal,
        }
    }
}

impl Not for Direction {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

impl StoreKey for NodeId {
    type Value = NodeId;
}

impl AsWrapper for BSP {
    #[cfg(not(feature = "conrod_ui"))]
    fn child(&mut self) -> Option<&mut Callback> {
        None
    }

    #[cfg(feature = "conrod_ui")]
    fn child(&mut self) -> Option<&mut Callback> {
        Some(&mut self.gui)
    }
}

impl Mode for Wrapper<BSP> {
    type Arguments = BSPConfig;

    #[cfg(not(feature = "conrod_ui"))]
    fn new(arguments: BSPConfig) -> Self {
        let mode = BSP {
            tree: Tree::new(),
            tiling_root: None,
            size: Geometry {
                origin: Point { x: 0, y: 0 },
                size: Size { w: 0, h: 0 },
            },
            next_orientation: arguments.starting_orientation,
            keys: arguments.keys,
            logger: slog_scope::logger().new(o!("instance" => "BSP")),
        };
        debug!(mode.logger, "Created");
        mode.into_callback()
    }

    #[cfg(feature = "conrod_ui")]
    fn new(arguments: BSPConfig) -> Self {
        let mode = BSP {
            tree: Tree::new(),
            tiling_root: None,
            size: Geometry {
                origin: Point { x: 0, y: 0 },
                size: Size { w: 0, h: 0 },
            },
            next_orientation: arguments.starting_orientation,
            keys: arguments.keys,
            gui: IndicatorsHandler::new(arguments
                                            .ui
                                            .unwrap_or_default()
                                            .indicator
                                            .unwrap_or(IndicatorConfig { width: 0 })),
            logger: slog_scope::logger().new(o!("instance" => "BSP")),
        };
        debug!(mode.logger, "Created");
        mode.into_callback()
    }

    fn len(&self) -> usize {
        self.number_of_children()
    }
}

impl Callback for Wrapper<BSP> {
    fn view_created(&mut self, view: &View) -> bool {
        slog_scope::scope(self.logger.clone(), || {
            let result = self.insert_view(view);
            if result {
                if let Some(child) = self.child() {
                    child.view_created(view);
                }
                self.recalculate();
            }
            result
        })
    }

    fn view_destroyed(&mut self, view: &View) {
        slog_scope::scope(self.logger.clone(), || {
            if let Some(child) = self.child() {
                child.view_destroyed(view);
            }
            self.remove_view(view);
            self.recalculate();
        })
    }

    fn view_focus(&mut self, view: &View, focus: bool) {
        if focus {
            debug!(self.logger, "Focus shifts tiling root to {:?}", view);
            self.tiling_root = Some(view.weak_reference());
        } else {
            self.tiling_root = None;
        }
        if let Some(child) = self.child() {
            child.view_focus(view, focus);
        }
    }

    fn view_move_to_output(&mut self, view: &View, _from: &Output, _to: &Output) {
        if let Some(child) = self.child() {
            child.view_destroyed(view);
            child.view_created(view);
        }
    }

    fn output_resolution(&mut self, output: &Output, from: Size, to: Size) {
        slog_scope::scope(self.logger.clone(), || {
            let lock = output.get::<UsableScreenGeometry>();
            self.size = match lock.as_ref().and_then(|x| x.read().ok()) {
                Some(size) => *size,
                None => {
                    Geometry {
                        origin: Point { x: 0, y: 0 },
                        size: to,
                    }
                }
            };
            if let Some(child) = self.child() {
                child.output_resolution(output, from, to);
            }
            self.recalculate();
        })
    }

    #[cfg_attr(feature = "cargo-clippy", allow(cyclomatic_complexity))]
    fn keyboard_key(&mut self, view: Option<&View>, _time: u32, modifiers: Modifiers, key: Key,
                    state: KeyState)
                    -> bool {
        slog_scope::scope(self.logger.clone(), || {
            match Some(KeyPattern::new(state, modifiers.mods, key)) {
                x if x == self.keys.focus_left => {
                    if let Some(view) = view {
                        self.move_focus(view, Orientation::Horizontal, Direction::Left);
                    }
                }
                x if x == self.keys.focus_right => {
                    if let Some(view) = view {
                        self.move_focus(view, Orientation::Horizontal, Direction::Right)
                    }
                }
                x if x == self.keys.focus_up => {
                    if let Some(view) = view {
                        self.move_focus(view, Orientation::Vertical, Direction::Left)
                    }
                }
                x if x == self.keys.focus_down => {
                    if let Some(view) = view {
                        self.move_focus(view, Orientation::Vertical, Direction::Right)
                    }
                }
                x if x == self.keys.move_left => {
                    if let Some(view) = view {
                        self.move_view(view, Orientation::Horizontal, Direction::Left)
                    }
                }
                x if x == self.keys.move_right => {
                    if let Some(view) = view {
                        self.move_view(view, Orientation::Horizontal, Direction::Right)
                    }
                }
                x if x == self.keys.move_up => {
                    if let Some(view) = view {
                        self.move_view(view, Orientation::Vertical, Direction::Left)
                    }
                }
                x if x == self.keys.move_down => {
                    if let Some(view) = view {
                        self.move_view(view, Orientation::Vertical, Direction::Right)
                    }
                }
                x if x == self.keys.resize_left => {
                    if let Some(view) = view {
                        self.resize(view, Orientation::Horizontal, Direction::Left, 0.02)
                    }
                }
                x if x == self.keys.resize_right => {
                    if let Some(view) = view {
                        self.resize(view, Orientation::Horizontal, Direction::Right, 0.02)
                    }
                }
                x if x == self.keys.resize_up => {
                    if let Some(view) = view {
                        self.resize(view, Orientation::Vertical, Direction::Left, 0.02)
                    }
                }
                x if x == self.keys.resize_down => {
                    if let Some(view) = view {
                        self.resize(view, Orientation::Vertical, Direction::Right, 0.02)
                    }
                }
                x if x == self.keys.toggle_orientation => self.next_orientation = !self.next_orientation,
                x if x == self.keys.horizontal_orientation => self.next_orientation = Orientation::Horizontal,
                x if x == self.keys.vertical_orientation => self.next_orientation = Orientation::Vertical,
                _ => {
                    return false;
                }

            };

            self.recalculate();
            true
        })
    }
}

include!("node.rs");
include!("ops.rs");
