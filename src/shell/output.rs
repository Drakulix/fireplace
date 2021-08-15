use std::{cell::RefCell, rc::Rc};

use smithay::{
    reexports::{
        wayland_protocols::xdg_shell::server::xdg_toplevel,
        wayland_server::{
            protocol::{
                wl_output,
                wl_surface::{self, WlSurface},
            },
            Display, Global, UserDataMap,
        },
    },
    utils::{Logical, Point, Rectangle, Size, Raw},
    wayland::{
        compositor::{with_surface_tree_downward, SubsurfaceCachedState, TraversalAction},
        output::{self, Mode, PhysicalProperties},
    },
};

// The minimum DPI at which we turn on a scale of 2
const HIDPI_DPI_STEP: f32 = 2.0 * 96.0;
// The minimum screen height at which we turn on a scale of 2
const HIDPI_MIN_HEIGHT: i32 = 1200;
// 1 inch = 25.4 mm
const MM_PER_INCH: f32 = 25.4;

#[derive(Debug)]
pub struct Output {
    name: String,
    output: output::Output,
    global: Option<Global<wl_output::WlOutput>>,
    surfaces: Vec<WlSurface>,
    layer_surfaces: RefCell<Vec<wl_surface::WlSurface>>,
    current_mode: Mode,
    scale: f32,
    output_scale: i32,
    location: Point<i32, Logical>,
    userdata: UserDataMap,
}

// Some manufacturers hardcode the aspect-ratio of the output in the physical size field.
fn phys_size_is_aspect_ratio(size: &Size<i32, Raw>) -> bool {
    (size.w == 1600 && size.h == 900) ||
    (size.w == 1600 && size.h == 1000) ||
    (size.w == 160 && size.h == 90) ||
    (size.w == 160 && size.h == 100) ||
    (size.w == 16 && size.h == 9) ||
    (size.w == 16 && size.h == 10)
}

impl Output {
    pub fn new<N>(
        name: N,
        location: Point<i32, Logical>,
        display: &mut Display,
        physical: PhysicalProperties,
        mode: Mode,
    ) -> Self
    where
        N: AsRef<str>,
    {
        let physical_size = physical.size.clone();
        let (output, global) = output::Output::new(display, name.as_ref().into(), physical, None);

        let (width, height) = mode.size.into();
        let scale = if height < HIDPI_MIN_HEIGHT { 1.0 }
            else if phys_size_is_aspect_ratio(&physical_size) { 1.0 }
            else if physical_size.h == 0 || physical_size.w == 0 { 1.0 }
            else {
                let dpi_x = width as f32 / (physical_size.w as f32 / MM_PER_INCH);
                let dpi_y = height as f32 / (physical_size.h as f32 / MM_PER_INCH);
                slog_scope::debug!("Output DPI: {}x{}", dpi_x, dpi_y);
                    (((dpi_x / HIDPI_DPI_STEP) * 4.0).floor() / 4.0)
                    .min(((dpi_y / HIDPI_DPI_STEP) * 4.0).floor() / 4.0)
            };
        let output_scale = scale.ceil() as i32;

        output.change_current_state(Some(mode), None, Some(output_scale), Some(location));
        output.set_preferred(mode);

        Self {
            name: name.as_ref().to_owned(),
            global: Some(global),
            output,
            location,
            surfaces: Vec::new(),
            layer_surfaces: Default::default(),
            current_mode: mode,
            scale,
            output_scale,
            userdata: Default::default(),
        }
    }

    pub fn userdata(&self) -> &UserDataMap {
        &self.userdata
    }

    pub fn geometry(&self) -> Rectangle<i32, Logical> {
        let loc = self.location();
        let size = self.size();

        Rectangle { loc, size }
    }

    pub fn size(&self) -> Size<i32, Logical> {
        self.current_mode
            .size
            .to_f64()
            .to_logical(self.scale as f64)
            .to_i32_round()
    }

    pub fn location(&self) -> Point<i32, Logical> {
        self.location
    }

