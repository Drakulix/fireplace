//! Handler and types related to taking screenshots.
//!

use chrono::Local;
use handlers::keyboard::KeyPattern;

use handlers::store::{Store, StoreKey};
use image::{DynamicImage, ImageFormat, RgbaImage};
use slog_scope;
use std::fs::{self, File};

use std::path::PathBuf;
use std::process::{Command, Stdio};
use wlc::{Callback, Geometry, Key, KeyState, Modifiers, Output, Point, View};
use wlc::render::{GLES2PixelFormat, RenderInstance, RenderOutput, Renderer};

/// Key for receiving the current list of queued screenshot
/// though each `Output`s [`Store`](../trait.Store.html).
///
/// Meant to be write only. Only append screenshots you want
/// to have taken when the next rendering happens of parts
/// of the specified `Output`.
pub struct QueuedScreenshots;
impl StoreKey for QueuedScreenshots {
    type Value = Vec<Geometry>;
}

/// Handler that initializes `QueuedScreenshots` per `Output`
/// and automatically calls `make_screenshot` for every queued
/// screenshot on rendering and automatically saves them as PNG
/// files in the users pictures folder.
///
/// ## Dependencies
///
/// - [`StoreHandler`](../struct.StoreHandler.html)
///
/// ### Optional - but must be loaded before to be rendered on the screenshots
///
/// - [`GraphicsRenderer`](./struct.GraphicsRenderer.html)
/// - [`ConrodHandler`](./conrod/struct.ConrodHandler.html)
///
#[derive(Default)]
pub struct ScreenshotHandler {
    config: ScreenshotConfig,
}

impl ScreenshotHandler {
    /// Initialize a new `ScreenshotHandler`
    pub fn new(config: ScreenshotConfig) -> ScreenshotHandler {
        ScreenshotHandler { config: config }
    }
}

/// Configuration for `ScreenshotHandler`
#[derive(Default, Clone, PartialEq, Eq, Hash, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ScreenshotConfig {
    /// Location for saved screenshots
    #[serde(default = "::handlers::render::screenshot::default_path")]
    location: PathBuf,
    /// Key actions for saving screenshots
    #[serde(default)]
    keys: KeyPatterns,
}

fn default_path() -> PathBuf {
    let child = Command::new("xdg-user-dir")
        .arg("DESKTOP")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute xdg-user-dir. Could not find default path for screenshots");
    let output = child.wait_with_output()
        .expect("xdg-user-dir did terminate in an unusual way");
    PathBuf::from(String::from_utf8_lossy(&*output.stdout).into_owned().trim())
}

/// `KeyPattern`s toggling fullscreen
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Deserialize)]
pub struct KeyPatterns {
    /// Make screenshot of the currently focused `Output`
    #[serde(default)]
    pub output: Option<KeyPattern>,
    /// Make screenshot of the currently focused `View`
    #[serde(default)]
    pub view: Option<KeyPattern>,
}

impl Callback for ScreenshotHandler {
    fn output_created(&mut self, output: &Output) -> bool {
        output.insert::<QueuedScreenshots>(Vec::new());
        true
    }

    fn output_render_post(&mut self, output: &mut RenderOutput) {
        if let Some(queued) = output.get::<QueuedScreenshots>() {
            let mut lock = queued.write().unwrap();
            for mut geometry in lock.drain(..) {
                if let Some(image) = make_screenshot(output, &mut geometry) {
                    let filename = Local::now().format("screenshot_%Y-%m-%dT%H:%M:%S%.f%:z.png");

                    let mut path = self.config.location.clone();
                    fs::create_dir_all(path.clone()).expect("Could not create screenshots folder");

                    path.push(format!("{}", filename));
                    image.save(&mut File::create(path).expect("Failed to create screenshot file"),
                              ImageFormat::PNG)
                        .expect("Failed to encode screenshot");
                }
            }
        }
    }

    fn keyboard_key(&mut self, view: Option<&View>, _time: u32, modifiers: Modifiers, key: Key,
                    state: KeyState)
                    -> bool {
        match Some(KeyPattern::new(state, modifiers.mods, key)) {
            x if x == self.config.keys.output => {
                Output::with_focused_output(|output| {
                    let delayed = output.get::<QueuedScreenshots>().unwrap();
                    let mut lock = delayed.write().unwrap();
                    lock.push(Geometry {
                        origin: Point { x: 0, y: 0 },
                        size: output.resolution(),
                    });
                });
                true
            }
            x if x == self.config.keys.view => {
                if let Some(view) = view {
                    let output = view.output();
                    let queued = output.get::<QueuedScreenshots>().unwrap();
                    let mut lock = queued.write().unwrap();
                    lock.push(view.visible_geometry());
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

/// Create a screenshot of a given `Geometry` inside a given `RenderOutput`.
///
/// Because you need a `RenderOutput`, you may only take screenshots in the
/// `*_render_pre` and `*_render_post` hooks of the `Callback` trait.
///
/// During `*_render_pre` no `View`s will have been drawn and only handlers
/// queued before will be visible. `view_render_*` functions will be during
/// `View` rendering and will most likely be incomplete and should only be
/// used to screenshot that specific `View`. `output_render_post` is the
/// function, that should be used for complete screenshot rendering. Care
/// should be taken, that every other handler's rendering has been done before.
///
/// For an easier alternative not requiring and `RenderOutput`, see the
/// `ScreenshotHandler`
pub fn make_screenshot(output: &mut RenderOutput, geo: &mut Geometry) -> Option<DynamicImage> {
    match output.get_renderer() {
        RenderInstance::GLES2(renderer) => {
            let pixels = renderer.pixels_read(GLES2PixelFormat::RGBA8888, geo);
            let image = RgbaImage::from_raw(geo.size.w, geo.size.h, pixels).expect("Invalid sizes");
            Some(DynamicImage::ImageRgba8(image).flipv())
        }
        _ => {
            error!(slog_scope::logger(), "Unsupported renderer for screenshots");
            None
        }
    }
}
