// Re-export only the actual code, and then only use this re-export
// The `generated` module below is just some boilerplate to properly isolate stuff
// and avoid exposing internal details.
//
// You can use all the types from my_protocol as if they went from `wayland_client::protocol`.
pub use generated::server::{wl_eglstream, wl_eglstream_display, wl_eglstream_controller};

mod generated {
    // The generated code tends to trigger a lot of warnings
    // so we isolate it into a very permissive module
    #![allow(dead_code,non_camel_case_types,unused_unsafe,unused_variables)]
    #![allow(non_upper_case_globals,non_snake_case,unused_imports)]

    pub mod server {

        mod eglstream {
            use smithay::reexports::{wayland_commons, wayland_server};
            // These imports are used by the generated code
            pub(crate) use wayland_server::{Main, AnonymousObject, Resource, ResourceMap};
            pub(crate) use wayland_commons::map::{Object, ObjectMetadata};
            pub(crate) use wayland_commons::{Interface, MessageGroup};
            pub(crate) use wayland_commons::wire::{Argument, MessageDesc, ArgumentType, Message};
            pub(crate) use wayland_commons::smallvec;
            pub(crate) use wayland_server::sys;
            pub(crate) use wayland_server::protocol::{wl_buffer, wl_surface};
            include!(concat!(env!("OUT_DIR"), "/wl_eglstream.rs"));
        }
        mod eglstream_controller {
            use smithay::reexports::{wayland_commons, wayland_server};
            // These imports are used by the generated code
            pub(crate) use wayland_server::{Main, AnonymousObject, Resource, ResourceMap};
            pub(crate) use wayland_commons::map::{Object, ObjectMetadata};
            pub(crate) use wayland_commons::{Interface, MessageGroup};
            pub(crate) use wayland_commons::wire::{Argument, MessageDesc, ArgumentType, Message};
            pub(crate) use wayland_commons::smallvec;
            pub(crate) use wayland_server::sys;
            pub(crate) use wayland_server::protocol::{wl_buffer, wl_surface};
            include!(concat!(env!("OUT_DIR"), "/wl_eglstream_controller.rs"));
        }
        pub use self::eglstream::*;
        pub use self::eglstream_controller::*;
    }
}

use smithay::{
    backend::{
        allocator::{Fourcc, Modifier, dmabuf::{Dmabuf, DmabufFlags}},
        egl::{EGLDisplay, display::EGLDisplayHandle},
    },
    reexports::{
        nix,
        wayland_server::{
            Client, Display, Filter, Global, Main,
            protocol::{
                wl_buffer::WlBuffer,
                wl_surface::WlSurface,
            }
        }
    },
    utils::{Size, Buffer},
};

use std::{
    cell::RefCell,
    convert::TryFrom,
    fmt,
    ptr,
    sync::{Arc, Weak},
};

use crate::backend::egl as ffi;

pub struct EGLStream {
    pub display: Weak<EGLDisplayHandle>,
    pub handle: ffi::types::EGLStreamKHR,
    pub y_inverted: bool,
    pub size: Size<i32, Buffer>,
    buffers: RefCell<Vec<(ffi::types::EGLImageKHR, Option<Dmabuf>)>>,
    current_buffer: RefCell<Option<(ffi::types::EGLImageKHR, Dmabuf)>>,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum EGLStreamDisplayError {
    ExtensionMissing(&'static str),
}

impl fmt::Display for EGLStreamDisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EGLStreamDisplayError::ExtensionMissing(ext) => write!(f, "Extension missing: {}", ext),
        }
    }
}

pub fn init_eglstream_globals<F>(
    display: &mut Display,
    egl_display: &EGLDisplay,
    client_filter: F,
) -> Result<(
        Global<wl_eglstream_display::WlEglstreamDisplay>, 
        Global<wl_eglstream_controller::WlEglstreamController>,
    ), EGLStreamDisplayError>
