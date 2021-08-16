use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        Mutex,
        atomic::Ordering,
    },
};

use smithay::{
    utils::{Point, Rectangle, Logical, Size},
    wayland::{
        compositor::with_states,
        seat::{Seat, GrabStartData, PointerGrab, PointerInnerHandle, AxisFrame},
        shell::xdg::{ToplevelConfigure, SurfaceCachedState, XdgToplevelSurfaceRoleAttributes},
        Serial,
    },
    reexports::{
        wayland_server::protocol::{
            wl_surface,
            wl_shell_surface,
            wl_pointer::ButtonState,
        },
        wayland_protocols::{
            xdg_shell::server::xdg_toplevel,
        }
    },
};

use crate::shell::{
    window::{Window, Kind},
    SurfaceData,
};
use super::{Layout, ID_COUNTER};

bitflags::bitflags! {
    struct ResizeEdge: u32 {
        const NONE = 0;
        const TOP = 1;
        const BOTTOM = 2;
        const LEFT = 4;
        const TOP_LEFT = 5;
        const BOTTOM_LEFT = 6;
        const RIGHT = 8;
        const TOP_RIGHT = 9;
        const BOTTOM_RIGHT = 10;
    }
}

impl From<wl_shell_surface::Resize> for ResizeEdge {
    #[inline]
    fn from(x: wl_shell_surface::Resize) -> Self {
        Self::from_bits(x.bits()).unwrap()
    }
}

impl From<ResizeEdge> for wl_shell_surface::Resize {
    #[inline]
    fn from(x: ResizeEdge) -> Self {
        Self::from_bits(x.bits()).unwrap()
    }
}

impl From<xdg_toplevel::ResizeEdge> for ResizeEdge {
    #[inline]
    fn from(x: xdg_toplevel::ResizeEdge) -> Self {
        Self::from_bits(x.to_raw()).unwrap()
    }
}

impl From<ResizeEdge> for xdg_toplevel::ResizeEdge {
    #[inline]
    fn from(x: ResizeEdge) -> Self {
        Self::from_raw(x.bits()).unwrap()
    }
}

/// Information about the resize operation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ResizeData {
    /// The edges the surface is being resized with.
    edges: ResizeEdge,
    /// The initial window location.
    initial_window_location: Point<i32, Logical>,
    /// The initial window size (geometry width and height).
    initial_window_size: Size<i32, Logical>,
}

/// State of the resize operation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ResizeState {
    /// The surface is not being resized.
    NotResizing,
    /// The surface is currently being resized.
    Resizing(ResizeData),
    /// The resize has finished, and the surface needs to ack the final configure.
    WaitingForFinalAck(ResizeData, Serial),
    /// The resize has finished, and the surface needs to commit its final state.
    WaitingForCommit(ResizeData),
}

impl Default for ResizeState {
    fn default() -> Self {
        ResizeState::NotResizing
    }
}
struct ResizeSurfaceGrab {
    start_data: GrabStartData,
    toplevel: Kind,
    edges: ResizeEdge,
    initial_window_size: Size<i32, Logical>,
    last_window_size: Size<i32, Logical>,
}

