use slog;
use slog_scope;

use std::any::Any;
use std::rc::Rc;
use std::sync::RwLock;
use typemap::{Key as TypeMapKey, TypeMap};
use wlc::{Callback, Handle, Output, Size, View};

/// Handler to initialize a `Store` for each `View` and `Output`
///
/// Must be initialized before every other struct that needs to use the `Store`.
///
/// ## Dependencies
///
/// This handler has no dependencies.
///
pub struct StoreHandler {
    logger: slog::Logger,
}

impl Default for StoreHandler {
    fn default() -> StoreHandler {
        StoreHandler::new()
    }
}

impl StoreHandler {
    /// Initialize a new `StoreHandler`
    pub fn new() -> StoreHandler {
        StoreHandler { logger: slog_scope::logger().new(o!("handler" => "Store")) }
    }
}

impl Callback for StoreHandler {
    fn output_resolution(&mut self, output: &Output, _from: Size, _to: Size) {
        self.output_created(output); // Issue #207 in wlc
    }

    fn output_created(&mut self, output: &Output) -> bool {
        let is_initialized = unsafe { output.user_data::<RwLock<TypeMap>>().is_some() };
        if !is_initialized {
            debug!(self.logger, "Output initialized");
            let data = RwLock::new(TypeMap::new());
            output.set_user_data(data);
        }
        true
    }

    fn view_created(&mut self, view: &View) -> bool {
        let is_initialized = unsafe { view.user_data::<RwLock<TypeMap>>().is_some() };
        if !is_initialized {
            debug!(self.logger, "View initialized");
            let data = RwLock::new(TypeMap::new());
            view.set_user_data(data);
        }
        true
    }
}

struct KeyWrapper<K: StoreKey>(K);

/// Implement to store a certain type in the `Store`.
pub trait StoreKey: Any {
    /// `Value` is the type to be stored, while the type of the
    /// key may be used to add, get and remove the `Value` type from the
    /// `Store`;
    type Value: Any;
}

impl<K: StoreKey> TypeMapKey for KeyWrapper<K> {
    type Value = Rc<RwLock<K::Value>>;
}

/// Implemented by any `View` and `Output` to store arbitrary data.
///
/// Use structs implementing `StoreKey` to store any value.
/// Be careful when nesting functions, as `get` returns a `RwLock`.
pub trait Store {
    /// Insert a value into the `Store`.
    ///
    /// Returns any old value that was previously stored under the same key if
    /// any.
    fn insert<T: StoreKey + 'static>(&self, value: T::Value) -> Option<T::Value>;
    /// Check if the `Store` holds any value for the key `T`.
    fn contains<T: StoreKey + 'static>(&self) -> bool;
    /// Get a reference-counted and locked reference to the stored value
    ///
    /// Be careful when nesting functions calls or handlers
    /// and make sure you don't deadlock.
    fn get<T: StoreKey + 'static>(&self) -> Option<Rc<RwLock<T::Value>>>;
    /// Try to remove and receive the currently stored value
    ///
    /// Returns `None` if any reference or lock is still hold on the value
    /// or if no value is stored in the first place.
    fn remove<T: StoreKey + 'static>(&self) -> Option<T::Value>;
}

impl<T: Handle> Store for T {
    fn insert<A: StoreKey + 'static>(&self, value: A::Value) -> Option<A::Value> {
        if let Some(x) = unsafe { self.user_data::<RwLock<TypeMap>>() } {
            if let Ok(mut x) = x.write() {
                if let Some(x) = x.insert::<KeyWrapper<A>>(Rc::new(RwLock::new(value))) {
                    return Rc::try_unwrap(x).ok().and_then(|x| x.into_inner().ok());
                }
            }
        }
        None
    }

    fn contains<A: StoreKey + 'static>(&self) -> bool {
        match unsafe { self.user_data::<RwLock<TypeMap>>() }.map(|x| match x.read().ok() {
                                                                     Some(x) => x.contains::<KeyWrapper<A>>(),
                                                                     None => false,
                                                                 }) {
            Some(x) => x,
            None => false,
        }
    }

    fn get<A: StoreKey + 'static>(&self) -> Option<Rc<RwLock<A::Value>>> {
        if let Some(x) = unsafe { self.user_data::<RwLock<TypeMap>>() } {
            if let Ok(x) = x.read() {
                if let Some(x) = x.get::<KeyWrapper<A>>() {
                    return Some(x.clone());
                }
            }
        }
        None
    }

    fn remove<A: StoreKey + 'static>(&self) -> Option<A::Value> {
        if let Some(x) = unsafe { self.user_data::<RwLock<TypeMap>>() } {
            if let Ok(mut x) = x.write() {
                if let Some(x) = x.remove::<KeyWrapper<A>>() {
                    return Rc::try_unwrap(x).ok().and_then(|x| x.into_inner().ok());
                }
            }
        }
        None
    }
}
