//! # Handlers
//!
//! Handlers provide the basic building blocks composed by
//! the compositor to provide it's full functionality.
//!
//!
//! ## Dependencies
//!
//! Handlers might expect other not directly related `handlers` to be
//! active as well. Because handlers are able to indirectly exchange data
//! through the `Store` (see below), handlers might expect certain data
//! to be already available from previously run handlers for the same
//! event.
//!
//! Please take note of the circumstances when designing and combining
//! handlers, or when writing your main module directly in code instead
//! of using a configuration file.
//!
//! All handlers list their dependencies (if any) in their documentation.
//!
//!
//! ## Notable handlers
//!
//!
//! ### Store
//!
//! Handlers may exchange data or track `View` and `Output` specific
//! state though the [`StoreHandler`](./store/struct.StoreHandler.html).
//!
//! When studing the source code of most other handlers understanding how
//! the "Store" may be used is very important.
//!
//!
//! ### Workspaces
//!
//! The [`WorkspaceHandler`](./workspaces/struct.WorkspaceHandler.html) is
//! easily the most complex proxy of fireplace, directing all events to
//! related workspaces so they appear to have access to their own `Output`.
//!
//! Essentially being an `Output` multiplexer.
//!
//! Handling of individual `Views` is then handled by `Modes`, like
//!  `Floating`, `Fullscreen` or `BSP` (i3-like **B**inary **S**plit
//!  **P**artitioning).
//!
//!
//! ### Render - (might be deactivated by individual features)
//!
//! The handlers related to the `render` module enable OpenGL ES drawing
//! and even more high-level drawing engines like `conrod` directly
//! within the compositor to offer high-performance and/or easy-to-use
//! drawing capabilities.
//!

mod store;
pub use self::store::*;

mod focus;
pub use self::focus::*;

pub mod keyboard;
pub use self::keyboard::KeyboardHandler;

mod pointer;
pub use self::pointer::*;

pub mod workspaces;
pub use self::workspaces::{WorkspaceHandler, WorkspacesConfig};

pub mod geometry;

mod output;
pub use self::output::*;

#[cfg(feature = "render")]
pub mod render;