where
    F: FnMut(Client) -> bool + Clone + 'static
{
    if !egl_display.get_extensions().iter().any(|ext| ext == "EGL_KHR_stream_cross_process_fd") {
        return Err(EGLStreamDisplayError::ExtensionMissing("EGL_KHR_stream_cross_process_fd"));
    }
    
    if !egl_display.get_extensions().iter().any(|ext| ext == "EGL_NV_stream_consumer_eglimage") {
        return Err(EGLStreamDisplayError::ExtensionMissing("EGL_NV_stream_consumer_eglimage"));
    }
    
    let display_handle_display = egl_display.get_display_handle();
    let display_handle_controller = display_handle_display.clone();

    Ok((
        display.create_global_with_filter(1, Filter::new(move |(eglstream_display, _version): (Main<wl_eglstream_display::WlEglstreamDisplay>, u32), _, _| {
            let display_handle = display_handle_display.clone();
            eglstream_display.quick_assign(move |_, req, _| {
                match req {
                    wl_eglstream_display::Request::CreateStream {
                        id,
                        width,
                        height,
                        handle,
                        _type,
                        attribs,
                    } => {
                        let stream: ffi::types::EGLStreamKHR = unsafe { ffi::CreateStreamFromFileDescriptorKHR(**display_handle, handle) };
                        if stream == ffi::NO_STREAM_KHR {
                            id.as_ref().post_error(wl_eglstream::Error::BadHandle.to_raw(), String::from("EGLStream creation failed"));
                        }
                        id.as_ref().user_data().set(|| EGLStream {
                            display: Arc::downgrade(&display_handle),
                            handle: stream,
                            y_inverted: attribs.iter().any(|x| *x as u32 == wl_eglstream::Attrib::YInverted.to_raw()),
                            size: (width, height).into(),
                            buffers: RefCell::new(Vec::with_capacity(4)),
                            current_buffer: RefCell::new(None),
                        });
                        id.quick_assign(|_, _, _| {});
                        slog_scope::trace!("Created a new eglstream wl_buffer.");
                    },
                    wl_eglstream_display::Request::SwapInterval {
                        stream,
                        interval,
                    } => {
                        let _ = (stream, interval);
                    },
                }
            });
            eglstream_display.caps(wl_eglstream_display::Cap::StreamFd.to_raw() as i32);
        }), client_filter.clone()),
        display.create_global_with_filter(1, Filter::new(move |(eglstream_controller, _): (Main<wl_eglstream_controller::WlEglstreamController>, u32), _, _| {
            let display_handle = display_handle_controller.clone();
            eglstream_controller.quick_assign(move |_, req, _| {
                fn attach_egl_stream_consumer(
                    display: *const nix::libc::c_void,
                    surface: WlSurface,
                    buffer: WlBuffer,
                    //formats: &Vec<Format>,
                    attribs: Vec<isize>,
                ) {
                    if let Some(stream) = buffer.as_ref().user_data().get::<EGLStream>() {
                        if unsafe { ffi::StreamImageConsumerConnectNV(
                            display,
                            stream.handle,
                            /*
                            1,
                            [Modifier::Linear].as_ptr(),
                            */
                            0,
                            ptr::null_mut(),
                            attribs.as_ptr(),
                        )} != ffi::TRUE {
                            surface.as_ref().post_error(wl_eglstream::Error::BadAlloc.to_raw(), String::from("Failed to connect EGLStream to consumer"));
                            return;
                        }
                    } else {
                        surface.as_ref().post_error(wl_eglstream::Error::BadHandle.to_raw(), String::from("Invalid EGLStream handle in attach request"));
                    }
                }

                match req {
                    wl_eglstream_controller::Request::AttachEglstreamConsumer {
                        wl_surface,
                        wl_resource,
                    } => {
                        attach_egl_stream_consumer(**display_handle, wl_surface, wl_resource, Vec::new())
                    },
                    wl_eglstream_controller::Request::AttachEglstreamConsumerAttribs {
                        wl_surface,
                        wl_resource,
                        attribs,
                    } => {
                        // TODO: What are these attribs? 
                        //attach_egl_stream_consumer(**display_handle, wl_surface, wl_resource, attribs)
                        let _ = (wl_surface, wl_resource, attribs);
                        unreachable!("We advertise version 1");
                    }
                }
            })
        }), client_filter)
    ))
}

