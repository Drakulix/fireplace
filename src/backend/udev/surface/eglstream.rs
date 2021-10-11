use anyhow::{Context, Result};
use smithay::backend::{
    drm::DrmSurface,
    egl::{
        EGLError, SwapBuffersError,
        display::EGLDisplayHandle,
        native::{EGLNativeDisplay, EGLNativeSurface, EGLPlatform}
    }
};
use smithay::reexports::{
    drm::control::{
        Device as ControlDevice,
        dumbbuffer::DumbBuffer,
        framebuffer,
    },
    nix::{libc::{major, minor, c_void}, sys::stat::fstat},
};

use std::{
    cell::Cell,
    ffi::CStr,
    ptr,
    os::unix::{
        io::AsRawFd,
    },
    sync::{Arc, atomic::{AtomicPtr, Ordering}},
};

use super::super::SessionFd as Fd;
use crate::backend::egl::{self as ffi, wrap_egl_call};

pub struct EGLDeviceEXT {
    device: ffi::types::EGLDeviceEXT,
    raw: Fd,
}

unsafe impl Send for EGLDeviceEXT {}

impl EGLDeviceEXT {
    pub fn new(raw: Fd, log: slog::Logger) -> Result<EGLDeviceEXT> {
        smithay::backend::egl::ffi::make_sure_egl_is_loaded()?;
        ffi::load_with(|sym| unsafe { smithay::backend::egl::get_proc_address(sym) });
        ffi::StreamConsumerAcquireAttribNV::load_with(|sym| unsafe { smithay::backend::egl::get_proc_address(sym) });
        ffi::StreamConsumerReleaseAttribNV::load_with(|sym| unsafe { smithay::backend::egl::get_proc_address(sym) });

        let device = unsafe {
            // the first step is to query the list of extensions without any display, if supported
            let dp_extensions = {
                let p = wrap_egl_call(|| {
                    ffi::QueryString(ffi::NO_DISPLAY, ffi::EXTENSIONS as i32)
                })?;

                // this possibility is available only with EGL 1.5 or EGL_EXT_platform_base, otherwise
                // `eglQueryString` returns an error
                if p.is_null() {
                    vec![]
                } else {
                    let p = CStr::from_ptr(p);
                    let list = String::from_utf8(p.to_bytes().to_vec()).unwrap_or_else(|_| String::new());
                    list.split(' ').map(|e| e.to_string()).collect::<Vec<_>>()
                }
            };
            slog::debug!(log, "EGL No-Display Extensions: {:?}", dp_extensions);

            // we need either EGL_EXT_device_base or EGL_EXT_device_enumeration &_query
            if !dp_extensions.iter().any(|x|  x == "EGL_EXT_device_base") {
                if !(
                    dp_extensions.iter().any(|x| x == "EGL_EXT_device_enumeration")
                 && dp_extensions.iter().any(|x| x == "EGL_EXT_device_query")
                ) {
                    anyhow::bail!("Device does not support EGL_EXT_device");
                }
            }

            let mut num_devices = 0;
            wrap_egl_call(|| ffi::QueryDevicesEXT(0, ptr::null_mut(), &mut num_devices))?;
            if num_devices == 0 {
                anyhow::bail!("Device does not support EGL_EXT_device");
            }

            let mut devices = Vec::with_capacity(num_devices as usize);
            wrap_egl_call(|| ffi::QueryDevicesEXT(num_devices, devices.as_mut_ptr(), &mut num_devices))?;
            devices.set_len(num_devices as usize);
            slog::debug!(log, "Devices: {:#?}, Count: {}", devices, num_devices);
                            
            let drm_rdev = fstat(raw.as_raw_fd()).expect("Unable to get device id").st_rdev;
            slog::debug!(log, "rdev: {:?} ({}:{})", drm_rdev, major(drm_rdev), minor(drm_rdev));
            let name = std::fs::read_dir(format!("/sys/dev/char/{}:{}/device/drm", major(drm_rdev), minor(drm_rdev)))?
                .find(|x| x.as_ref().ok()
                    .and_then(|entry| entry.file_name().to_str().map(|x| x.starts_with("card")))
                    .unwrap_or(false)
                ).context("Unable to find device")??;
            let path = format!("/dev/dri/{}", name.file_name().to_str().unwrap());

            devices
                .into_iter()
                .find(|device| {
                    *device != ffi::NO_DEVICE_EXT
                        && {
                            let device_extensions = {
                                let p = ffi::QueryDeviceStringEXT(*device, ffi::EXTENSIONS as i32);
                                if p.is_null() {
                                    vec![]
                                } else {
                                    let p = CStr::from_ptr(p);
                                    let list = String::from_utf8(p.to_bytes().to_vec())
                                        .unwrap_or_else(|_| String::new());
                                    list.split(' ').map(|e| e.to_string()).collect::<Vec<_>>()
                                }
                            };
                            slog::debug!(log, "EGL Device Extensions: {:?}", device_extensions);

                            device_extensions.iter().any(|s| *s == "EGL_EXT_device_drm")
                        }
                        && {
                            let egl_path = {
                                let p = ffi::QueryDeviceStringEXT(
                                    *device,
                                    ffi::DRM_DEVICE_FILE_EXT as i32,
                                );
                                if p.is_null() {
                                    String::new()
                                } else {
                                    let p = CStr::from_ptr(p);
                                    String::from_utf8(p.to_bytes().to_vec()).unwrap_or_else(|_| String::new())
                                }
                            };

                            egl_path == path
                        }
                }).ok_or(anyhow::anyhow!("Device does not support EGL_EXT_device"))?
        };

        Ok(EGLDeviceEXT {
            device,
            raw
        })
    }
}