impl PointerGrab for ResizeSurfaceGrab {
    fn motion(
        &mut self,
        handle: &mut PointerInnerHandle<'_>,
        location: Point<f64, Logical>,
        _focus: Option<(wl_surface::WlSurface, Point<i32, Logical>)>,
        serial: Serial,
        time: u32,
    ) {
        // It is impossible to get `min_size` and `max_size` of dead toplevel, so we return early.
        if !self.toplevel.alive() | self.toplevel.get_surface().is_none() {
            handle.unset_grab(serial, time);
            return;
        }

        let (mut dx, mut dy) = (location - self.start_data.location).into();

        let mut new_window_width = self.initial_window_size.w;
        let mut new_window_height = self.initial_window_size.h;

        let left_right = ResizeEdge::LEFT | ResizeEdge::RIGHT;
        let top_bottom = ResizeEdge::TOP | ResizeEdge::BOTTOM;

        if self.edges.intersects(left_right) {
            if self.edges.intersects(ResizeEdge::LEFT) {
                dx = -dx;
            }

            new_window_width = (self.initial_window_size.w as f64 + dx) as i32;
        }

        if self.edges.intersects(top_bottom) {
            if self.edges.intersects(ResizeEdge::TOP) {
                dy = -dy;
            }

            new_window_height = (self.initial_window_size.h as f64 + dy) as i32;
        }

        let (min_size, max_size) = with_states(self.toplevel.get_surface().unwrap(), |states| {
            let data = states.cached_state.current::<SurfaceCachedState>();
            (data.min_size, data.max_size)
        })
        .unwrap();

        let min_width = min_size.w.max(1);
        let min_height = min_size.h.max(1);
        let max_width = if max_size.w == 0 {
            i32::max_value()
        } else {
            max_size.w
        };
        let max_height = if max_size.h == 0 {
            i32::max_value()
        } else {
            max_size.h
        };

        new_window_width = new_window_width.max(min_width).min(max_width);
        new_window_height = new_window_height.max(min_height).min(max_height);

        self.last_window_size = (new_window_width, new_window_height).into();

        match &self.toplevel {
            Kind::Xdg(xdg) => {
                let ret = xdg.with_pending_state(|state| {
                    state.states.set(xdg_toplevel::State::Resizing);
                    state.size = Some(self.last_window_size);
                });
                if ret.is_ok() {
                    xdg.send_configure();
                }
            }
        }
    }

    fn button(
        &mut self,
        handle: &mut PointerInnerHandle<'_>,
        button: u32,
        state: ButtonState,
        serial: Serial,
        time: u32,
    ) {
        handle.button(button, state, serial, time);
        if handle.current_pressed().is_empty() {
            // No more buttons are pressed, release the grab.
            handle.unset_grab(serial, time);

            // If toplevel is dead, we can't resize it, so we return early.
            if !self.toplevel.alive() | self.toplevel.get_surface().is_none() {
                return;
            }

            #[allow(irrefutable_let_patterns)]
            if let Kind::Xdg(xdg) = &self.toplevel {
                let ret = xdg.with_pending_state(|state| {
                    state.states.unset(xdg_toplevel::State::Resizing);
                    state.size = Some(self.last_window_size);
                });
                if ret.is_ok() {
                    xdg.send_configure();
                }

                with_states(self.toplevel.get_surface().unwrap(), |states| {
                    let mut data = states
                        .data_map
                        .get::<RefCell<SurfaceData>>()
                        .unwrap()
                        .borrow_mut();

                    let resize_state_cell = data.userdata().get::<RefCell<ResizeState>>().unwrap();
                    let mut resize_state = resize_state_cell.borrow_mut();

                    if let ResizeState::Resizing(resize_data) = *resize_state {
                        *resize_state = ResizeState::WaitingForFinalAck(resize_data, serial);
                    } else {
                        panic!("invalid resize state: {:?}", resize_state);
                    }
                })
                .unwrap();
            }
        }
    }

    fn axis(&mut self, handle: &mut PointerInnerHandle<'_>, details: AxisFrame) {
        handle.axis(details)
    }

    fn start_data(&self) -> &GrabStartData {
        &self.start_data
    }
}

struct MoveSurfaceGrab {
    start_data: GrabStartData,
    window: Rc<RefCell<Window>>,
    toplevel: Kind,
    initial_window_location: Point<i32, Logical>,
}

impl PointerGrab for MoveSurfaceGrab {
    fn motion(
        &mut self,
        _handle: &mut PointerInnerHandle<'_>,
        location: Point<f64, Logical>,
        _focus: Option<(wl_surface::WlSurface, Point<i32, Logical>)>,
        _serial: Serial,
        _time: u32,
    ) {
        let delta = location - self.start_data.location;
        let new_location = self.initial_window_location.to_f64() + delta;

        self.window.borrow_mut().set_location(
            (new_location.x as i32, new_location.y as i32).into(),
        );
    }

    fn button(
        &mut self,
        handle: &mut PointerInnerHandle<'_>,
        button: u32,
        state: ButtonState,
        serial: Serial,
        time: u32,
    ) {
        handle.button(button, state, serial, time);
        if handle.current_pressed().is_empty() {
            // No more buttons are pressed, release the grab.
            handle.unset_grab(serial, time);
        }
    }

