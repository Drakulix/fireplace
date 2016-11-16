use handlers::geometry::UsableViewGeometry;
use handlers::store::Store;
use wlc::{Callback, View};

/// Configuration for the `GapsHandler`
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GapsConfig {
    /// Width in screen points of the gaps created by the handler
    #[serde(default = "::handlers::geometry::gaps::default_width")]
    pub width: u32,
}

impl Default for GapsConfig {
    fn default() -> GapsConfig {
        GapsConfig { width: default_width() }
    }
}

fn default_width() -> u32 {
    2
}

/// Handler to make some gaps around `View`s
///
/// ## Dependencies
///
/// - [`StoreHandler`](../struct.StoreHandler.html)
/// - [`GeometryHandler`](./struct.GeometryHandler.html)
///
#[derive(Default)]
pub struct GapsHandler {
    width: u32,
}

impl GapsHandler {
    /// Initialize a new `GapsHandler` with a given `GapsConfig`
    pub fn new(config: GapsConfig) -> GapsHandler {
        GapsHandler { width: config.width }
    }
}

impl Callback for GapsHandler {
    fn view_created(&mut self, view: &View) -> bool {
        if let Some(lock) = view.get::<UsableViewGeometry>() {
            let mut scissor = lock.write().unwrap();
            scissor.up += (self.width * view.output().scale()) as usize;
            scissor.down += (self.width * view.output().scale()) as usize;
            scissor.left += (self.width * view.output().scale()) as usize;
            scissor.right += (self.width * view.output().scale()) as usize;
        };
        true
    }
}
