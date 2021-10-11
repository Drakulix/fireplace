use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use smithay::{
    backend::renderer::buffer_dimensions,
    reexports::wayland_server::{
        protocol::{wl_buffer, wl_surface},
        Display, UserDataMap,
    },
    utils::{Logical, Physical, Point, Rectangle, Size},
    wayland::{
        compositor::{
            compositor_init, is_sync_subsurface, with_states, with_surface_tree_upward,
            BufferAssignment, SurfaceAttributes, TraversalAction,
        },
        seat::Seat,
        shell::{
            wlr_layer::{LayerShellRequest, LayerSurfaceAttributes},
            xdg::{
                xdg_shell_init, Configure, ShellState as XdgShellState,
                XdgPopupSurfaceRoleAttributes, XdgRequest, XdgToplevelSurfaceRoleAttributes,
            },
        },
        Serial,
    },
};

//pub mod layer;
pub mod layout;
pub mod output;
pub mod window;
pub mod workspace;

use self::{
    layout::Layout,
    window::{Kind as SurfaceKind, PopupKind},
    workspace::Workspaces,
};
use crate::{
    backend::render::BufferTextures,
    state::Fireplace,
};

#[derive(Clone)]
pub struct ShellHandles {
    pub xdg_state: Arc<Mutex<XdgShellState>>,
    pub workspaces: Rc<RefCell<Workspaces>>,
    pub popups: Rc<RefCell<Vec<PopupKind>>>,
}

