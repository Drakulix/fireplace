use crate::{
    handler::ActiveOutput,
    state::{Fireplace, BackendData, SurfaceData}
};
use anyhow::{Context, Result};
use edid_rs::{parse as edid_parse, MonitorDescriptor};
use image::ImageBuffer;
use smithay::{
    backend::{
        drm::{DrmDevice, DrmEvent},
        egl::{EGLDisplay, EGLContext, context::{PixelFormatRequirements, GlAttributes}},
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        session::{Session, Signal, auto::AutoSession, AsErrno},
        udev::{UdevBackend, UdevEvent, driver},
        renderer::{Frame, Renderer, Transform, gles2::Gles2Renderer},
    },
    reexports::{
        calloop::{EventLoop, LoopHandle, generic::Generic, Interest, Mode, PostAction, timer::Timer},
        drm::control::{crtc, connector, property, Device as ControlDevice},
        input::Libinput,
        nix::{fcntl::OFlag, sys::stat::dev_t},
        wayland_server::protocol::wl_output,
    },
    utils::{
        Point, Logical,
        signaling::{Signaler, Linkable}
    },
    wayland::{
        seat::CursorImageStatus,
        output::{Mode as OutputMode, PhysicalProperties}
    },
};

use std::{
    cell::RefCell,
    collections::HashMap,
    path::PathBuf,
    os::unix::io::{AsRawFd, RawFd},
};

mod cursor;
pub use self::cursor::Cursor;

mod drm;
use self::drm::*;

mod surface;
use self::surface::*;
pub use self::surface::RenderSurface;

use super::render::{render_space, draw_cursor, gl_import_bitmap};