impl EGLNativeDisplay for EGLDeviceEXT {
    fn supported_platforms(&self) -> Vec<EGLPlatform<'_>> {
		vec![
	        // see: https://www.khronos.org/registry/EGL/extensions/EXT/EGL_EXT_platform_device.txt
            EGLPlatform::new(
                ffi::PLATFORM_DEVICE_EXT,
                "PLATFORM_DEVICE_EXT",
                self.device as *mut _,
                vec![
                    ffi::DRM_MASTER_FD_EXT as ffi::EGLint,
                    self.raw.as_raw_fd(),
                    ffi::NONE as i32,
                ],
                &["EGL_EXT_platform_device"],
            )
        ]
    }

    fn surface_type(&self) -> smithay::backend::egl::ffi::EGLint {
        ffi::STREAM_BIT_KHR as smithay::backend::egl::ffi::EGLint
    }
}

pub struct EglStreamSurface<A: AsRawFd + 'static> {
    stream: Cell<Option<ffi::types::EGLStreamKHR>>,
    drm: DrmSurface<A>,
    test_fb: Cell<Option<(DumbBuffer, framebuffer::Handle)>>,
    surface: AtomicPtr<c_void>,
    mode: Cell<(i32, i32)>,
    logger: slog::Logger,
}

impl<A: AsRawFd + 'static> Drop for EglStreamSurface<A> {
    fn drop(&mut self) { 
        if let Some((old_db, old_fb)) = self.test_fb.replace(None) {
            let _ = self.drm.destroy_framebuffer(old_fb);
            let _ = self.drm.destroy_dumb_buffer(old_db);
        };
    }
}

