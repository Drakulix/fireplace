//! # Fireplace Window Manager Library
//!
//! Fireplace is a modular wayland window manager.
//! This library can be used to build your own window manager by
//!
//! * adding additional functionality to fireplace
//! * replacing functionality of fireplace
//! * or just build upon the concepts of fireplace
//!
//! As a starting point you might want to fork the [`fireplace`](../fireplace)
//! binary
//! to get a functional window manager and use this library to implement your
//! changes.
//! By keeping your changes in the binary or even additional libraries you can
//! make
//! your changes/additions easier accessible to a broder audicence and your
//! fork does
//! not need to maintain a whole window manager with most of the functionality
//! still
//! kept in the unmodified `fireplace_lib` library.
//!
//! For a more low level wayland Rust Library take a look at:
//!
//! * [wlc](https://github.com/Drakulix/wlc) (used by fireplace)
//! * or even
//! [wayland-server](https://github.
//! com/vberger/wayland-client-rs/tree/master/wayland-server)
//!
//! # Concepts
//!
//! ## The `Callback` trait
//! To understand why fireplace was build the way it was, you need to
//! understand basics
//! of the underlying library [wlc](../wlc/):
//!
//! * The [`Callback`](../wlc/trait.Callback.html) Trait
//! * The [`View`](../wlc/struct.View.html) Struct
//! * The [`Output`](../wlc/struct.Output.html) Struct
//!
//! ### [`Output`](../wlc/struct.Output.html)
//! An `Output` represents a Display or the X Window when running Fireplace
//! nested in X.
//! In future there may be even more options. An Output is basically everthing
//! `wlc`
//! and therefor fireplace may render on.
//!
//! ### [`View`](../wlc/struct.View.html)
//! A `View` represents an Application Window in the most basic way.
//! Everything rendered by a wayland client is represented as a `View`.
//!
//! ### [`Callback`](../wlc/struct.Callback.html)
//! Any object implementing the `Callback` trait can be used by `wlc` to drive
//! the compositor.
//! It includes many optionally handled functions that represents events
//! changing the state
//! of the compositor.
//!
//! Some examples might be:
//!
//! * [`output_created`](../wlc/trait.Callback.html#method.output_created)
//! * [`output_destroyed`](../wlc/trait.Callback.html#method.output_destroyed)
//! * [`view_created`](../wlc/trait.Callback.html#method.view_created)
//! * [`view_destroyed`](../wlc/trait.Callback.html#method.view_destroyed)
//! * [`pointer_motion`](../wlc/trait.Callback.html#method.pointer_motion)
//! * [`keyboard_key`](../wlc/trait.Callback.html#method.keyboard_key)
//!
//! In `wlc` you may only use one callback to drive the compositor.
//! Fireplace breaks this restriction by providing `Callback` implementations
//! that might
//! split the calls onto two or even more other `Callback`s and implementations
//! that may
//! wrap existing `Callback`s modifing parameters, injecting calls or blocking
//! calls
//! however you want.
//!
//! Using this functionality every complex process might be split into multiple
//! self-contained so called `handlers` that can be recombined to create new
//! functionality
//! and can be composed in the end to provide a fully functional window manager.
//!
//! ## Handlers
//!
//! All `handlers` implement the `Callback` trait in some way:
//!
//! * directly
//! * by implementing [`AsWrapper`](./callback/trait.AsWrapper.html) - Wrapping
//! existing handlers
//! * by implementing [`AsSplit`](./callback/trait.AsSplit.html) - Splitting
//! calls between up to two handlers
//! * by implementing [`AsVec`](./callback/trait.AsVec.html) - Splitting calls
//! onto an arbitrary number of handlers
//!
//! All these traits can be found in the [`callback`](./callback/index.html)
//! module
//!
//! All handlers provided and used by fireplace and a more specific
//! introduction can be found
//! in the [`handlers`](./handlers/) module.
//!

#![cfg_attr(feature = "cargo-clippy", deny(clippy))]
#![cfg_attr(feature = "cargo-clippy", allow(let_and_return))]
#![feature(specialization)]
#![deny(missing_docs)]

extern crate wlc;
extern crate anymap;
extern crate typemap;
extern crate linked_hash_map;
extern crate id_tree;

extern crate serde;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate slog;
extern crate slog_scope;

#[cfg(feature = "conrod_ui")]
extern crate conrod;
#[cfg(feature = "render")]
extern crate image;
#[cfg(feature = "conrod_ui")]
extern crate texture;
#[cfg(feature = "render")]
extern crate chrono;
#[cfg(feature = "conrod_ui")]
extern crate font_loader;
#[cfg(feature = "graphics")]
extern crate graphics;
#[cfg(feature = "gl")]
extern crate opengles_graphics;
#[cfg(feature = "gl")]
extern crate egli;

#[macro_use]
pub mod utils;
pub mod callback;
pub mod handlers;