#[derive(Clone)]
pub struct SessionFd(RawFd);
impl AsRawFd for SessionFd {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

pub fn init_udev(event_loop: &mut EventLoop<'static, Fireplace>, state: &mut Fireplace) -> Result<()> {
    let (mut session, notifier) = AutoSession::new(None).context("Failed to create Session")?;
    let signaler = notifier.signaler();

    let udev_backend = UdevBackend::new(session.seat(), None)?;
    let mut libinput_context = Libinput::new_with_udev::<LibinputSessionInterface<AutoSession>>(session.clone().into());
    libinput_context.udev_assign_seat(&session.seat()).unwrap();
    let mut libinput_backend = LibinputInputBackend::new(libinput_context, None);
    libinput_backend.link(signaler.clone());

    let libinput_event_source = event_loop
        .handle()
        .insert_source(libinput_backend, move |event, _, anvil_state| {
            anvil_state.process_input_event(event)
        }).unwrap();
    let session_event_source = event_loop
        .handle()
        .insert_source(notifier, |(), &mut (), _anvil_state| {}).unwrap();

    let handle = event_loop.handle();
    for (dev, path) in udev_backend.device_list() {
        state.device_added(handle.clone(), &mut session, signaler.clone(), dev, path.into())?;
    }

    let handle = event_loop.handle();
    let udev_event_source = event_loop
        .handle()
        .insert_source(udev_backend, move |event, _, state| match match event {
            UdevEvent::Added { device_id, path } => state.device_added(handle.clone(), &mut session, signaler.clone(), device_id, path),
            UdevEvent::Changed { device_id } => state.device_changed(&mut session, device_id),
            UdevEvent::Removed { device_id } => state.device_removed(&mut session, device_id),
        } {
            Ok(()) => { slog_scope::info!("Successfully handled udev event") },
            Err(err) => { slog_scope::error!("Unable to handle udev event : {}", err) },
        }).unwrap();
    
    state.tokens.push(libinput_event_source);
    state.tokens.push(session_event_source);
    state.tokens.push(udev_event_source);

    Ok(())
}

impl Fireplace {
    fn device_added<S, E>(&mut self, handle: LoopHandle<'static, Fireplace>, session: &mut S, signaler: Signaler<Signal>, device_id: dev_t, path: PathBuf) -> Result<()>
    where
        S: Session<Error=E>,
        E: std::error::Error + Send + Sync + AsErrno + 'static,
    {
        let fd = SessionFd(session.open(&path, OFlag::O_RDWR | OFlag::O_CLOEXEC | OFlag::O_NOCTTY | OFlag::O_NONBLOCK)?);
        let mut drm = DrmDevice::new(fd.clone(), false, None)?;

        let driver = driver(device_id)?.map(|x| x.to_string_lossy().into_owned());
        
        // we do not actually need to use the gbm platform, mesa supports EGLDevice just a well.
        let egl_device = EGLDeviceEXT::new(fd.clone(), slog_scope::logger())?;
        let egl_display = EGLDisplay::new(&egl_device, None)?;
        let egl_context = if driver.as_ref().map(|x| &**x) == Some("nvidia") {
            EGLContext::new_with_config(
                &egl_display,
                GlAttributes {
                    version: (3, 0),
                    profile: None,
                    debug: cfg!(debug_assertions),
                    vsync: true,
                },
                PixelFormatRequirements {
                    hardware_accelerated: Some(true),
                    ..Default::default()
                },
                None,
            )?
        } else {
            EGLContext::new(&egl_display, None)?
        };

        // enumerate our outputs
        let mut surfaces = HashMap::new();
        for (conn, crtc) in display_configuration(&mut drm)?.iter() {
            let conn_info = drm.get_connector(*conn)?;
            let crtc_info = drm.get_crtc(*crtc)?;
            let mode = crtc_info.mode().unwrap_or(conn_info.modes()[0]);
            let mut surface = drm.create_surface(*crtc, mode, &[*conn])?;
            surface.link(signaler.clone());

            let target = match driver.as_ref().map(|x| &**x) {
                Some("nvidia") => {
                    RenderSurface::new_eglstream(surface, &egl_display, &egl_context)?
                },
                _ => {
                    RenderSurface::new_gbm(surface, fd.clone(), &egl_context)?
                },
            };

            let mode = OutputMode {
                size: (mode.size().0 as i32, mode.size().1 as i32).into(),
                refresh: (mode.vrefresh() * 1000) as i32,
            };

            let other_short_name;
            let interface_short_name = match conn_info.interface() {
                connector::Interface::DVII => "DVI-I",
                connector::Interface::DVID => "DVI-D",
                connector::Interface::DVIA => "DVI-A",
                connector::Interface::SVideo => "S-VIDEO",
                connector::Interface::DisplayPort => "DP",
                connector::Interface::HDMIA => "HDMI-A",
                connector::Interface::HDMIB => "HDMI-B",
                connector::Interface::EmbeddedDisplayPort => "eDP",
                other => {
                    other_short_name = format!("{:?}", other);
                    &other_short_name
                }
            };
            let output_name = format!("{}-{}", interface_short_name, conn_info.interface_id());

            let edid_prop = get_prop(&drm, *conn, "EDID")?;
            let edid_info = drm.get_property(edid_prop)?;
            let mut manufacturer = "Unknown".into();
            let mut model = "Unknown".into();
            let props = drm.get_properties(*conn)?;
            let (ids, vals) = props.as_props_and_values();
            for (&id, &val) in ids.iter().zip(vals.iter()) {
                if id == edid_prop {
                    if let property::Value::Blob(edid_blob) =
                        edid_info.value_type().convert_value(val)
                    {
                        let blob = drm.get_property_blob(edid_blob)?;
                        let mut reader = std::io::Cursor::new(blob);
                        if let Some(edid) = edid_parse(&mut reader).ok() {
                            manufacturer = {
                                let id = edid.product.manufacturer_id;
                                let code = [id.0, id.1, id.2];
                                get_manufacturer(&code).into()
                            };
                            model = if let Some(MonitorDescriptor::MonitorName(name)) = edid.descriptors.0
                                .iter()
                                .find(|x| matches!(x, MonitorDescriptor::MonitorName(_)))
                            {
                                name.clone()
                            } else {
                                format!("{}", edid.product.product_code)
                            };
                        }
                    }
                    break;
                }
            }            

            let (phys_w, phys_h) = conn_info.size().unwrap_or((0, 0));
            let mut workspaces = self.workspaces.borrow_mut();
            workspaces.add_output(
                &output_name,
                PhysicalProperties {
                    size: (phys_w as i32, phys_h as i32).into(),
                    subpixel: wl_output::Subpixel::Unknown,
                    make: manufacturer,
                    model,
                },
                mode,
            );

            let timer = Timer::new()?;

            let data = SurfaceData {
                output: output_name,
                size: mode.size,
                surface: target,
                render_timer: timer.handle(),
            };

            // re-render timer
            handle
                .insert_source(timer, |(dev_id, crtc), _, state| {
                    if let Err(err) = state.render(dev_id, Some(crtc)) {
                        slog_scope::error!("Error rendering: {}", err);
                    }
                })
                .unwrap();
            surfaces.insert(*crtc, data);
        }

        if surfaces.is_empty() {
            return Ok(());
        }
        
        // create our renderer
        let renderer = unsafe { Gles2Renderer::new(egl_context, None)? };
        let pointer = cursor::Cursor::load(&slog_scope::logger());

        let restart_handle = handle.clone();
        let restart_token = signaler.register(move |signal| match signal {
            Signal::ActivateSession | Signal::ActivateDevice { .. } => {
                restart_handle.insert_idle(move |state| {
                    if let Err(err) = state.render(device_id, None) {
                        slog_scope::error!("Error rendering on {:?}: {}", device_id, err);   
                    }
                    // TODO do re-schedule
                });
            }
            _ => {}
        });
        drm.link(signaler.clone());
        let event_dispatcher = Dispatcher::new(
            drm,
            move |event, _, state: &mut Fireplace| match event {
                DrmEvent::VBlank(crtc) => {
                    {
                        if let Some(backend) = state.udev.get_mut(&device_id) {
                            if let Some(surface) = backend.surfaces.get_mut(&crtc) {
                                if let Err(err) = surface.surface.frame_submitted() {
                                    slog_scope::error!("Error submitting frame on {:?}: {}", device_id, err);
                                    return;
                                }
                            }
                        }
                    }
                    if let Err(err) = state.render(device_id, Some(crtc)) {
                        slog_scope::error!("Error rendering on {:?}: {}", device_id, err);
                    } // TODO re-schedule
                },
                DrmEvent::Error(error) => {
                    slog_scope::error!("{:?}", error);
                }
            },
        );
        let registration_token = handle.register_dispatcher(event_dispatcher.clone()).unwrap();

        let data = BackendData {
            registration_token,
            _restart_token: restart_token,
            surfaces,
            renderer,
            pointer,
            pointer_images: Vec::new(),
        };
        self.udev.insert(device_id, data);

        if let Err(err) = self.render(device_id, None) {
            slog_scope::error!("Error rendering on {:?}: {}", device_id, err);
        }

        Ok(())
    }

    fn device_changed<S, E>(&mut self, session: &mut S, device: dev_t) -> Result<()>
    where
        S: Session<Error=E>,
        E: AsErrno,
    {
        Ok(())
    }

    fn device_removed<S, E>(&mut self, session: &mut S, device: dev_t) -> Result<()>
    where
        S: Session<Error=E>,
        E: AsErrno,
    {
        Ok(())
    }

    pub fn render(&mut self, dev_id: dev_t, crtc: Option<crtc::Handle>) -> Result<()> {
        let device_backend = match self.udev.get_mut(&dev_id) {
            Some(backend) => backend,
            None => {
                slog_scope::error!("Trying to render on non-existent backend {}", dev_id);
                return Ok(());
            }
        };

        for surface in device_backend.surfaces
            .iter_mut()
            .filter(|(c, _)| crtc.map(|x| x == **c).unwrap_or(true))
            .map(|(_, surf)| surf)
        {
            let mut workspaces = self.workspaces.borrow_mut();
            let scale = workspaces.output_by_name(&surface.output).unwrap().scale();
            let space = workspaces.space_by_output_name(&surface.output).unwrap();
            let popups = self.popups.borrow();

            let seats = &self.seats;
            let output_name = &surface.output;
            let frame = device_backend
                .pointer
                .get_image(scale.ceil() as u32, self.start_time.elapsed().as_millis() as u32);
            let hotspot: Point<i32, Logical> = (frame.xhot as i32, frame.yhot as i32).into();
            let pointer_images = &mut device_backend.pointer_images;
            let renderer = &mut device_backend.renderer;
            let pointer_image = pointer_images
                .iter()
                .find_map(|(image, texture)| if image == &frame { Some(texture) } else { None })
                .cloned()
                .unwrap_or_else(|| {
                    let image =
                        ImageBuffer::from_raw(frame.width, frame.height, &*frame.pixels_rgba).unwrap();
                    let texture = gl_import_bitmap(renderer, &image).expect("Failed to import cursor bitmap");
                    pointer_images.push((frame, texture.clone()));
                    texture
                });

            surface.surface.bind(&mut device_backend.renderer)?;
            device_backend.renderer.render(surface.size, surface.surface.transform(Transform::Normal), |renderer, frame| {

                // render the cursors for all seats
                // TODO tint the cursors by seats
                for seat in seats.iter().filter(|seat| {
                    seat.user_data().get::<ActiveOutput>().map(|name| &*name.0.borrow() == output_name).unwrap_or(false)
                }) {
                    if let Some(position) = seat.get_pointer()
                        .map(|ptr| ptr.current_location())
                    {
                        let userdata = seat.user_data();
                        let status_ref = userdata.get::<RefCell<CursorImageStatus>>().unwrap();
                        let mut status = status_ref.borrow_mut();
                        let mut reset = false;
                        if let CursorImageStatus::Image(ref surface) = *status {
                            reset = !surface.as_ref().is_alive();
                        }
                        if reset {
                            *status = CursorImageStatus::Default;
                        }
                        match &*status {
                            &CursorImageStatus::Default => {
                                frame.render_texture_at(
                                    &pointer_image,
                                    (position - hotspot.to_f64()).to_physical(scale as f64).to_i32_round(),
                                    1, scale as f64,
                                    Transform::Normal,
                                    1.0
                                )?;
                            },
                            &CursorImageStatus::Image(ref surface) => {
                                draw_cursor(dev_id, renderer, frame, surface, position.to_i32_round(), scale)?;
                            }
                            CursorImageStatus::Hidden => {},
                        }
                    }
                }
                Ok(())
            }).and_then(|x| x)?;
            match surface.surface.queue_buffer(&mut device_backend.renderer)
            {
                Ok(_) => {
            space.send_frames(self.start_time.elapsed().as_millis() as u32);
                },
                Err(err) => {
                    use smithay::{
                        backend::{
                            SwapBuffersError,
                            drm::DrmError,
                            egl::{SwapBuffersError as EGLSwapBuffersError, EGLError},
                        },
                        reexports::drm,
                    };

                    let reschedule = match err {
                        SwapBuffersError::AlreadySwapped => false,
                        SwapBuffersError::TemporaryFailure(err) => !matches!(
                            err.downcast_ref::<DrmError>(),
                            Some(&DrmError::DeviceInactive)
                                | Some(&DrmError::Access {
                                    source: drm::SystemError::PermissionDenied,
                                    ..
                                })
                        ),
                        SwapBuffersError::ContextLost(err) => matches!(
                            err.downcast_ref::<EGLSwapBuffersError>(),
                            Some(&EGLSwapBuffersError::EGLSwapBuffers(EGLError::Unknown(0x3353)))
                        ),
                    };

                    if reschedule {
                        slog_scope::debug!("Rescheduling frame");
                        surface.render_timer.add_timeout(
                            std::time::Duration::from_millis(1000),// /*a second*/ / 60 /*refresh rate*/),
                            (dev_id, surface.surface.crtc()),
                        );
                    }
                }
            }
        }

        Ok(())
    }
}