pub fn init_shell(display: Rc<RefCell<Display>>) -> ShellHandles {
    // Create the compositor
    compositor_init(
        &mut *display.borrow_mut(),
        move |surface, mut ddata| {
            let state = ddata.get::<Fireplace>().unwrap();
            let mut workspaces = state.workspaces.borrow_mut();
            let mut popups = state.popups.borrow_mut();
            surface_commit(&surface, &mut *workspaces, &mut *popups)
        },
        None,
    );

    let popups = Rc::new(RefCell::new(Vec::new()));
    let workspaces = Rc::new(RefCell::new(Workspaces::new(display.clone())));

    // init the xdg_shell
    let (xdg_shell_state, _, _) = xdg_shell_init(
        &mut *display.borrow_mut(),
        move |shell_event, mut ddata| {
            let state = ddata.get::<Fireplace>().unwrap();
            let mut workspaces = state.workspaces.borrow_mut();
            let mut popups = state.popups.borrow_mut();
            match shell_event {
                XdgRequest::NewToplevel { surface } => {
                    let seat = state.last_active_seat();
                    let space = workspaces.space_by_seat(&seat).unwrap();
                    space.new_toplevel(SurfaceKind::Xdg(surface));
                }
                XdgRequest::NewPopup { surface, .. /*TODO*/ } => {
                    popups.push(PopupKind::Xdg(surface));
                }
                XdgRequest::Move {
                    surface,
                    seat,
                    serial,
                } => {
                    let seat = Seat::from_resource(&seat).unwrap();
                    // TODO: touch move.
                    let pointer = seat.get_pointer().unwrap();

                    // Check that this surface has a click grab.
                    if !pointer.has_grab(serial) {
                        return;
                    }

                    let start_data = pointer.grab_start_data().unwrap();

                    // If the focus was for a different surface, ignore the request.
                    if start_data.focus.is_none()
                        || !start_data
                            .focus
                            .as_ref()
                            .unwrap()
                            .0
                            .as_ref()
                            .same_client_as(surface.get_surface().unwrap().as_ref())
                    {
                        return;
                    }

                    let toplevel = SurfaceKind::Xdg(surface.clone());

                    let space = workspaces.space_by_seat(&seat).unwrap();
                    space.move_request(toplevel, &seat, serial, start_data)
                }
                XdgRequest::Resize {
                    surface,
                    seat,
                    serial,
                    edges,
                } => {
                    let seat = Seat::from_resource(&seat).unwrap();
                    // TODO: touch resize.
                    let pointer = seat.get_pointer().unwrap();

                    // Check that this surface has a click grab.
                    if !pointer.has_grab(serial) {
                        return;
                    }

                    let start_data = pointer.grab_start_data().unwrap();

                    // If the focus was for a different surface, ignore the request.
                    if start_data.focus.is_none()
                        || !start_data
                            .focus
                            .as_ref()
                            .unwrap()
                            .0
                            .as_ref()
                            .same_client_as(surface.get_surface().unwrap().as_ref())
                    {
                        return;
                    }

                    let toplevel = SurfaceKind::Xdg(surface.clone());

                    let space = workspaces.space_by_seat(&seat).unwrap();
                    space.resize_request(toplevel, &seat, serial, start_data, edges)
                }
                XdgRequest::AckConfigure {
                    surface,
                    configure: Configure::Toplevel(configure),
                    ..
                } => {
                    if let Some(space) = workspaces.space_by_surface(&surface) {
                        space.ack_configure(surface, configure);
                    }
                }
                XdgRequest::Fullscreen {
                    surface, output, ..
                } => {
                    // NOTE: This is only one part of the solution. We can set the
                    // location and configure size here, but the surface should be rendered fullscreen
                    // independently from its buffer size
                    if let Some(wl_surface) = surface.get_surface() {
                        let toplevel = SurfaceKind::Xdg(surface.clone());
                        if let Some(space) = if let Some(output) = output {
                            if let Some(output_requested) = workspaces
                                .output_by_wl(&output)
                                .map(|x| String::from(x.name()))
                            {
                                let space_requested_id = workspaces
                                    .space_by_output_name(&output_requested)
                                    .map(|x| x.id());
                                let current_space_id =
                                    workspaces.space_by_surface(&wl_surface).map(|x| x.id());
                                if space_requested_id != current_space_id {
                                    if let Some(current_space) =
                                        workspaces.space_by_surface(&wl_surface)
                                    {
                                        current_space.remove_toplevel(toplevel.clone());
                                    }
                                    if let Some(space) =
                                        workspaces.space_by_output_name(&output_requested)
                                    {
                                        space.new_toplevel(toplevel.clone());
                                    }
                                }
                                workspaces.space_by_output_name(&output_requested)
                            } else {
                                None
                            }
                        } else {
                            workspaces.space_by_surface(&wl_surface)
                        } {
                            space.fullscreen_request(toplevel, true);
                        }
                    }
                }
                XdgRequest::UnFullscreen { surface } => {
                    if let Some(wl_surface) = surface.get_surface() {
                        if let Some(space) = workspaces.space_by_surface(wl_surface) {
                            let toplevel = SurfaceKind::Xdg(surface.clone());
                            space.fullscreen_request(toplevel, false);
                        }
                    }
                }
                XdgRequest::Maximize { surface } => {
                    if let Some(wl_surface) = surface.get_surface() {
                        if let Some(space) = workspaces.space_by_surface(wl_surface) {
                            let toplevel = SurfaceKind::Xdg(surface.clone());
                            space.maximize_request(toplevel, true);
                        }
                    }
                }
                XdgRequest::UnMaximize { surface } => {
                    if let Some(wl_surface) = surface.get_surface() {
                        if let Some(space) = workspaces.space_by_surface(wl_surface) {
                            let toplevel = SurfaceKind::Xdg(surface.clone());
                            space.maximize_request(toplevel, false);
                        }
                    }
                }
                _ => (),
            }
        },
        None,
    );

    /*
    smithay::wayland::shell::wlr_layer::wlr_layer_shell_init(
        &mut *display.borrow_mut(),
        move |event, mut ddata| match event {
            LayerShellRequest::NewLayerSurface {
                surface,
                output,
                layer,
                ..
            } => {
                let output_map = layer_output_map.borrow();
                let anvil_state = ddata.get::<AnvilState<BackendData>>().unwrap();

                let output = output.and_then(|output| output_map.find_by_output(&output));
                let output = output.unwrap_or_else(|| {
                    output_map
                        .find_by_position(anvil_state.pointer_location.to_i32_round())
                        .unwrap_or_else(|| output_map.with_primary().unwrap())
                });

                if let Some(wl_surface) = surface.get_surface() {
                    output.add_layer_surface(wl_surface.clone());

                    layer_window_map.borrow_mut().layers.insert(surface, layer);
                }
            }
            LayerShellRequest::AckConfigure { .. } => {}
        },
        log.clone(),
    );
    */

    ShellHandles {
        xdg_state: xdg_shell_state,
        workspaces,
        popups,
    }
}

#[derive(Default)]
pub struct SurfaceData {
    pub buffer: Option<wl_buffer::WlBuffer>,
    pub texture: Option<BufferTextures>,
    pub geometry: Option<Rectangle<i32, Logical>>,
    pub buffer_dimensions: Option<Size<i32, Physical>>,
    pub buffer_scale: i32,
    pub userdata: UserDataMap,
}

impl SurfaceData {
    pub fn update_buffer(&mut self, attrs: &mut SurfaceAttributes) {
        match attrs.buffer.take() {
            Some(BufferAssignment::NewBuffer { buffer, .. }) => {
                // new contents
                self.buffer_dimensions = buffer_dimensions(&buffer);
                self.buffer_scale = attrs.buffer_scale;
                                
                if let Some(old_buffer) = std::mem::replace(&mut self.buffer, Some(buffer)) {
                    if &old_buffer != self.buffer.as_ref().unwrap() {
                    old_buffer.release();
                    }
                }
                self.texture = None;
            }
            Some(BufferAssignment::Removed) => {
                // remove the contents
                if let Some(buffer) = self.buffer.take() {
                    buffer.release();
                };
                self.buffer_dimensions = None;
                self.texture = None;
            }
            None => {}
        }
    }

