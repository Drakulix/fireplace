//! Types to render a statusbar via `conrod` on the compositor

use conrod::{Borderable, Colorable, Positionable, Sizeable, Widget};
use conrod::UiCell;
use conrod::color;
use conrod::position::Align;
use conrod::widget::Canvas;
use conrod::widget::id::Id;
use handlers::UiConfig;
use handlers::geometry::UsableScreenGeometry;
use handlers::render::conrod::ConrodRenderer;
use handlers::render::conrod::deserializer::Color;
use handlers::render::conrod::provider::ConrodProvider;

use handlers::store::Store;

use wlc::{Callback, Output, Size};

mod time;
mod workspace;
pub use self::time::{Time, TimeConfig};
pub use self::workspace::{WorkspaceIndicator, WorkspaceIndicatorConfig};

/// Handler that initializes a `Statusbar` provider for every created `Output`
///
/// ## Dependencies
///
/// - [`StoreHandler`](../../../../struct.StoreHandler.html)
/// - [`OutputConfigHandler`](../../../../struct.OutputConfigHandler.html)
/// - [`ConrodHandler`](../struct.ConrodHandler.html)
///
#[derive(Default)]
pub struct StatusbarHandler;

/// Configuration for a `StatusbarHandler` describing how the statubar shall
/// look and what it shall display
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct StatusbarConfig {
    /// Height of the statusbar
    #[serde(default = "::handlers::render::conrod::provider::statusbar::default_height")]
    pub height: u32,
    /// Background color of the statusbar
    #[serde(default = "::handlers::render::conrod::provider::statusbar::default_color")]
    pub color: Color,
    /// Location of the statusbar
    #[serde(default)]
    pub location: Location,
    /// Configuration for the `Time` `StatusbarItem` if desired
    #[serde(default)]
    pub time: Option<TimeConfig>,
    /// Configuration for the `WorkspaceIndicator` `StatusbarItem` if desired
    #[serde(default)]
    pub workspace: Option<WorkspaceIndicatorConfig>,
}

fn default_height() -> u32 {
    20
}

fn default_color() -> Color {
    Color(color::BLACK)
}

impl Default for StatusbarConfig {
    fn default() -> StatusbarConfig {
        StatusbarConfig {
            height: default_height(),
            color: default_color(),
            location: Location::default(),
            time: Some(TimeConfig::default()),
            workspace: Some(WorkspaceIndicatorConfig::default()),
        }
    }
}

/// Interface for a statusbar item, that might be rendered on the statusbar
pub trait StatusbarItem {
    /// Returns a `Canvas` with an `Id`, that shall be positioned on the
    /// statusbar
    /// according to `Justify`, so the item may be rendered onto of the `Canvas`
    /// without obstructing other `StatusbarItem`s
    fn positionable(&mut self, output: &Output, height: f64) -> (Align, Id, Canvas);
    /// Renders the item using `ui` with the given `height` on top of the
    /// previously
    /// via `positionable` returned `Canvas`
    fn render(&mut self, output: &Output, height: f64, ui: &mut UiCell);
}

/// Location of the statusbar
#[cfg_attr(rustfmt, rustfmt_skip)]
enum_str!(pub enum Location
{
    Top,
    Bottom,
});

impl Default for Location {
    fn default() -> Location {
        Location::Top
    }
}

impl StatusbarHandler {
    /// Initialize a new `StatusbarHandler`
    pub fn new() -> StatusbarHandler {
        StatusbarHandler
    }

    fn set_usable_screen_geometry(&self, output: &Output) {
        if let Some(lock) = output.get::<UiConfig>() {
            let conf = &(*lock.read().unwrap()).statusbar;
            let geometry = output.get::<UsableScreenGeometry>();
            if let Some(mut lock) = geometry.as_ref().and_then(|x| x.write().ok()) {
                lock.origin.y += if conf.location == Location::Top {
                    (conf.height * output.scale()) as i32
                } else {
                    0
                };
                lock.size.h -= conf.height * output.scale();
            };
        }
    }
}

impl Callback for StatusbarHandler {
    fn output_context_created(&mut self, output: &Output) {
        if let Some(lock) = output.get::<UiConfig>() {
            let conf = &(*lock.read().unwrap()).statusbar;
            if let Some(lock) = output.get::<ConrodRenderer>() {
                let mut ui = lock.write().unwrap();
                let ids = [ui.background.widget_id_generator().next(),
                           ui.background.widget_id_generator().next(),
                           ui.background.widget_id_generator().next(),
                           ui.background.widget_id_generator().next()];
                let statusbar = Statusbar {
                    ids: ids,
                    children: {
                        let mut children = Vec::new();
                        if let Some(ref time) = conf.time {
                            children.push(Box::new(Time::new(output, &mut ui.background, time.clone())) as
                                          Box<StatusbarItem>);
                        }
                        if let Some(ref workspace) = conf.workspace {
                            children.push(Box::new(WorkspaceIndicator::new(output,
                                                                           &mut ui.background,
                                                                           workspace.clone())) as
                                          Box<StatusbarItem>);
                        }
                        children
                    },
                };
                ui.background.register(statusbar);
            }
        }
    }

    fn output_created(&mut self, output: &Output) -> bool {
        self.set_usable_screen_geometry(output);
        true
    }

    fn output_resolution(&mut self, output: &Output, _from: Size, _to: Size) {
        self.set_usable_screen_geometry(output);
    }
}

/// A provider rendering a statusbar
pub struct Statusbar {
    ids: [Id; 4],
    children: Vec<Box<StatusbarItem>>,
}

impl ConrodProvider for Statusbar {
    fn render(&mut self, output: &Output, ui: &mut UiCell) {
        if let Some(lock) = output.get::<UiConfig>() {
            let conf = &(*lock.read().unwrap()).statusbar;
            let root = {
                let builder1 = Canvas::new();
                let builder2 = match conf.location {
                    Location::Top => builder1.top_left(),
                    Location::Bottom => builder1.bottom_left(),
                };
                builder2.h((conf.height * output.scale()) as f64)
                    .w(ui.window_dim()[0])
                    .color(*conf.color)
                    .border(0.0)
            };

            {
                let mut left = Vec::new();
                let mut center = Vec::new();
                let mut right = Vec::new();

                for mut child in &mut self.children {
                    let (alignment, id, canvas) = child.positionable(output, conf.height as f64);

                    match alignment {
                        Align::Start => left.push((id, canvas)),
                        Align::Middle => center.push((id, canvas)),
                        Align::End => right.push((id, canvas)),
                    };
                }

                root.flow_right(&[(self.ids[1],
                                   Canvas::new()
                                       .h(conf.height as f64)
                                       .color(color::TRANSPARENT)
                                       .border(0.0)
                                       .flow_right(&left)),
                                  (self.ids[2],
                                   Canvas::new()
                                       .h(conf.height as f64)
                                       .color(color::TRANSPARENT)
                                       .border(0.0)
                                       .flow_left(&center)),
                                  (self.ids[3],
                                   Canvas::new()
                                       .h(conf.height as f64)
                                       .color(color::TRANSPARENT)
                                       .border(0.0)
                                       .flow_left(&right))])
                    .set(self.ids[0], ui);
            }

            for mut child in &mut self.children {
                child.render(output, conf.height as f64, ui);
            }
        }
    }
}
