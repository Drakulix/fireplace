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