impl<A: AsRawFd + 'static> EglStreamSurface<A> {
    pub fn new(drm: DrmSurface<A>, logger: slog::Logger) -> EglStreamSurface<A> {
        let mode = drm.pending_mode().size();
        EglStreamSurface {
            stream: Cell::new(None),
            drm,
            test_fb: Cell::new(None),
            surface: AtomicPtr::new(std::ptr::null_mut()),
            mode: Cell::new((mode.0 as i32, mode.1 as i32)),
            logger,
        }
    }

    fn create_stream(&self, handle: &Arc<EGLDisplayHandle>) -> Result<(), EGLError> {
        let output_attribs = [
            ffi::DRM_PLANE_EXT as isize,
            Into::<u32>::into(self.drm.plane()) as isize,
            ffi::DRM_CRTC_EXT as isize,
            Into::<u32>::into(self.drm.crtc()) as isize,
            ffi::NONE as isize,
        ];

        let extensions = {
            let p =
                unsafe { CStr::from_ptr(ffi::QueryString(***handle, ffi::EXTENSIONS as i32)) };
            let list = String::from_utf8(p.to_bytes().to_vec()).unwrap_or_else(|_| String::new());
            list.split(' ').map(|e| e.to_string()).collect::<Vec<_>>()
        };

        if !extensions.iter().any(|s| *s == "EGL_EXT_output_base")
            || !extensions.iter().any(|s| *s == "EGL_EXT_output_drm")
            || !extensions.iter().any(|s| *s == "EGL_KHR_stream")
            || !extensions
                .iter()
                .any(|s| *s == "EGL_NV_output_drm_flip_event")
            || !extensions
                .iter()
                .any(|s| *s == "EGL_EXT_stream_consumer_egloutput")
            || !extensions
                .iter()
                .any(|s| *s == "EGL_KHR_stream_producer_eglsurface")
        {
            slog::error!(self.logger, "Extension for EGLStream surface creation missing");
            return Err(EGLError::BadNativeWindow);
        }

        let mut num_layers = 0;
        if unsafe {
            ffi::GetOutputLayersEXT(
                ***handle,
                output_attribs.as_ptr(),
                ptr::null_mut(),
                0,
                &mut num_layers,
            )
        } == 0
        {
            slog::error!(
                self.logger,
                "Failed to acquire Output Layer. Attributes {:?}", output_attribs
            );
            return Err(EGLError::BadParameter);
        }
        if num_layers == 0 {
            slog::error!(self.logger, "Failed to find Output Layer");
            return Err(EGLError::BadParameter);
        }
        let mut layers = Vec::with_capacity(num_layers as usize);
        if unsafe {
            ffi::GetOutputLayersEXT(
                ***handle,
                output_attribs.as_ptr(),
                layers.as_mut_ptr(),
                num_layers,
                &mut num_layers,
            )
        } == 0
        {
            slog::error!(self.logger, "Failed to get Output Layer");
            return Err(EGLError::BadParameter);
        }
        unsafe {
            layers.set_len(num_layers as usize);
        }

        let layer = layers[0];
        unsafe {
            let mut interval = 1;
            ffi::QueryOutputLayerAttribEXT(***handle, layer, ffi::MIN_SWAP_INTERVAL as i32, &mut interval);
            slog::debug!(self.logger, "Min swap interval: {}", interval);
            if interval == 0 { interval = 1; }
            ffi::OutputLayerAttribEXT(***handle, layer, ffi::SWAP_INTERVAL_EXT as i32, interval);
        }

        let stream_attributes = [
            ffi::STREAM_FIFO_LENGTH_KHR as i32,
            0,
            ffi::CONSUMER_AUTO_ACQUIRE_EXT as i32,
            ffi::FALSE as i32,
            ffi::CONSUMER_ACQUIRE_TIMEOUT_USEC_KHR as i32,
            0,
            ffi::NONE as i32,
        ];

        let stream = unsafe { ffi::CreateStreamKHR(***handle, stream_attributes.as_ptr()) };
        if stream == ffi::NO_STREAM_KHR {
            slog::error!(self.logger, "Failed to create egl stream");
            return Err(EGLError::BadParameter);
        }

        let mut val = 0;
        unsafe { ffi::QueryStreamKHR(***handle, stream, ffi::STREAM_STATE_KHR, &mut val as *mut _) };
        slog::debug!(self.logger, "Stream State: 0x{:x}", val);

        if unsafe { ffi::StreamConsumerOutputEXT(***handle, stream, layer) } == 0 {
            slog::error!(self.logger, "Failed to link Output Layer as Stream Consumer");
            return Err(EGLError::BadParameter);
        }

        let mut val = 0;
        unsafe { ffi::QueryStreamKHR(***handle, stream, ffi::STREAM_STATE_KHR, &mut val as *mut _) };
        slog::debug!(self.logger, "Stream State: 0x{:x}", val);
        
        self.stream.set(Some(stream));

        Ok(())
    }
}

// HACK: We are single threaded anyway and smithay is by default as well.
//  Hopefully EGL is?
unsafe impl<A: AsRawFd + 'static> Send for EglStreamSurface<A> {}
unsafe impl<A: AsRawFd + 'static> Sync for EglStreamSurface<A> {}

