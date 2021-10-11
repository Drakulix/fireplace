use image::{ImageBuffer, Rgba};
use smithay::{
    backend::{
        allocator::{Buffer, dmabuf::Dmabuf},
        renderer::{
            buffer_type, Bind, BufferType, Frame, ImportAll, ImportDma, Renderer, Texture, Transform, Unbind,
            gles2::{Gles2Renderer, Gles2Texture, Gles2Error}
        },
    },
    reexports::{
        nix::sys::stat::dev_t,
        wayland_server::protocol::{wl_buffer, wl_surface},
    },
    utils::{Logical, Point, Buffer as BufferCoords, Rectangle},
    wayland::{
        compositor::{
            with_surface_tree_upward, with_states, Damage, SubsurfaceCachedState, SurfaceAttributes, TraversalAction,
        },
        seat::CursorImageAttributes,
    },
};

use std::{
    cell::RefCell,
    collections::HashMap,
    sync::Mutex,
};

use crate::{
    backend::udev::DevId,
    shell::{child_popups, SurfaceData, layout::Layout, window::PopupKind},
    state::BackendData,
    wayland::handle_eglstream_events,
};

static PLACEHOLDER: &[u8] = &[255, 0, 255, 255];

pub struct BufferTextures {
    buffer: wl_buffer::WlBuffer,
    damage: Vec<Rectangle<i32, BufferCoords>>,
    textures: HashMap<Option<DevId>, Box<dyn std::any::Any>>,
}

impl Drop for BufferTextures {
    fn drop(&mut self) {
        self.buffer.release();
    }
}

pub fn render_space<'a, R, E, F, T>(
    space: &dyn Layout,
    scale: f32,
    popups: &[PopupKind],
    device: Option<DevId>,
    renderer: &mut R,
    frame: &mut F,
    other_backends: &mut [(&dev_t, &mut BackendData)],
) -> Result<(), E>
where
    R: Renderer<Error = E, TextureId = T, Frame = F> + ImportDma + ImportAll + CpuAccess,
    F: Frame<Error = E, TextureId = T>,
    T: Texture + 'static,
    E: std::error::Error,
{
    frame.clear([0.8, 0.8, 0.8, 1.0])?;

    // redraw the frame, in a simple but inneficient way
    for (toplevel_surface, location, _bounding_box) in space.windows_from_bottom_to_top() {
        if let Some(wl_surface) = toplevel_surface.get_surface() {
            // this surface is a root of a subsurface tree that needs to be drawn
            draw_surface_tree(device, renderer, frame, wl_surface, location, scale, other_backends)?;

            // furthermore, draw its popups
            let toplevel_geometry_offset: Point<i32, Logical> = (0, 0).into(); // TODO
                                                                                /*
                                                                                window_map
                                                                                    .geometry(toplevel_surface)
                                                                                    .map(|g| g.loc)
                                                                                    .unwrap_or_default();
                                                                                */

            for popup in child_popups(popups.iter(), &wl_surface) {
                let popup_location = popup.location();
                let draw_location = location + popup_location + toplevel_geometry_offset;
                if let Some(wl_surface) = popup.get_surface() {
                    draw_surface_tree(device, renderer, frame, wl_surface, draw_location, scale, other_backends)?;
                }
            }
        }
    }

    Ok(())
}

pub fn draw_cursor<R, E, F, T>(
    device: Option<DevId>,
    renderer: &mut R,
    frame: &mut F,
    surface: &wl_surface::WlSurface,
    location: Point<i32, Logical>,
    output_scale: f32,
    other_backends: &mut [(&dev_t, &mut BackendData)],
) -> Result<(), E>
where
    R: Renderer<Error = E, TextureId = T, Frame = F> + ImportDma + ImportAll + CpuAccess,
    F: Frame<Error = E, TextureId = T>,
    E: std::error::Error,
    T: Texture + 'static,
{
    let ret = with_states(surface, |states| {
        Some(
            states
                .data_map
                .get::<Mutex<CursorImageAttributes>>()
                .unwrap()
                .lock()
                .unwrap()
                .hotspot,
        )
    })
    .unwrap_or(None);
    let delta = match ret {
        Some(h) => h,
        None => {
            slog_scope::warn!(
                "Trying to display as a cursor a surface that does not have the CursorImage role."
            );
            (0, 0).into()
        }
    };
    draw_surface_tree(device, renderer, frame, surface, location - delta, output_scale, other_backends)
}