pub fn handle_eglstream_events(buffer: &WlBuffer) -> Option<(ffi::types::EGLImageKHR, Dmabuf)> {
    if let Some(stream) = buffer.as_ref().user_data().get::<EGLStream>() {
        slog_scope::trace!("Got buffer with stream");
        if let Some(display) = stream.display.upgrade() {
            let mut event: ffi::types::EGLenum = 0;
            let aux = ptr::null_mut();
            let mut result = unsafe { ffi::QueryStreamConsumerEventNV(**display, stream.handle, 0, &mut event as *mut _, aux) };
            while result == ffi::TRUE {
                match event {
                    ffi::STREAM_IMAGE_ADD_NV => {
                        slog_scope::debug!("EGLStream event ADD_NV");
                        let image = unsafe { ffi::CreateImage(**display, ffi::NO_CONTEXT, ffi::STREAM_CONSUMER_IMAGE_NV, stream.handle as ffi::types::EGLClientBuffer, ptr::null()) };
                        if image == ffi::NO_IMAGE_KHR {
                            buffer.as_ref().post_error(wl_eglstream::Error::BadAlloc.to_raw(), String::from("Failed to allocate EGLImage for EGLStream"));
                        }
                        stream.buffers.borrow_mut().push((image, None));
                    },
                    ffi::STREAM_IMAGE_REMOVE_NV => {
                        slog_scope::debug!("EGLStream event REMOVE_NV");
                        let image = aux as ffi::types::EGLImageKHR;
                        stream.buffers.borrow_mut().retain(|(img, _)| *img == image);
                        unsafe { ffi::DestroyImage(**display, image); }
                    },
                    ffi::STREAM_IMAGE_AVAILABLE_NV => {
                        slog_scope::debug!("EGLStream event IMAGE_AVAILABLE_NV");
                        unsafe {
                            let mut image = ffi::NO_IMAGE_KHR;
                            if ffi::StreamAcquireImageNV(**display, stream.handle, &mut image as *mut _, ffi::NO_SYNC) == ffi::FALSE {
                                buffer.as_ref().post_error(wl_eglstream::Error::BadHandle.to_raw(), String::from("Failed to acquire EGLImage for EGLStream"));
                            }

                            let mut buffers = stream.buffers.borrow_mut();
                            let (img, buf) = buffers.iter_mut().find(|(img, _)| *img == image).unwrap();

                            if buf.is_none() {
                                let mut format: nix::libc::c_int = 0;
                                let mut num_planes: nix::libc::c_int = 0;
                                let mut modifier: ffi::types::EGLuint64KHR = 0;
                                if ffi::ExportDMABUFImageQueryMESA(**display, image, &mut format as *mut _, &mut num_planes as *mut _, &mut modifier as *mut _) == ffi::FALSE {
                                    ffi::StreamReleaseImageNV(**display, stream.handle, &mut image as *mut _, ffi::NO_SYNC);
                                    buffer.as_ref().post_error(wl_eglstream::Error::BadAlloc.to_raw(), String::from("Failed to export EGLImage of EGLStream"));
                                }

                                let mut fds: Vec<nix::libc::c_int> = Vec::with_capacity(num_planes as usize);
                                let mut strides: Vec<ffi::types::EGLint> = Vec::with_capacity(num_planes as usize);
                                let mut offsets: Vec<ffi::types::EGLint> = Vec::with_capacity(num_planes as usize);
                                if ffi::ExportDMABUFImageMESA(**display, image, fds.as_mut_ptr(), strides.as_mut_ptr(), offsets.as_mut_ptr()) == ffi::FALSE {
                                    ffi::StreamReleaseImageNV(**display, stream.handle, &mut image as *mut _, ffi::NO_SYNC);
                                    buffer.as_ref().post_error(wl_eglstream::Error::BadAlloc.to_raw(), String::from("Failed to export EGLImage of EGLStream"));
                                }
                                fds.set_len(num_planes as usize);
                                strides.set_len(num_planes as usize);
                                offsets.set_len(num_planes as usize);

                                let mut dma = Dmabuf::builder(stream.size, Fourcc::try_from(format as u32).expect("Unknown format"), if stream.y_inverted { DmabufFlags::Y_INVERT } else { DmabufFlags::empty() });
                                for i in 0..num_planes {
                                    dma.add_plane(fds[i as usize], i as u32, offsets[i as usize] as u32, strides[i as usize] as u32, Modifier::from(modifier));
                                }

                                *buf = dma.build();
                            }

                            if let Some(new) = buf.clone().map(|dma| (*img, dma)) {
                                if let Some((mut old_img, _)) = stream.current_buffer.borrow_mut().replace(new) {
                                    ffi::StreamReleaseImageNV(**display, stream.handle, &mut old_img as *mut _, ffi::NO_SYNC);
                                }
                            }
                        }
                    },
                    x => {
                        slog_scope::warn!("Unknown EGLStream event: {}", x);
                    }
                }
                result = unsafe { ffi::QueryStreamConsumerEventNV(**display, stream.handle, 0, &mut event as *mut _, aux) };
            }

            if result == ffi::FALSE {
                match unsafe { ffi::GetError() as u32 } {
                    ffi::SUCCESS | ffi::RESOURCE_BUSY_EXT => {},
                    x => {
                        slog_scope::warn!("Committed buffer uses destroyed/invalid EGLStream: {}", smithay::backend::egl::EGLError::from(x));
                        buffer.as_ref().post_error(wl_eglstream::Error::BadHandle.to_raw(), String::from("Committed buffer uses destroyed/invalid EGLStream"));
                    },
                };
            }

            return stream.current_buffer.borrow().clone();
        }
    }
    None
}