unsafe impl<A: AsRawFd + 'static> EGLNativeSurface for EglStreamSurface<A> {
    fn create(
        &self,
        display: &Arc<EGLDisplayHandle>,
        config_id: ffi::types::EGLConfig,
    ) -> Result<*const c_void, EGLError> {
        use smithay::reexports::drm::{
            buffer::DrmFourcc as Fourcc,
        };

        self.create_stream(display)?;
        
        let (w, h) = self.mode.get();
        slog::info!(self.logger, "Creating stream surface with size: ({}:{})", w, h);
        let surface_attributes = [
            ffi::WIDTH as i32,
            w,
            ffi::HEIGHT as i32,
            h,
            ffi::NONE as i32,
        ];

        let db = self.drm.create_dumb_buffer((w as u32, h as u32), Fourcc::Argb8888, 32).map_err(|_| EGLError::ContextLost)?;
        let fb = self.drm.add_framebuffer(&db, 24, 32).map_err(|_| EGLError::ContextLost)?;
        self.drm.commit([&(fb, self.drm.plane())].iter().cloned(), true).map_err(|_| EGLError::ContextLost)?;

        if let Some((old_db, old_fb)) = self.test_fb.replace(Some((db, fb))) {
            let _ = self.drm.destroy_framebuffer(old_fb);
            let _ = self.drm.destroy_dumb_buffer(old_db);
        };

        let surface = unsafe {
            ffi::CreateStreamProducerSurfaceKHR(
                ***display,
                config_id,
                self.stream.get().unwrap(),
                surface_attributes.as_ptr(),
            )
        };
        if surface == ffi::NO_SURFACE {
            slog::error!(self.logger, "Failed to create surface: 0x{:X}", unsafe {
                ffi::GetError()
            });
        }

        let mut val = 0;
        unsafe { ffi::QueryStreamKHR(***display, self.stream.get().unwrap(), ffi::STREAM_STATE_KHR, &mut val as *mut _) };
        slog::debug!(self.logger, "Stream State: 0x{:x}", val);

        self.surface.store(surface as *mut _, Ordering::SeqCst);

        Ok(surface)
    }

    fn needs_recreation(&self) -> bool {
        self.stream.get().is_none() || self.drm.commit_pending()
    }

    fn resize(&self, width: i32, height: i32, _dx: i32, _dy: i32) -> bool {
        if self.mode.get() != (width, height) {
            self.stream.set(None);
            self.mode.set((width, height));
        }
        true
    }

    fn swap_buffers(
        &self,
        display: &Arc<EGLDisplayHandle>,
        surface: ffi::types::EGLSurface,
    ) -> Result<(), SwapBuffersError> {
        let acquire_attributes = [
            ffi::DRM_FLIP_EVENT_DATA_NV as isize,
            Into::<u32>::into(self.drm.crtc()) as isize,
            ffi::NONE as isize,
        ];

        let stream = self.stream.get().unwrap();

        let mut val = 0;
        unsafe { ffi::QueryStreamKHR(***display, stream, ffi::STREAM_STATE_KHR, &mut val as *mut _) };
        slog::debug!(self.logger, "Stream State (PRE SWAP): 0x{:x}", val);

        let res = wrap_egl_call(|| unsafe { ffi::SwapBuffers(***display, surface as *const _) })
            .map_err(SwapBuffersError::EGLSwapBuffers)?;
        slog::debug!(self.logger, "res: {}", res);
        
        let mut val = 0;
        unsafe { ffi::QueryStreamKHR(***display, stream, ffi::STREAM_STATE_KHR, &mut val as *mut _) };
        slog::debug!(self.logger, "Stream State (AFTER SWAP): 0x{:x}", val);
        wrap_egl_call(|| unsafe {
            ffi::StreamConsumerAcquireAttribNV(
                ***display,
                stream,
                acquire_attributes.as_ptr(),
            );
        })
        .map_err(SwapBuffersError::EGLSwapBuffers)?;

        let mut val = 0;
        unsafe { ffi::QueryStreamKHR(***display, stream, ffi::STREAM_STATE_KHR, &mut val as *mut _) };
        slog::debug!(self.logger, "Stream State (AFTER ACQUIRE): 0x{:x}", val);
        
        Ok(())
    }
}