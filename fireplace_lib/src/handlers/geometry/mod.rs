//! Handlers related to `Output` and `View` geometry
//!
//! Contains a public interface to restrict and modify the actual used space
//! for `View` and general and individually. `Mode`s setting their geometry are
//! expected to follow the set values and other handlers may use this interface
//! to change these values.
//!

use handlers::store::{Store, StoreKey};
use wlc::{Callback, Geometry, Output, Point, Size, View};

mod gaps;
pub use self::gaps::*;

/// Key for receiving the usable screen geometry, that is
/// the part of the screen not used by UI elements, from an `Output`s
/// [`Store`](../trait.Store.html).
///
/// Should be modified when creating UI elements at the borders of the
/// screen, so no `View`s are placed above them.
///
/// Take note other UI elements may as well and always substract or add
/// value (when destroying), but dont set to absolute values.
pub struct UsableScreenGeometry;
impl StoreKey for UsableScreenGeometry {
    type Value = Geometry;
}


/// Key for receiving the usable view geometry, that is
/// the part of and area mapped to a `View` not used
/// by UI elements around it, from a `View`s
/// [`Store`](../trait.Store.html).
///
/// Should be modified when creating UI elements at the borders of a
/// `View`, so the actual `View` gets drawn inside of it.
///
/// Take note other UI elements may as well and always substract or add
/// value (when it gets destroyed), but dont set to absolute values.
pub struct UsableViewGeometry;
impl StoreKey for UsableViewGeometry {
    type Value = ViewScissor;
}

/// Value to express that borders of a View shall be shortened
pub struct ViewScissor {
    /// Amount of points the upper border should be moved inside the geometry
    /// used
    pub up: usize,
    /// Amount of points the right border should be moved inside the geometry
    /// used
    pub right: usize,
    /// Amount of points the lower border should be moved inside the geometry
    /// used
    pub down: usize,
    /// Amount of points the left border should be moved inside the geometry
    /// used
    pub left: usize,
}

/// Key for receiving the initial view geometry, that is the size and origin
/// the `View` has requested at launch, from a `View`s
/// [`Sotre`](../trait.Store.html).
///
/// Should be considered read-only and not modified.
///
/// Most useful for modes that might have lost that information otherwise.
///
pub struct InitialViewGeometry;
impl StoreKey for InitialViewGeometry {
    type Value = Geometry;
}

/// Handler that initializes default `UsableScreenGeometry` and
/// `UsableViewGeometry` values for created `View`s and `Output`s
///
/// ## Dependencies
///
/// - [`StoreHandler`](../struct.StoreHandler.html)
///
#[derive(Default)]
pub struct GeometryHandler;
impl GeometryHandler {
    /// Initialize a new `GeometryHandler`
    pub fn new() -> GeometryHandler {
        GeometryHandler
    }
}

impl Callback for GeometryHandler {
    fn output_created(&mut self, output: &Output) -> bool {
        output.insert::<UsableScreenGeometry>(Geometry {
            origin: Point { x: 0, y: 0 },
            size: output.resolution(),
        });
        true
    }

    fn output_resolution(&mut self, output: &Output, _from: Size, to: Size) {
        output.insert::<UsableScreenGeometry>(Geometry {
            origin: Point { x: 0, y: 0 },
            size: to,
        });
    }

    fn view_created(&mut self, view: &View) -> bool {
        view.insert::<InitialViewGeometry>(view.geometry());
        view.insert::<UsableViewGeometry>(ViewScissor {
            up: 0,
            right: 0,
            down: 0,
            left: 0,
        });
        true
    }
}
