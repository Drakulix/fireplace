use std::cell::RefCell;
use std::sync::Mutex;

use smithay::{
    reexports::{
        wayland_protocols::xdg_shell::server::xdg_toplevel,
        wayland_server::protocol::wl_surface,
    },
    utils::{Logical, Point, Rectangle, Size},
    wayland::{
        compositor::{
            with_states, with_surface_tree_downward, SubsurfaceCachedState,
            SurfaceData as WlSurfaceData, TraversalAction,
        },
        shell::xdg::{
            PopupSurface, SurfaceCachedState, ToplevelSurface, XdgPopupSurfaceRoleAttributes,
        },
    },
};

use super::SurfaceData;
#[cfg(feature = "xwayland")]
use crate::xwayland::X11Surface;

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Xdg(ToplevelSurface),
    #[cfg(feature = "xwayland")]
    X11(X11Surface),
}

impl Kind {
    pub fn alive(&self) -> bool {
        match *self {
            Kind::Xdg(ref t) => t.alive(),
            #[cfg(feature = "xwayland")]
            Kind::X11(ref t) => t.alive(),
        }
    }

    pub fn get_surface(&self) -> Option<&wl_surface::WlSurface> {
        match *self {
            Kind::Xdg(ref t) => t.get_surface(),
            #[cfg(feature = "xwayland")]
            Kind::X11(ref t) => t.get_surface(),
        }
    }

    /// Activate/Deactivate this window
    pub fn set_activated(&self, active: bool) {
        #[allow(irrefutable_let_patterns)]
        if let Kind::Xdg(ref t) = self {
            let changed = t.with_pending_state(|state| {
                if active {
                    state.states.set(xdg_toplevel::State::Activated)
                } else {
                    state.states.unset(xdg_toplevel::State::Activated)
                }
            });
            if let Ok(true) = changed {
                t.send_configure();
            }
        }
    }

