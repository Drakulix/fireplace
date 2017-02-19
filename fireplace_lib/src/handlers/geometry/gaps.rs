use handlers::UiConfig;
use handlers::geometry::UsableViewGeometry;
use handlers::store::Store;
use wlc::{Callback, View};

/// Configuration for the `GapsHandler`
#[derive(Deserialize, Clone)]
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
/// - [`OutputConfigHandler`](./struct.OutputConfigHandler.html)
/// - [`GeometryHandler`](./struct.GeometryHandler.html)
///
#[derive(Default)]
pub struct GapsHandler;

impl GapsHandler {
    /// Initialize a new `GapsHandler` with a given `GapsConfig`
    pub fn new() -> GapsHandler {
        GapsHandler
    }
}

impl Callback for GapsHandler {
    fn view_created(&mut self, view: &View) -> bool {
        if let Some(lock) = view.get::<UiConfig>() {
            let conf = &(*lock.read().unwrap()).gaps;
            if let Some(lock) = view.get::<UsableViewGeometry>() {
                let mut scissor = lock.write().unwrap();
                scissor.up += (conf.width * view.output().scale()) as usize;
                scissor.down += (conf.width * view.output().scale()) as usize;
                scissor.left += (conf.width * view.output().scale()) as usize;
                scissor.right += (conf.width * view.output().scale()) as usize;
            };
        }
        true
    }
}
