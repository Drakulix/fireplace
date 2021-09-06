// Re-export only the actual code, and then only use this re-export
// The `generated` module below is just some boilerplate to properly isolate stuff
// and avoid exposing internal details.
//
// You can use all the types from my_protocol as if they went from `wayland_client::protocol`.
pub use generated::server::wl_eglstream_controller;

mod generated {
    // The generated code tends to trigger a lot of warnings
    // so we isolate it into a very permissive module
    #![allow(dead_code,non_camel_case_types,unused_unsafe,unused_variables)]
    #![allow(non_upper_case_globals,non_snake_case,unused_imports)]

    pub mod server {
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
}