use smithay::{
    backend::renderer::{buffer_type, BufferType, Frame, ImportAll, Renderer, Texture, Transform},
    reexports::{
        nix::sys::stat::dev_t,
        wayland_server::protocol::{wl_buffer, wl_surface},
    },
    utils::{Logical, Point, Buffer, Rectangle},
    wayland::compositor::{
        with_surface_tree_upward, Damage, SubsurfaceCachedState, SurfaceAttributes, TraversalAction,
    },
};

use std::{
    cell::RefCell,
    collections::HashMap,
};

use crate::{shell::{child_popups, SurfaceData, layout::Layout, window::PopupKind}};

pub struct BufferTextures {
    buffer: wl_buffer::WlBuffer,
    damage: Vec<Rectangle<i32, Buffer>>,
    textures: HashMap<dev_t, Box<dyn std::any::Any>>,
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
    device: dev_t,
    renderer: &mut R,
    frame: &mut F,
) -> Result<(), E>
where
    R: Renderer<Error = E, TextureId = T, Frame = F> + ImportAll,
    F: Frame<Error = E, TextureId = T>,
    T: Texture + 'static,
    E: std::error::Error,
{
    frame.clear([0.8, 0.8, 0.8, 1.0])?;

    // redraw the frame, in a simple but inneficient way
    for (toplevel_surface, location, _bounding_box) in space.windows_from_bottom_to_top() {
        if let Some(wl_surface) = toplevel_surface.get_surface() {
            // this surface is a root of a subsurface tree that needs to be drawn
            draw_surface_tree(device, renderer, frame, wl_surface, location, scale)?;

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
                    draw_surface_tree(device, renderer, frame, wl_surface, draw_location, scale)?;
                }
            }
        }
    }

    Ok(())
}

fn draw_surface_tree<R, E, F, T>(
    device: dev_t,
    renderer: &mut R,
    frame: &mut F,
    root: &wl_surface::WlSurface,
    location: Point<i32, Logical>,
    output_scale: f32,
) -> Result<(), E>
where
    R: Renderer<Error = E, TextureId = T, Frame = F> + ImportAll,
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
                    if !texture.textures.contains_key(&device) {
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