    fn axis(&mut self, handle: &mut PointerInnerHandle<'_>, details: AxisFrame) {
        handle.axis(details)
    }

    fn start_data(&self) -> &GrabStartData {
        &self.start_data
    }
}

pub struct Floating {
    id: usize,
    size: Size<i32, Logical>,
    windows: Vec<Rc<RefCell<Window>>>,
}

impl PartialEq for Floating {
    fn eq(&self, other: &Floating) -> bool {
        self.id == other.id
    }
}

impl Floating {
    pub fn new<S: Into<Size<i32, Logical>>>(size: S) -> Floating {
        Floating {
            id: ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            size: size.into(),
            windows: Vec::new(),
        }
    }

    /// Returns the location of the toplevel, if it exists.
    pub fn location(&self, surface: &Kind) -> Option<Point<i32, Logical>> {
        self.windows
            .iter()
            .find(|w| &w.borrow().toplevel == surface)
            .and_then(|w| w.borrow().location())
    }

    pub fn window_for_toplevel(&self, surface: &Kind) -> Option<Rc<RefCell<Window>>> {
        self.windows.iter().find(|w| &w.borrow().toplevel == surface).cloned()
    }
}

impl Layout for Floating {
    fn id(&self) -> usize {
        self.id
    }

    fn new_toplevel(&mut self, surface: Kind) {
        let mut window = Window::new(
            None,
            None,
            surface,
        );
        // might happen if an already configured window is moved here
        if window.bbox().size != (0, 0).into() {
            let geometry = window.geometry();
            // center the window for now
            let location = (
                self.size.w / 2 - geometry.size.w / 2,
                self.size.h / 2 - geometry.size.h / 2,
            ).into();
            window.set_location(location);
        }
        self.windows.insert(0, Rc::new(RefCell::new(window)));
    }

    fn remove_toplevel(&mut self, surface: Kind) {
        self.windows.retain(|x| x.borrow().toplevel != surface);
    }

    fn move_request(&mut self, surface: Kind, seat: &Seat, serial: Serial, start_data: GrabStartData) {
        let window = match self.window_for_toplevel(&surface) {
            Some(w) => w,
            None => return,
        };
        let pointer = seat.get_pointer().unwrap();
        let mut initial_window_location = match window.borrow().location() {
            Some(p) => p,
            None => return,
        };

        // If surface is maximized then unmaximize it
        #[allow(irrefutable_let_patterns)]
        if let Kind::Xdg(xdg_surface) = surface {
            if let Some(current_state) = xdg_surface.current_state() {
                if current_state.states.contains(xdg_toplevel::State::Maximized) {
                    let fs_changed = xdg_surface.with_pending_state(|state| {
                        state.states.unset(xdg_toplevel::State::Maximized);
                        state.size = None;
                    });

                    if fs_changed.is_ok() {
                        xdg_surface.send_configure();

                        // NOTE: In real compositor mouse location should be mapped to a new window size
                        // For example, you could:
                        // 1) transform mouse pointer position from compositor space to window space (location relative)
                        // 2) divide the x coordinate by width of the window to get the percentage
                        //   - 0.0 would be on the far left of the window
                        //   - 0.5 would be in middle of the window
                        //   - 1.0 would be on the far right of the window
                        // 3) multiply the percentage by new window width
                        // 4) by doing that, drag will look a lot more natural
                        //
                        // but for anvil needs setting location to pointer location is fine
                        let pos = pointer.current_location();
                        initial_window_location = (pos.x as i32, pos.y as i32).into();
                    }
                }
            }

            let grab = MoveSurfaceGrab {
                start_data,
                toplevel: Kind::Xdg(xdg_surface),
                window,
                initial_window_location,
            };

            pointer.set_grab(grab, serial);
        }
    }

