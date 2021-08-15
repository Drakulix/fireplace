use smithay::{
    backend::{
        SwapBuffersError,
        renderer::{Renderer, Frame, Texture, ImportAll, Transform, BufferType, buffer_type},
    },
    reexports::wayland_server::protocol::{wl_buffer, wl_surface},
    utils::{Point, Logical},
    wayland::compositor::{
        SurfaceAttributes, TraversalAction, Damage, SubsurfaceCachedState,
        with_surface_tree_upward,
    },
};

use std::{
    cell::RefCell,
};

use crate::{
    state::Fireplace,
    shell::SurfaceData,
};

struct BufferTextures<T> {
    buffer: Option<wl_buffer::WlBuffer>,
    texture: T,
}

impl<T> Drop for BufferTextures<T> {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            buffer.release();
        }
    }
}

impl Fireplace {
    pub fn render_output<R, E, F, T>(
        &mut self,
        output_name: &str,
        renderer: &mut R,
        frame: &mut F,
    ) -> Result<(), SwapBuffersError>
    where
        R: Renderer<Error=E, TextureId=T, Frame=F> + ImportAll,
        F: Frame<Error=E, TextureId=T>,
        T: Texture + 'static,
        E: std::error::Error + Into<SwapBuffersError>,
    {
        frame.clear([0.8, 0.8, 0.8, 1.0]).map_err(Into::into)?;
        let mut workspaces = self.workspaces.borrow_mut();
        let scale = workspaces.output_by_name(output_name).unwrap().scale();
        let space = workspaces.space_by_output_name(output_name).unwrap();
        let mut result = Ok(());

        // redraw the frame, in a simple but inneficient way
        for (toplevel_surface, location, _bounding_box) in space.windows_from_bottom_to_top() {
            if let Some(wl_surface) = toplevel_surface.get_surface() {
                // this surface is a root of a subsurface tree that needs to be drawn
                if let Err(err) = draw_surface_tree(renderer, frame, wl_surface, location, scale)
                {
                    result = Err(err);
                }

                // furthermore, draw its popups
                let toplevel_geometry_offset: Point<i32, Logical> = (0, 0).into(); // TODO
                /*
                window_map
                    .geometry(toplevel_surface)
                    .map(|g| g.loc)
                    .unwrap_or_default();
                */

                self.with_child_popups(wl_surface, |popup| {
                    let popup_location = popup.location();
                    let draw_location = location + popup_location + toplevel_geometry_offset;
                    if let Some(wl_surface) = popup.get_surface() {
                        if let Err(err) =
                            draw_surface_tree(renderer, frame, wl_surface, draw_location, scale)
                        {
                            result = Err(err);
                        }
                    }
                });
            }
        };

        space.send_frames(self.start_time.elapsed().as_millis() as u32);

        result
    }
}

fn draw_surface_tree<R, E, F, T>(
    renderer: &mut R,
    frame: &mut F,
    root: &wl_surface::WlSurface,
    location: Point<i32, Logical>,
    output_scale: f32,
) -> Result<(), SwapBuffersError>
where
    R: Renderer<Error = E, TextureId = T, Frame = F> + ImportAll,
    F: Frame<Error = E, TextureId = T>,
    E: std::error::Error + Into<SwapBuffersError>,
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

                        match renderer.import_buffer(&buffer, Some(states), &damage) {
                            Some(Ok(m)) => {
                                let texture_buffer = if let Some(BufferType::Shm) = buffer_type(&buffer) {
                                    buffer.release();
                                    None
                                } else {
                                    Some(buffer)
                                };
                                data.texture = Some(Box::new(BufferTextures {
                                    buffer: texture_buffer,
                                    texture: m,
                                }))
                            }
                            Some(Err(err)) => {
                                slog_scope::warn!("Error loading buffer: {:?}", err);
                                buffer.release();
                            }
                            None => {
                                slog_scope::error!("Unknown buffer format for: {:?}", buffer);
                                buffer.release();
                            }
                        }
                    }
                }
                
                // Now, should we be drawn ?
                if data.texture.is_some() {
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
                    .and_then(|x| x.downcast_mut::<BufferTextures<T>>())
                {
                    // we need to re-extract the subsurface offset, as the previous closure
                    // only passes it to our children
                    if states.role == Some("subsurface") {
                        let current = states.cached_state.current::<SubsurfaceCachedState>();
                        location += current.location;
                    }
                    if let Err(err) = frame.render_texture_at(
                        &texture.texture,
                        location.to_f64().to_physical(output_scale as f64).to_i32_round(),
                        buffer_scale,
                        output_scale as f64,
                        Transform::Normal, /* TODO */
                        1.0,
                    ) {
                        result = Err(err.into());
                    }
                }
            }
        },
        |_, _, _| true,
    );

    result
}