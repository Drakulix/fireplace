extern crate gl_generator;
extern crate wayland_scanner;

use gl_generator::{Api, Fallbacks, Profile, Registry};
use wayland_scanner::{Side, generate_code};
use std::{
    env,
    fs::File,
    process::Command,
    path::PathBuf,
};

fn main() {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());
    let mut file = File::create(&dest.join("egl_bindings.rs")).unwrap();
    Registry::new(
        Api::Egl,
        (1, 5),
        Profile::Core,
        Fallbacks::All,
        [
            "EGL_KHR_create_context",
            "EGL_EXT_create_context_robustness",
            "EGL_KHR_create_context_no_error",
            "EGL_MESA_platform_gbm",
            "EGL_WL_bind_wayland_display",
            "EGL_KHR_image_base",
            "EGL_EXT_image_dma_buf_import",
            "EGL_EXT_image_dma_buf_import_modifiers",
            "EGL_MESA_image_dma_buf_export",
            "EGL_EXT_platform_base",
            "EGL_EXT_platform_device",
            "EGL_EXT_output_base",
            "EGL_EXT_output_drm",
            "EGL_EXT_device_drm",
            "EGL_EXT_device_enumeration",
            "EGL_EXT_device_query",
            "EGL_KHR_stream",
            "EGL_KHR_stream_cross_process_fd",
            "EGL_NV_stream_consumer_eglimage",
            "EGL_KHR_stream_producer_eglsurface",
            "EGL_EXT_stream_consumer_egloutput",
            "EGL_EXT_stream_acquire_mode",
            "EGL_KHR_stream_fifo",
            "EGL_NV_output_drm_flip_event",
            "EGL_NV_stream_attrib",
        ],
    )
    .write_bindings(gl_generator::GlobalGenerator, &mut file)
    .unwrap();

    // Location of the xml file, relative to the `Cargo.toml`
    let drm_protocol_file = "resources/wayland-drm.xml";
    let eglstream_protocol_file = "resources/wayland-eglstream.xml";
    let eglstream_controller_protocol_file = "resources/wayland-eglstream-controller.xml";

    // Target directory for the generate files
    generate_code(
        drm_protocol_file,
        &dest.join("wl_drm.rs"),
        Side::Server,
    );
    generate_code(
        eglstream_protocol_file,
        &dest.join("wl_eglstream.rs"),
        Side::Server,
    );
    generate_code(
        eglstream_controller_protocol_file,
        &dest.join("wl_eglstream_controller.rs"),
        Side::Server,
    );
}
