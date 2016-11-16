//! Types to render a statusbar via `conrod` on the compositor

use conrod::{Borderable, Colorable, Positionable, Sizeable, Widget};
use conrod::UiCell;
use conrod::color::{self, Color as ConrodColor};
use conrod::position::Align;
use conrod::widget::Canvas;
use conrod::widget::id::Id;
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
/// - [`StoreHandler`](./struct.StoreHandler.html)
/// - [`ConrodHandler`](../struct.ConrodHandler.html)
///
pub struct StatusbarHandler {
    height: u32,
    color: ConrodColor,
    location: Location,
    time: Option<TimeConfig>,
    workspace: Option<WorkspaceIndicatorConfig>,
}

/// Configuration for a `StatusbarHandler` describing how the statubar shall
/// look
/// and what it shall display
#[derive(Deserialize)]
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
    Buttom,
});

impl Default for Location {
    fn default() -> Location {
        Location::Top
    }
}

impl StatusbarHandler {
    /// Initialize a new `StatusbarHandler`
    pub fn new(config: StatusbarConfig) -> StatusbarHandler {
        StatusbarHandler {
            height: config.height,
            color: *config.color,
            location: config.location,
            time: config.time,
            workspace: config.workspace,
        }
    }

    fn set_usable_screen_geometry(&self, output: &Output) {
        let geometry = output.get::<UsableScreenGeometry>();
        if let Some(mut lock) = geometry.as_ref().and_then(|x| x.write().ok()) {
            lock.origin.y += if self.location == Location::Top {
                (self.height * output.scale()) as i32
            } else {
                0
            };
            lock.size.h -= self.height * output.scale();
        };
    }
}

impl Callback for StatusbarHandler {
    fn output_context_created(&mut self, output: &Output) {
        if let Some(lock) = output.get::<ConrodRenderer>() {
            let mut ui = lock.write().unwrap();
            let ids = [ui.background.widget_id_generator().next(),
                       ui.background.widget_id_generator().next(),
                       ui.background.widget_id_generator().next(),
                       ui.background.widget_id_generator().next()];
            let statusbar = Statusbar {
                ids: ids,
                height: (self.height * output.scale()) as f64,
                color: self.color,
                location: self.location,
                children: {
                    let mut children = Vec::new();
                    if let Some(ref time) = self.time {
                        children.push(Box::new(Time::new(output, &mut ui.background, time.clone())) as
                                      Box<StatusbarItem>);
                    }
                    if let Some(ref workspace) = self.workspace {
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
    height: f64,
    color: ConrodColor,
    location: Location,
    children: Vec<Box<StatusbarItem>>,
}

impl ConrodProvider for Statusbar {
    fn render(&mut self, output: &Output, ui: &mut UiCell) {
        let root = {
            let builder1 = Canvas::new();
            let builder2 = match self.location {
                Location::Top => builder1.top_left(),
                Location::Buttom => builder1.bottom_left(),
            };
            builder2.h(self.height * output.scale() as f64)
                .w(ui.window_dim()[0])
                .color(self.color)
                .border(0.0)
        };

        {
            let mut left = Vec::new();
            let mut center = Vec::new();
            let mut right = Vec::new();

            for mut child in &mut self.children {
                let (alignment, id, canvas) = child.positionable(output, self.height);

                match alignment {
                    Align::Start => left.push((id, canvas)),
                    Align::Middle => center.push((id, canvas)),
                    Align::End => right.push((id, canvas)),
                };
            }

            root.flow_right(&[(self.ids[1],
                               Canvas::new()
                                   .h(self.height)
                                   .color(color::TRANSPARENT)
                                   .border(0.0)
                                   .flow_right(&left)),
                              (self.ids[2],
                               Canvas::new()
                                   .h(self.height)
                                   .color(color::TRANSPARENT)
                                   .border(0.0)
                                   .flow_left(&center)),
                              (self.ids[3],
                               Canvas::new()
                                   .h(self.height)
                                   .color(color::TRANSPARENT)
                                   .border(0.0)
                                   .flow_left(&right))])
                .set(self.ids[0], ui);
        }

        for mut child in &mut self.children {
            child.render(output, self.height, ui);
        }
    }
}