    pub fn send_close(&self) {
        match *self {
            Kind::Xdg(ref t) => t.send_close(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PopupKind {
    Xdg(PopupSurface),
}

impl PopupKind {
    pub fn alive(&self) -> bool {
        match *self {
            PopupKind::Xdg(ref t) => t.alive(),
        }
    }

    pub fn get_surface(&self) -> Option<&wl_surface::WlSurface> {
        match *self {
            PopupKind::Xdg(ref t) => t.get_surface(),
        }
    }

    pub fn parent(&self) -> Option<wl_surface::WlSurface> {
        let wl_surface = match self.get_surface() {
            Some(s) => s,
            None => return None,
        };
        with_states(wl_surface, |states| {
            states
                .data_map
                .get::<Mutex<XdgPopupSurfaceRoleAttributes>>()
                .unwrap()
                .lock()
                .unwrap()
                .parent
                .clone()
        })
        .ok()
        .flatten()
    }

    pub fn location(&self) -> Point<i32, Logical> {
        let wl_surface = match self.get_surface() {
            Some(s) => s,
            None => return (0, 0).into(),
        };
        with_states(wl_surface, |states| {
            states
                .data_map
                .get::<Mutex<XdgPopupSurfaceRoleAttributes>>()
                .unwrap()
                .lock()
                .unwrap()
                .current
                .geometry
        })
        .unwrap_or_default()
        .loc
    }
}

#[derive(Debug)]
pub struct Window {
    location: Option<Point<i32, Logical>>,
    size: Size<i32, Logical>,
    pub toplevel: Kind,
}

impl Window {
    pub fn new(
        location: Option<Point<i32, Logical>>,
        size: Option<Size<i32, Logical>>,
        toplevel: Kind,
    ) -> Window {
        if let Some(size) = size {
            match &toplevel {
                Kind::Xdg(xdg) => {
                    let ret = xdg.with_pending_state(|state| {
                        state.size = Some(size);
                    });
                    if ret.is_ok() {
                        xdg.send_configure();
                    }
                }
            }
        }

        let mut window = Window {
            location,
            size: size.unwrap_or((0, 0).into()),
            toplevel,
        };
        window
    }

    /// Finds the topmost surface under this point if any and returns it together with the location of this
    /// surface.
    pub fn matching(
        &self,
        point: Point<f64, Logical>,
    ) -> Option<(wl_surface::WlSurface, Point<i32, Logical>)> {
        if !self.bbox().to_f64().contains(point) {
            return None;
        }
        // need to check more carefully
        let found = RefCell::new(None);
        if let Some(wl_surface) = self.toplevel.get_surface() {
            with_surface_tree_downward(
                wl_surface,
                self.location.unwrap(), // if we do not have a location, we have already returned early
                |wl_surface, states, location| {
                    let mut location = *location;
                    let data = states.data_map.get::<RefCell<SurfaceData>>();

                    if states.role == Some("subsurface") {
                        let current = states.cached_state.current::<SubsurfaceCachedState>();
                        location += current.location;
                    }

                    let contains_the_point = data
                        .map(|data| {
                            data.borrow().contains_point(
                                &*states.cached_state.current(),
                                point - location.to_f64(),
                            )
                        })
                        .unwrap_or(false);
                    if contains_the_point {
                        *found.borrow_mut() = Some((wl_surface.clone(), location));
                    }

                    TraversalAction::DoChildren(location)
                },
                |_, _, _| {},
                |_, _, _| {
                    // only continue if the point is not found
                    found.borrow().is_none()
                },
            );
        }
        found.into_inner()
    }

    pub fn contains_surface(&self, surface: &wl_surface::WlSurface) -> bool {
        if let Some(wl_surface) = self.toplevel.get_surface() {
            let found = RefCell::new(false);
            with_surface_tree_downward(
                wl_surface,
                (),
                |wl_surface, _, _| {
                    if wl_surface == surface {
                        *found.borrow_mut() = true;
                        TraversalAction::Break
                    } else {
                        TraversalAction::DoChildren(())
                    }
                },
                |_, _, _| {},
                |_, _, _| !*found.borrow(),
            );
            found.into_inner()
        } else {
            false
        }
    }

    pub fn bbox(&self) -> Rectangle<i32, Logical> {
        let location = self.location.unwrap_or((0, 0).into());
        let mut bounding_box = Rectangle::from_loc_and_size(location, (0, 0));
        if let Some(wl_surface) = self.toplevel.get_surface() {
            with_surface_tree_downward(
                wl_surface,
                location,
                |_, states: &WlSurfaceData, loc: &Point<i32, Logical>| {
                    let mut loc = *loc;
                    let data = states.data_map.get::<RefCell<SurfaceData>>();

                    if let Some(size) = data.and_then(|d| d.borrow().size()) {
                        if states.role == Some("subsurface") {
                            let current = states.cached_state.current::<SubsurfaceCachedState>();
                            loc += current.location;
                        }

                        // Update the bounding box.
                        bounding_box = bounding_box.merge(Rectangle::from_loc_and_size(loc, size));

                        TraversalAction::DoChildren(loc)
                    } else {
                        // If the parent surface is unmapped, then the child surfaces are hidden as
                        // well, no need to consider them here.
                        TraversalAction::SkipChildren
                    }
                },
                |_, _, _| {},
                |_, _, _| true,
            );
        }
        bounding_box
    }

    /// Returns the geometry of this window.
    pub fn geometry(&self) -> Rectangle<i32, Logical> {
        // It's the set geometry with the full bounding box as the fallback.
        with_states(self.toplevel.get_surface().unwrap(), |states| {
            states.cached_state.current::<SurfaceCachedState>().geometry
        })
        .unwrap()
        .unwrap_or(self.bbox())
    }

    pub fn location(&self) -> Option<Point<i32, Logical>> {
        self.location
    }

    /// Sets the location
    pub fn set_location(&mut self, location: Point<i32, Logical>) {
        self.location = Some(location);
    }
}