    pub fn set_location(&mut self, to: Point<i32, Logical>) {
        self.location = to;
        self.output.change_current_state(None, None, None, Some(to));
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn current_mode(&self) -> Mode {
        self.current_mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.output.delete_mode(self.current_mode);
        self
            .output
            .change_current_state(Some(mode), None, Some(self.output_scale), None);
        self.output.set_preferred(mode);
        self.current_mode = mode;
    }

    pub fn owns(&self, wl: &wl_output::WlOutput) -> bool {
        self.output.owns(wl)
    }

    /// Add a layer surface to this output
    pub fn add_layer_surface(&self, layer: wl_surface::WlSurface) {
        self.layer_surfaces.borrow_mut().push(layer);
    }

    /// Get all layer surfaces assigned to this output
    pub fn layer_surfaces(&self) -> Vec<wl_surface::WlSurface> {
        self.layer_surfaces.borrow().iter().cloned().collect()
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        self.global.take().unwrap().destroy();
    }
}

/*
pub struct OutputMap {
    display: Rc<RefCell<Display>>,
    outputs: Vec<Output>,
    window_map: Rc<RefCell<crate::state::WindowMap>>,
    logger: slog::Logger,
}

impl OutputMap {
    pub fn new(
        display: Rc<RefCell<Display>>,
        window_map: Rc<RefCell<crate::state::WindowMap>>,
        logger: ::slog::Logger,
    ) -> Self {
        Self {
            display,
            outputs: Vec::new(),
            window_map,
            logger,
        }
    }

    pub fn arrange(&mut self) {
        // First recalculate the outputs location
        let mut output_x = 0;
        for output in self.outputs.iter_mut() {
            let output_x_shift = output_x - output.location.x;

            // If the scale changed we shift all windows on that output
            // so that the location of the window will stay the same on screen
            if output_x_shift != 0 {
                let mut window_map = self.window_map.borrow_mut();

                for surface in output.surfaces.iter() {
                    let toplevel = window_map.find(surface);

                    if let Some(toplevel) = toplevel {
                        let current_location = window_map.location(&toplevel);

                        if let Some(mut location) = current_location {
                            if output.geometry().contains(location) {
                                location.x += output_x_shift;
                                window_map.set_location(&toplevel, location);
                            }
                        }
                    }
                }
            }

            output.location.x = output_x;
            output.location.y = 0;

            output
                .output
                .change_current_state(None, None, None, Some(output.location));

            output_x += output.size().w;
        }

        // Check if any windows are now out of outputs range
        // and move them to the primary output
        let primary_output_location = self.with_primary().map(|o| o.location()).unwrap_or_default();
        let mut window_map = self.window_map.borrow_mut();

        // TODO: This is a bit unfortunate, we save the windows in a temp vector
        // cause we can not call window_map.set_location within the closure.
        let mut windows_to_move = Vec::new();
        window_map.with_windows_from_bottom_to_top(|kind, _, &bbox| {
            let within_outputs = self.outputs.iter().any(|o| o.geometry().overlaps(bbox));

            if !within_outputs {
                windows_to_move.push((kind.to_owned(), primary_output_location));
            }
        });
        for (window, location) in windows_to_move.drain(..) {
            window_map.set_location(&window, location);
        }

        // Update the size and location for maximized and fullscreen windows
        window_map.with_windows_from_bottom_to_top(|kind, location, _| {
            if let crate::window_map::Kind::Xdg(xdg) = kind {
                if let Some(state) = xdg.current_state() {
                    if state.states.contains(xdg_toplevel::State::Maximized)
                        || state.states.contains(xdg_toplevel::State::Fullscreen)
                    {
                        let output_geometry = if let Some(output) = state.fullscreen_output.as_ref() {
                            self.find_by_output(output).map(|o| o.geometry())
                        } else {
                            self.find_by_position(location).map(|o| o.geometry())
                        };

                        if let Some(geometry) = output_geometry {
                            if location != geometry.loc {
                                windows_to_move.push((kind.to_owned(), geometry.loc));
                            }

                            let res = xdg.with_pending_state(|pending_state| {
                                pending_state.size = Some(geometry.size);
                            });

                            if res.is_ok() {
                                xdg.send_configure();
                            }
                        }
                    }
                }
            }
        });
        for (window, location) in windows_to_move.drain(..) {
            window_map.set_location(&window, location);
        }

        for output in self.outputs.iter() {
            window_map.layers.arange_layers(output);
        }
    }

    

    

    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }

    pub fn with_primary(&self) -> Option<&Output> {
        self.outputs.get(0)
    }

    pub fn find_by_layer_surface(&self, surface: &wl_surface::WlSurface) -> Option<&Output> {
        self.find(|o| {
            o.layer_surfaces
                .borrow()
                .iter()
                .any(|s| s.as_ref().equals(surface.as_ref()))
        })
    }

    pub fn find_by_name<N>(&self, name: N) -> Option<&Output>
    where
        N: AsRef<str>,
    {
        self.find(|o| o.name == name.as_ref())
    }

    pub fn find_by_position(&self, position: Point<i32, Logical>) -> Option<&Output> {
        self.find(|o| o.geometry().contains(position))
    }

    pub fn find_by_index(&self, index: usize) -> Option<&Output> {
        self.outputs.get(index)
    }

    pub fn update<F>(&mut self, mode: Option<Mode>, scale: Option<f32>, mut f: F)
    where
        F: FnMut(&Output) -> bool,
    {
        let output = self.outputs.iter_mut().find(|o| f(&**o));

        if let Some(output) = output {
            if let Some(mode) = mode {
                output.output.delete_mode(output.current_mode);
                output
                    .output
                    .change_current_state(Some(mode), None, Some(output.output_scale), None);
                output.output.set_preferred(mode);
                output.current_mode = mode;
            }

            if let Some(scale) = scale {
                // Calculate in which direction the scale changed
                let rescale = output.scale() / scale;

                {
                    // We take the current location of our toplevels and move them
                    // to the same location using the new scale
                    let mut window_map = self.window_map.borrow_mut();
                    for surface in output.surfaces.iter() {
                        let toplevel = window_map.find(surface);

                        if let Some(toplevel) = toplevel {
                            let current_location = window_map.location(&toplevel);

                            if let Some(location) = current_location {
                                let output_geometry = output.geometry();

                                if output_geometry.contains(location) {
                                    let mut toplevel_output_location =
                                        (location - output_geometry.loc).to_f64();
                                    toplevel_output_location.x *= rescale as f64;
                                    toplevel_output_location.y *= rescale as f64;
                                    window_map.set_location(
                                        &toplevel,
                                        output_geometry.loc + toplevel_output_location.to_i32_round(),
                                    );
                                }
                            }
                        }
                    }
                }

                let output_scale = scale.round() as i32;
                output.scale = scale;

                if output.output_scale != output_scale {
                    output.output_scale = output_scale;
                    output.output.change_current_state(
                        Some(output.current_mode),
                        None,
                        Some(output_scale),
                        None,
                    );
                }
            }
        }

        self.arrange();
    }

    pub fn update_by_name<N: AsRef<str>>(&mut self, mode: Option<Mode>, scale: Option<f32>, name: N) {
        self.update(mode, scale, |o| o.name() == name.as_ref())
    }

    pub fn update_scale_by_name<N: AsRef<str>>(&mut self, scale: f32, name: N) {
        self.update_by_name(None, Some(scale), name)
    }

    pub fn update_mode_by_name<N: AsRef<str>>(&mut self, mode: Mode, name: N) {
        self.update_by_name(Some(mode), None, name)
    }

    pub fn refresh(&mut self) {
        // Clean-up dead surfaces
        self.outputs.iter_mut().for_each(|o| {
            o.surfaces.retain(|s| s.as_ref().is_alive());
            o.layer_surfaces.borrow_mut().retain(|s| s.as_ref().is_alive());
        });

        let window_map = self.window_map.clone();

        window_map
            .borrow()
            .with_windows_from_bottom_to_top(|kind, location, &bbox| {
                for output in self.outputs.iter_mut() {
                    // Check if the bounding box of the toplevel intersects with
                    // the output, if not no surface in the tree can intersect with
                    // the output.
                    if !output.geometry().overlaps(bbox) {
                        if let Some(surface) = kind.get_surface() {
                            with_surface_tree_downward(
                                surface,
                                (),
                                |_, _, _| TraversalAction::DoChildren(()),
                                |wl_surface, _, _| {
                                    if output.surfaces.contains(wl_surface) {
                                        output.output.leave(wl_surface);
                                        output.surfaces.retain(|s| s != wl_surface);
                                    }
                                },
                                |_, _, _| true,
                            )
                        }
                        continue;
                    }

                    if let Some(surface) = kind.get_surface() {
                        with_surface_tree_downward(
                            surface,
                            location,
                            |_, states, location| {
                                let mut location = *location;
                                let data = states.data_map.get::<RefCell<SurfaceData>>();

                                if data.is_some() {
                                    if states.role == Some("subsurface") {
                                        let current = states.cached_state.current::<SubsurfaceCachedState>();
                                        location += current.location;
                                    }

                                    TraversalAction::DoChildren(location)
                                } else {
                                    // If the parent surface is unmapped, then the child surfaces are hidden as
                                    // well, no need to consider them here.
                                    TraversalAction::SkipChildren
                                }
                            },
                            |wl_surface, states, &loc| {
                                let data = states.data_map.get::<RefCell<SurfaceData>>();

                                if let Some(size) = data.and_then(|d| d.borrow().size()) {
                                    let surface_rectangle = Rectangle { loc, size };

                                    if output.geometry().overlaps(surface_rectangle) {
                                        // We found a matching output, check if we already sent enter
                                        if !output.surfaces.contains(wl_surface) {
                                            output.output.enter(wl_surface);
                                            output.surfaces.push(wl_surface.clone());
                                        }
                                    } else {
                                        // Surface does not match output, if we sent enter earlier
                                        // we should now send leave
                                        if output.surfaces.contains(wl_surface) {
                                            output.output.leave(wl_surface);
                                            output.surfaces.retain(|s| s != wl_surface);
                                        }
                                    }
                                } else {
                                    // Maybe the the surface got unmapped, send leave on output
                                    if output.surfaces.contains(wl_surface) {
                                        output.output.leave(wl_surface);
                                        output.surfaces.retain(|s| s != wl_surface);
                                    }
                                }
                            },
                            |_, _, _| true,
                        )
                    }
                }
            });
    }
}
*/