    fn resize_request(&mut self, surface: Kind, seat: &Seat, serial: Serial, start_data: GrabStartData, edges: xdg_toplevel::ResizeEdge) {
        let window = match self.window_for_toplevel(&surface) {
            Some(w) => w,
            None => return,
        };
        let mut initial_window_location = match window.borrow().location() {
            Some(p) => p,
            None => return,
        };
        let geometry = window.borrow().geometry();
        let initial_window_size = geometry.size;

        let resize_state = ResizeState::Resizing(ResizeData {
            edges: edges.into(),
            initial_window_location,
            initial_window_size,
        });

        with_states(surface.get_surface().unwrap(), move |states| {
            let mut data = states
                .data_map
                .get::<RefCell<SurfaceData>>()
                .unwrap()
                .borrow_mut();
            
            data.userdata().insert_if_missing(|| RefCell::new(ResizeState::NotResizing));
            let resize_state_cell = data.userdata().get::<RefCell<ResizeState>>().unwrap();
            *resize_state_cell.borrow_mut() = resize_state;
        })
        .unwrap();

        let grab = ResizeSurfaceGrab {
            start_data,
            toplevel: surface,
            edges: edges.into(),
            initial_window_size,
            last_window_size: initial_window_size,
        };
        
        // TODO: Touch move
        let pointer = seat.get_pointer().unwrap();
        pointer.set_grab(grab, serial);   
    }
    
    fn ack_configure(&mut self, surface: wl_surface::WlSurface, configure: ToplevelConfigure) {
        let waiting_for_serial = with_states(&surface, |states| {
            if let Some(data) = states.data_map.get::<RefCell<SurfaceData>>() {
                if let Some(resize_state_cell) = data.borrow().userdata().get::<RefCell<ResizeState>>() {
                    if let ResizeState::WaitingForFinalAck(_, serial) = *resize_state_cell.borrow() {
                        return Some(serial);
                    }
                }
            }

            None
        })
        .unwrap();

        if let Some(serial) = waiting_for_serial {
            // When the resize grab is released the surface
            // resize state will be set to WaitingForFinalAck
            // and the client will receive a configure request
            // without the resize state to inform the client
            // resizing has finished. Here we will wait for
            // the client to acknowledge the end of the
            // resizing. To check if the surface was resizing
            // before sending the configure we need to use
            // the current state as the received acknowledge
            // will no longer have the resize state set
            let is_resizing = with_states(&surface, |states| {
                states
                    .data_map
                    .get::<Mutex<XdgToplevelSurfaceRoleAttributes>>()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .current
                    .states
                    .contains(xdg_toplevel::State::Resizing)
            })
            .unwrap();

            if configure.serial >= serial && is_resizing {
                with_states(&surface, |states| {
                    let mut data = states
                        .data_map
                        .get::<RefCell<SurfaceData>>()
                        .unwrap()
                        .borrow_mut();

                    let resize_state_cell = data.userdata().get::<RefCell<ResizeState>>().unwrap();
                    let mut resize_state = resize_state_cell.borrow_mut();
                    
                    if let ResizeState::WaitingForFinalAck(resize_data, _) = *resize_state {
                        *resize_state = ResizeState::WaitingForCommit(resize_data);
                    } else {
                        unreachable!()
                    }
                })
                .unwrap();
            }
        }
    }
    
    fn commit(&mut self, surface: Kind) {
        let window = match self.window_for_toplevel(&surface) {
            Some(w) => w,
            None => return,
        };

        // set initial position
        {
            let mut window = window.borrow_mut();
            if window.location().is_none() && window.bbox().size != (0, 0).into() {
                let geometry = window.geometry();
                // center the window for now
                let location = (
                    self.size.w / 2 - geometry.size.w / 2,
                    self.size.h / 2 - geometry.size.h / 2,
                ).into();
                window.set_location(location);
            }
            window.self_update();
        }

        let surface = surface.get_surface().unwrap();
        let new_location = with_states(surface, |states| {
            let mut data = states
                .data_map
                .get::<RefCell<SurfaceData>>()
                .unwrap()
                .borrow_mut();

            let mut new_location = None;

            data.userdata().insert_if_missing(|| RefCell::new(ResizeState::NotResizing));
            let resize_state_cell = data.userdata().get::<RefCell<ResizeState>>().unwrap();
            let mut resize_state = resize_state_cell.borrow_mut();
            // If the window is being resized by top or left, its location must be adjusted
            // accordingly.
            match *resize_state {
                ResizeState::Resizing(resize_data)
                | ResizeState::WaitingForFinalAck(resize_data, _)
                | ResizeState::WaitingForCommit(resize_data) => {
                    let ResizeData {
                        edges,
                        initial_window_location,
                        initial_window_size,
                    } = resize_data;

                    if edges.intersects(ResizeEdge::TOP_LEFT) {
                        let mut location = window.borrow().location().unwrap();
                        let geometry = window.borrow().geometry();

                        if edges.intersects(ResizeEdge::LEFT) {
                            location.x =
                                initial_window_location.x + (initial_window_size.w - geometry.size.w);
                        }
                        if edges.intersects(ResizeEdge::TOP) {
                            location.y =
                                initial_window_location.y + (initial_window_size.h - geometry.size.h);
                        }

                        new_location = Some(location);
                    }
                }
                ResizeState::NotResizing => (),
            }

            // Finish resizing.
            if let ResizeState::WaitingForCommit(_) = *resize_state {
                *resize_state = ResizeState::NotResizing;
            }

            new_location
        })
        .unwrap();

        if let Some(location) = new_location {
            window.borrow_mut().set_location(location);
        }
    }

