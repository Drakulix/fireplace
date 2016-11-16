//! Partial implementations of the [`Callback`](../wlc/trait.Callback.html)
//! trait for easy composability
//!
//! Using the nightly `specialization` feature of Rust the structs of this
//! module
//! allow overriding their default implementations of the `Callback` trait.
//!
//! Documentation on this subject is very thin, as it is both:
//!
//! * a currently *nightly* feature with some remaining api/soundness holes
//! * a relatively *new* nightly feature.
//!
//!
//! The only official documentation so far is this
//! [RFC](https://github.
//! com/rust-lang/rfcs/blob/master/text/1210-impl-specialization.md)
//!
//! Examples on how to use this feature can be found in the struct specific
//! documentation
//! and handler source code.

use wlc::Callback;

mod wrapper;
mod split;
mod vec;

pub use self::split::*;
pub use self::vec::*;
pub use self::wrapper::*;

/// A trait to express the ability to consume an object and acquire something
/// that implements `Callback`
pub trait IntoCallback<C: Callback> {
    /// Converts into a `Callback` implementation
    fn into_callback(self) -> C;
}

impl<C: Callback> IntoCallback<C> for C {
    fn into_callback(self) -> C {
        self
    }
}
