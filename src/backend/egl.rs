#![allow(clippy::all, non_camel_case_types, dead_code, unused_mut, non_upper_case_globals)]
use smithay::{
    backend::egl::EGLError,
    reexports::nix::libc::{c_long, c_void},
};

pub type khronos_utime_nanoseconds_t = khronos_uint64_t;
pub type khronos_uint64_t = u64;
pub type khronos_ssize_t = c_long;
pub type EGLint = i32;
pub type EGLNativeDisplayType = NativeDisplayType;
pub type EGLNativePixmapType = NativePixmapType;
pub type EGLNativeWindowType = NativeWindowType;
pub type NativeDisplayType = *const c_void;
pub type NativePixmapType = *const c_void;
pub type NativeWindowType = *const c_void;

include!(concat!(env!("OUT_DIR"), "/egl_bindings.rs"));

/// nVidia support needs some implemented but only proposed egl extensions...
/// Therefor gl_generator cannot generate them and we need some constants...
/// And a function...
pub const CONSUMER_AUTO_ACQUIRE_EXT: i32 = 0x332B;
pub const DRM_FLIP_EVENT_DATA_NV: i32 = 0x333E;
pub const CONSUMER_ACQUIRE_TIMEOUT_USEC_KHR: i32 = 0x321E;
pub const RESOURCE_BUSY_EXT: u32 = 0x3353;
pub const STREAM_CONSUMER_IMAGE_NV: u32 = 0x3373;
pub const STREAM_IMAGE_ADD_NV: u32 = 0x3374;
pub const STREAM_IMAGE_REMOVE_NV: u32 = 0x3375;
pub const STREAM_IMAGE_AVAILABLE_NV: u32 = 0x3376;

#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn StreamConsumerAcquireAttribNV(
    dpy: types::EGLDisplay,
    stream: types::EGLStreamKHR,
    attrib_list: *const types::EGLAttrib,
) -> types::EGLBoolean {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::EGLDisplay,
            types::EGLStreamKHR,
            *const types::EGLAttrib,
        ) -> types::EGLBoolean,
    >(nvidia_storage::StreamConsumerAcquireAttribNV.f)(dpy, stream, attrib_list)
}

#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn StreamConsumerReleaseAttribNV(
    dpy: types::EGLDisplay,
    stream: types::EGLStreamKHR,
    attrib_list: *const types::EGLAttrib,
) -> types::EGLBoolean {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::EGLDisplay,
            types::EGLStreamKHR,
            *const types::EGLAttrib,
        ) -> types::EGLBoolean,
    >(nvidia_storage::StreamConsumerReleaseAttribNV.f)(dpy, stream, attrib_list)
}

#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn StreamImageConsumerConnectNV(
    dpy: types::EGLDisplay,
    stream: types::EGLStreamKHR,
    max_modifiers: types::EGLint,
    modifiers: *mut types::EGLuint64KHR,
    attrib_list: *const types::EGLAttrib,
) -> types::EGLBoolean {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::EGLDisplay,
            types::EGLStreamKHR,
            types::EGLint,
            *const types::EGLuint64KHR,
            *const types::EGLAttrib,
        ) -> types::EGLBoolean,
    >(nvidia_storage::StreamImageConsumerConnectNV.f)(dpy, stream, max_modifiers, modifiers, attrib_list)
}

#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn QueryStreamConsumerEventNV(
    dpy: types::EGLDisplay,
    stream: types::EGLStreamKHR,
    timeout: types::EGLTime,
    event: *mut types::EGLenum,
    aux: *mut types::EGLAttrib,
) -> types::EGLBoolean {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::EGLDisplay,
            types::EGLStreamKHR,
            types::EGLTime,
            *const types::EGLenum,
            *const types::EGLAttrib,
        ) -> types::EGLBoolean,
    >(nvidia_storage::QueryStreamConsumerEventNV.f)(dpy, stream, timeout, event, aux)
}