    fn fullscreen_request(&mut self, surface: Kind, state: bool) {
        // do not allow fullscreening
        #[allow(irrefutable_let_patterns)]
        if let Kind::Xdg(xdg_surface) = surface {
            if !state {
                let ret = xdg_surface.with_pending_state(|state| {
                    state.states.unset(xdg_toplevel::State::Fullscreen);
                    state.size = None;
                    state.fullscreen_output = None;
                });
                if ret.is_ok() {
                    xdg_surface.send_configure();
                }
            }
        }
    }

    fn maximize_request(&mut self, surface: Kind, state: bool) {
        if state {
            let window = match self.window_for_toplevel(&surface) {
                Some(w) => w,
                None => return,
            }; 
            window.borrow_mut().set_location((0, 0).into());
            #[allow(irrefutable_let_patterns)]
            if let Kind::Xdg(xdg_surface) = surface {
                let ret = xdg_surface.with_pending_state(|state| {
                    state.states.set(xdg_toplevel::State::Maximized);
                    state.size = Some(window.borrow().geometry().size);
                });
                if ret.is_ok() {
                    xdg_surface.send_configure();
                }
            }
        } else {
            #[allow(irrefutable_let_patterns)]
            if let Kind::Xdg(xdg_surface) = surface {
                let ret = xdg_surface.with_pending_state(|state| {
                    state.states.unset(xdg_toplevel::State::Maximized);
                    state.size = None;
                });
                if ret.is_ok() {
                    xdg_surface.send_configure();
                }
            }
        }
    }

    fn minimize_request(&mut self, surface: Kind) {
        // done
    }

    //TODO: fn window_options(&mut self, surface: Kind) -> Vec<String>;

    fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }
    
    fn rearrange(&mut self, size: &Size<i32, Logical>) {
        // todo update windows out of new size
        self.size = *size;
    }
    
    fn windows<'a>(&'a self) -> Box<dyn Iterator<Item=Kind> + 'a> {
        Box::new(self.windows.iter().map(|w| w.borrow().toplevel.clone()))
    }
    fn windows_from_bottom_to_top<'a>(&'a self) -> Box<dyn Iterator<Item=(Kind, Point<i32, Logical>, Rectangle<i32, Logical>)> + 'a> {
        Box::new(self.windows.iter().rev().flat_map(|w| {
            let window = w.borrow();
            window.location().map(|location|
                (window.toplevel.clone(), location, window.bbox())
            )
        }))
    }
    
    fn on_focus(&mut self, surface: &wl_surface::WlSurface) {
        if let Some(idx) = self.windows.iter().enumerate().find(|(_, w)| {
            w.borrow().contains_surface(surface)
        }).map(|(i, _)| i) {
            let window = self.windows.remove(idx);

            for w in self.windows.iter() {
                w.borrow_mut().toplevel.set_activated(false);
            }

            window.borrow_mut().toplevel.set_activated(true);
            self.windows.insert(0, window);
        }
    }
    
    fn focused_window(&self) -> Option<Kind> {
        self.windows.iter().map(|w| w.borrow().toplevel.clone()).next()
    }
    
    fn surface_under(&mut self, point: Point<f64, Logical>) -> Option<(wl_surface::WlSurface, Point<i32, Logical>)> {
        self.windows.iter().find_map(|w| w.borrow().matching(point))
    }
}