    /// Returns the size of the surface.
    pub fn size(&self) -> Option<Size<i32, Logical>> {
        self.buffer_dimensions
            .map(|dims| dims.to_logical(self.buffer_scale))
    }

    /// Checks if the surface's input region contains the point.
    pub fn contains_point(&self, attrs: &SurfaceAttributes, point: Point<f64, Logical>) -> bool {
        let size = match self.size() {
            None => return false, // If the surface has no size, it can't have an input region.
            Some(size) => size,
        };

        let rect = Rectangle {
            loc: (0, 0).into(),
            size,
        }
        .to_f64();

        // The input region is always within the surface itself, so if the surface itself doesn't contain the
        // point we can return false.
        if !rect.contains(point) {
            return false;
        }

        // If there's no input region, we're done.
        if attrs.input_region.is_none() {
            return true;
        }

        attrs
            .input_region
            .as_ref()
            .unwrap()
            .contains(point.to_i32_floor())
    }

    /// Send the frame callback if it had been requested
    pub fn send_frame(attrs: &mut SurfaceAttributes, time: u32) {
        for callback in attrs.frame_callbacks.drain(..) {
            callback.done(time);
        }
    }

    pub fn userdata(&self) -> &UserDataMap {
        &self.userdata
    }
}

fn surface_commit(
    surface: &wl_surface::WlSurface,
    workspaces: &mut Workspaces,
    popups: &mut Vec<PopupKind>,
) {
    #[cfg(feature = "xwayland")]
    super::xwayland::commit_hook(surface);

    if !is_sync_subsurface(surface) {
        // Update the buffer of all child surfaces
        with_surface_tree_upward(
            surface,
            (),
            |_, _, _| TraversalAction::DoChildren(()),
            |_, states, _| {
                states
                    .data_map
                    .insert_if_missing(|| RefCell::new(SurfaceData::default()));
                let mut data = states
                    .data_map
                    .get::<RefCell<SurfaceData>>()
                    .unwrap()
                    .borrow_mut();
                data.update_buffer(&mut *states.cached_state.current::<SurfaceAttributes>());
            },
            |_, _, _| true,
        );
    }

    let toplevel = workspaces.toplevel_by_surface(surface);
    if let Some(toplevel) = toplevel {
        // send the initial configure if relevant
        #[allow(irrefutable_let_patterns)]
        if let SurfaceKind::Xdg(ref toplevel) = toplevel {
            let initial_configure_sent = with_states(surface, |states| {
                states
                    .data_map
                    .get::<Mutex<XdgToplevelSurfaceRoleAttributes>>()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .initial_configure_sent
            })
            .unwrap();
            if !initial_configure_sent {
                toplevel.send_configure();
            }
        }

        if let Some(space) = workspaces.space_by_surface(surface) {
            space.commit(toplevel.clone());
        }
    }

    if let Some(popup) = popups.iter().find(|x| x.get_surface() == Some(surface)) {
        let PopupKind::Xdg(ref popup) = popup;
        let initial_configure_sent = with_states(surface, |states| {
            states
                .data_map
                .get::<Mutex<XdgPopupSurfaceRoleAttributes>>()
                .unwrap()
                .lock()
                .unwrap()
                .initial_configure_sent
        })
        .unwrap();
        if !initial_configure_sent {
            // TODO: properly recompute the geometry with the whole of positioner state
            popup.send_configure();
        }
    }

    /*
    if let Some(layer) = window_map.layers.find(surface) {
        // send the initial configure if relevant
        let initial_configure_sent = with_states(surface, |states| {
            states
                .data_map
                .get::<Mutex<LayerSurfaceAttributes>>()
                .unwrap()
                .lock()
                .unwrap()
                .initial_configure_sent
        })
        .unwrap();
        if !initial_configure_sent {
            layer.surface.send_configure();
        }

        if let Some(output) = output_map.borrow().find_by_layer_surface(surface) {
            window_map.layers.arange_layers(output);
        }
    }
    */
}

pub fn child_popups<'a>(popups: impl DoubleEndedIterator<Item=&'a PopupKind>, base: &'a wl_surface::WlSurface) -> impl Iterator<Item=&'a PopupKind> {
    popups
        .rev()
        .filter(move |w| w.parent().as_ref() == Some(base))
}