fn draw_surface_tree<R, E, F, T>(
    device: Option<DevId>,
    renderer: &mut R,
    frame: &mut F,
    root: &wl_surface::WlSurface,
    location: Point<i32, Logical>,
    output_scale: f32,
    other_backends: &mut [(&dev_t, &mut BackendData)],
) -> Result<(), E>
where
    R: Renderer<Error = E, TextureId = T, Frame = F> + ImportDma + ImportAll + CpuAccess,
    F: Frame<Error = E, TextureId = T>,
    E: std::error::Error,
    T: Texture + 'static,
{
    let mut result = Ok(());

    with_surface_tree_upward(
        root,
        location,
        |_surface, states, location| {
            let mut location = *location;
            // Pull a new buffer if available
            if let Some(data) = states.data_map.get::<RefCell<SurfaceData>>() {
                let mut data = data.borrow_mut();
                let attributes = states.cached_state.current::<SurfaceAttributes>();
                if data.texture.is_none() {
                    if let Some(buffer) = data.buffer.take() {
                        let damage = attributes
                            .damage
                            .iter()
                            .map(|dmg| match dmg {
                                Damage::Buffer(rect) => *rect,
                                // TODO also apply transformations
                                Damage::Surface(rect) => rect.to_buffer(attributes.buffer_scale),
                            })
                            .collect::<Vec<_>>();
                        
                        data.texture = Some(BufferTextures {
                            buffer,
                            damage,
                            textures: HashMap::new(),
                        });
                    }
                }

                if let Some(texture) = data.texture.as_mut() {
                    let maybe_dma = handle_eglstream_events(&texture.buffer);
                    if !texture.textures.contains_key(&device) {
                        let client_id = texture.buffer.as_ref().client().and_then(|client| client.data_map().get::<DevId>().cloned());
                        match buffer_type(&texture.buffer) {
                            Some(BufferType::Dma) | None => {
                                // Not device local
                                let dma = texture.buffer.as_ref().user_data().get::<Dmabuf>().cloned().unwrap_or_else(|| maybe_dma.unwrap().1);
                                match renderer.import_dmabuf(&dma) {
                                    Ok(m) => {
                                        // hardware-accelerated copy, yeah!
                                        slog_scope::trace!("Imported dmabuf");
                                        texture.textures.insert(device, Box::new(m) as Box<dyn std::any::Any + 'static>);
                                    },
                                    Err(x) => {
                                        slog_scope::trace!("Failed to import dmabuf cross-device: {}", x);
                                        // cpu copy path...
                                        let m = cross_device_cpu_copy(other_backends, client_id, renderer, &dma);
                                        texture.textures.insert(device, Box::new(m) as Box<dyn std::any::Any + 'static>);
                                    }
                                }
                            },
                            _ /* SHM or device local */ => {
                                match renderer.import_buffer(&texture.buffer, Some(states), &texture.damage) {
                                    Some(Ok(m)) => {
                                        texture.textures.insert(device, Box::new(m) as Box<dyn std::any::Any + 'static>);
                                    }
                                    Some(Err(err)) => {
                                        slog_scope::warn!("Error loading buffer on device ({:?}): {:?}", device, err);
                                    }
                                    None => {
                                        slog_scope::error!("Unknown buffer format for: {:?}", &texture.buffer);
                                    }
                                }
                            }
                        }
                    }
                }

                // Now, should we be drawn ?
                if data.texture.as_ref().map(|x| x.textures.contains_key(&device)).unwrap_or(false) {
                    // if yes, also process the children
                    if states.role == Some("subsurface") {
                        let current = states.cached_state.current::<SubsurfaceCachedState>();
                        location += current.location;
                    }
                    TraversalAction::DoChildren(location)
                } else {
                    // we are not displayed, so our children are neither
                    TraversalAction::SkipChildren
                }
            } else {
                // we are not displayed, so our children are neither
                TraversalAction::SkipChildren
            }
        },
        |_surface, states, location| {
            let mut location = *location;
            if let Some(data) = states.data_map.get::<RefCell<SurfaceData>>() {
                let mut data = data.borrow_mut();
                let buffer_scale = data.buffer_scale;
                if let Some(texture) = data
                    .texture
                    .as_mut()
                    .and_then(|x| x.textures.get_mut(&device))
                    .and_then(|x| <dyn std::any::Any>::downcast_mut::<T>(&mut **x))
                {
                    // we need to re-extract the subsurface offset, as the previous closure
                    // only passes it to our children
                    if states.role == Some("subsurface") {
                        let current = states.cached_state.current::<SubsurfaceCachedState>();
                        location += current.location;
                    }
                    if let Err(err) = frame.render_texture_at(
                        texture,
                        location
                            .to_f64()
                            .to_physical(output_scale as f64)
                            .to_i32_round(),
                        buffer_scale,
                        output_scale as f64,
                        Transform::Normal, /* TODO */
                        1.0,
                    ) {
                        result = Err(err);
                    }
                }
            }
        },
        |_, _, _| true,
    );

    result
}

pub fn cross_device_cpu_copy<R: CpuAccess>(
    other_backends: &mut [(&dev_t, &mut BackendData)],
    client_id: Option<DevId>,
    renderer: &mut R,
    dma: &Dmabuf
) -> R::Texture {
    let tex = if let Some(src_backend) = other_backends.iter_mut().find(|&&mut (k, _)| client_id.map(|id| *k == id.0).unwrap_or(false)) {
        let src_renderer = &mut src_backend.1;
        match src_renderer.renderer.export_bitmap(&dma) {
            Ok(image_buffer) => match renderer.import_bitmap(
                &image_buffer,
            ) {
                Ok(m) => Some(m),
                Err(x) => {
                    slog_scope::error!("Failed to import bitmap: {}", x);
                    None
                }
            },
            Err(x) => {
                slog_scope::error!("Failed to read out app buffer: {}", x);
                None
            }
        }
    } else { None };
        
    tex.unwrap_or_else(|| {
        let fallback_buffer = ImageBuffer::from_raw(1, 1, PLACEHOLDER).unwrap();
        renderer.import_bitmap(&fallback_buffer).expect("Failed to import fallback texture")
    })
}

pub trait CpuAccess {
    type Error: std::error::Error;
    type Texture: Texture + 'static;

    fn export_bitmap(&mut self, buffer: &Dmabuf) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Self::Error>;
    fn import_bitmap<C: std::ops::Deref<Target = [u8]>>(&mut self, bitmap: &ImageBuffer<Rgba<u8>, C>) -> Result<Self::Texture, Self::Error>;
}

impl CpuAccess for Gles2Renderer {
    type Error = Gles2Error;
    type Texture = Gles2Texture;

    fn export_bitmap(&mut self, buffer: &Dmabuf) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Self::Error> {
        use crate::backend::egl;
        
        //another HACK
        let (display, context, draw, read) = unsafe {
            (
                egl::GetCurrentDisplay(),
                egl::GetCurrentContext(),
                egl::GetCurrentSurface(egl::DRAW as i32),
                egl::GetCurrentSurface(egl::READ as i32),
            )
        };

        let (w, h) = buffer.size().into();
        self.bind(buffer.clone())?;
        
        //TODO: depends on format, we need bits per pixel instead of 4, but we just force RGBA for now
        let mut buffer = vec![0u8; (w * h * 4) as usize];
        let buffer_ptr = buffer.as_mut_ptr() as *mut _;
        self.with_context(|_renderer, gl| unsafe {
            use smithay::backend::renderer::gles2::ffi;
            gl.ReadPixels(0, 0, w, h, ffi::RGBA, ffi::UNSIGNED_BYTE, buffer_ptr);
        })?;
        self.unbind()?;

        unsafe {
            egl::MakeCurrent(display, draw, read, context);
        }
        
        //TODO optimize and re-use buffer / copy with damage
        Ok(ImageBuffer::from_raw(w as u32, h as u32, buffer).unwrap()) 
    }

    fn import_bitmap<C: std::ops::Deref<Target = [u8]>>(&mut self, bitmap: &ImageBuffer<Rgba<u8>, C>) -> Result<Self::Texture, Self::Error> {
        use smithay::backend::renderer::gles2::ffi;

        self.with_context(|renderer, gl| unsafe {
            let mut tex = 0;
            gl.GenTextures(1, &mut tex);
            gl.BindTexture(ffi::TEXTURE_2D, tex);
            gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_WRAP_S, ffi::CLAMP_TO_EDGE as i32);
            gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_WRAP_T, ffi::CLAMP_TO_EDGE as i32);
            gl.TexImage2D(
                ffi::TEXTURE_2D,
                0,
                ffi::RGBA as i32,
                bitmap.width() as i32,
                bitmap.height() as i32,
                0,
                ffi::RGBA,
                ffi::UNSIGNED_BYTE as u32,
                bitmap.as_ptr() as *const _,
            );
            gl.BindTexture(ffi::TEXTURE_2D, 0);

            Gles2Texture::from_raw(
                renderer,
                tex,
                (bitmap.width() as i32, bitmap.height() as i32).into(),
            )
        })
    }
}