#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn StreamAcquireImageNV(
    dpy: types::EGLDisplay,
    stream: types::EGLStreamKHR,
    image: *mut types::EGLImage,
    sync: types::EGLSync,
) -> types::EGLBoolean {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::EGLDisplay,
            types::EGLStreamKHR,
            *const types::EGLImage,
            types::EGLSync
        ) -> types::EGLBoolean,
    >(nvidia_storage::StreamAcquireImageNV.f)(dpy, stream, image, sync)
}

#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn StreamReleaseImageNV(
    dpy: types::EGLDisplay,
    stream: types::EGLStreamKHR,
    image: *mut types::EGLImage,
    sync: types::EGLSync,
) -> types::EGLBoolean {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::EGLDisplay,
            types::EGLStreamKHR,
            *const types::EGLImage,
            types::EGLSync
        ) -> types::EGLBoolean
    >(nvidia_storage::StreamReleaseImageNV.f)(dpy, stream, image, sync)
}

mod nvidia_storage {
    use super::{FnPtr, __gl_imports::raw};
    pub static mut StreamConsumerAcquireAttribNV: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut StreamConsumerReleaseAttribNV: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut StreamImageConsumerConnectNV: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut QueryStreamConsumerEventNV: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut StreamAcquireImageNV: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut StreamReleaseImageNV: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
}

#[allow(non_snake_case)]
pub mod StreamConsumerAcquireAttribNV {
    use super::{FnPtr, __gl_imports::raw, metaloadfn, nvidia_storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { nvidia_storage::StreamConsumerAcquireAttribNV.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&str) -> *const raw::c_void,
    {
        unsafe {
            nvidia_storage::StreamConsumerAcquireAttribNV =
                FnPtr::new(metaloadfn(&mut loadfn, "eglStreamConsumerAcquireAttribNV", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod StreamConsumerReleaseAttribNV {
    use super::{FnPtr, __gl_imports::raw, metaloadfn, nvidia_storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { nvidia_storage::StreamConsumerReleaseAttribNV.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&str) -> *const raw::c_void,
    {
        unsafe {
            nvidia_storage::StreamConsumerReleaseAttribNV =
                FnPtr::new(metaloadfn(&mut loadfn, "eglStreamConsumerReleaseAttribNV", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod StreamImageConsumerConnectNV {
    use super::{FnPtr, __gl_imports::raw, metaloadfn, nvidia_storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { nvidia_storage::StreamImageConsumerConnectNV.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&str) -> *const raw::c_void,
    {
        unsafe {
            nvidia_storage::StreamImageConsumerConnectNV =
                FnPtr::new(metaloadfn(&mut loadfn, "eglStreamImageConsumerConnectNV", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod QueryStreamConsumerEventNV {
    use super::{FnPtr, __gl_imports::raw, metaloadfn, nvidia_storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { nvidia_storage::QueryStreamConsumerEventNV.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&str) -> *const raw::c_void,
    {
        unsafe {
            nvidia_storage::QueryStreamConsumerEventNV =
                FnPtr::new(metaloadfn(&mut loadfn, "eglQueryStreamConsumerEventNV", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod StreamAcquireImageNV {
    use super::{FnPtr, __gl_imports::raw, metaloadfn, nvidia_storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { nvidia_storage::StreamAcquireImageNV.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&str) -> *const raw::c_void,
    {
        unsafe {
            nvidia_storage::StreamAcquireImageNV =
                FnPtr::new(metaloadfn(&mut loadfn, "eglStreamAcquireImageNV", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod StreamReleaseImageNV {
    use super::{FnPtr, __gl_imports::raw, metaloadfn, nvidia_storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { nvidia_storage::StreamReleaseImageNV.is_loaded }
    }
    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&str) -> *const raw::c_void,
    {
        unsafe {
            nvidia_storage::StreamReleaseImageNV =
                FnPtr::new(metaloadfn(&mut loadfn, "eglStreamReleaseImageNV", &[]))
        }
    }
}

pub fn wrap_egl_call<R, F: FnOnce() -> R>(call: F) -> Result<R, EGLError> {
    let res = call();
    match unsafe { GetError() as u32 } {
        SUCCESS => Ok(res),
        x => Err(EGLError::from(x)),
    }
}