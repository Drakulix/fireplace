use conrod::{Borderable, Colorable, Positionable, Sizeable, UiCell, Widget};
use conrod::color::{self, Color as ConrodColor};
use conrod::position::Align as ConrodAlign;
use conrod::text::{Font, FontCollection, Justify, height as text_height};
use conrod::text::font::Id as FontId;
use conrod::text::line::width as text_width;
use conrod::widget::Canvas;
use conrod::widget::id::Id;
use conrod::widget::primitive::text::Text;

use font_loader::system_fonts::get as font_get;
use handlers::render::conrod::ConrodInstance;
use handlers::render::conrod::deserializer::{Align, Color, Font as ConfigFont};
use handlers::render::conrod::provider::statusbar::StatusbarItem;

use handlers::store::Store;
use handlers::workspaces::ActiveWorkspace;

use wlc::Output;

/// `StatusbarItem` for displaying the currently active workspace
pub struct WorkspaceIndicator {
    ids: [Id; 2],
    font_id: FontId,
    font: Font,
    alignment: ConrodAlign,
    margin: u32,
    color: ConrodColor,
}

/// Configuration for `WorkspaceIndicator`
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceIndicatorConfig {
    /// Font to be used for the rendered workspace number and name
    #[serde(default = "::handlers::render::conrod::provider::statusbar::workspace::default_font")]
    pub font: ConfigFont,
    /// Alignment of the indicator on the statusbar
    #[serde(default = "::handlers::render::conrod::provider::statusbar::workspace::default_alignment")]
    pub alignment: Align,
    /// Text color of the rendered workspace number and name
    #[serde(default = "::handlers::render::conrod::provider::statusbar::workspace::default_color")]
    pub color: Color,
    /// Margin to other `StatusbarItem`s
    #[serde(default = "::handlers::render::conrod::provider::statusbar::workspace::default_margin")]
    pub margin: u32,
}

impl Default for WorkspaceIndicatorConfig {
    fn default() -> WorkspaceIndicatorConfig {
        WorkspaceIndicatorConfig {
            font: default_font(),
            alignment: default_alignment(),
            color: default_color(),
            margin: default_margin(),
        }
    }
}

fn default_font() -> ConfigFont {
    ConfigFont {
        family: None,
        monospace: true,
        italic: false,
        oblique: false,
        bold: false,
    }
}

fn default_alignment() -> Align {
    Align(ConrodAlign::Start)
}

fn default_color() -> Color {
    Color(color::WHITE)
}

fn default_margin() -> u32 {
    4
}

impl WorkspaceIndicator {
    /// Initialize a new `Workspace` indicator, usually done by a
    /// `StatusbarHandler`
    pub fn new(_output: &Output, ui: &mut ConrodInstance, arguments: WorkspaceIndicatorConfig) -> Self {
        let (bytes, index) = font_get(&arguments.font.property()).expect("No font could be loaded");

        let font = FontCollection::from_bytes(bytes)
            .into_fonts()
            .nth(index as usize)
            .unwrap();

        let font_id = ui.fonts.insert(font.clone());

        WorkspaceIndicator {
            ids: [ui.widget_id_generator().next(),
                  ui.widget_id_generator().next()],
            font_id: font_id,
            font: font,
            alignment: *arguments.alignment,
            color: *arguments.color,
            margin: arguments.margin,
        }
    }
}

impl StatusbarItem for WorkspaceIndicator {
    fn positionable(&mut self, output: &Output, height: f64) -> (ConrodAlign, Id, Canvas) {
        let text = {
            let lock = output.get::<ActiveWorkspace>();
            let result = if let Some(active) = lock.as_ref().and_then(|x| x.read().ok()) {
                format!("{} {}", active.num(), active.name())
            } else {
                String::new()
            };
            result
        };

        let mut size = 2;
        while text_height(1, size, 0.0) <
              ((height as u32 * output.scale()) - (self.margin * 2 * output.scale())) as f64 {
            size += 1;
        }
        size -= 1;

        let text_width = text_width(&text, &self.font, size);

        let canvas = Canvas::new()
            .h(height * output.scale() as f64)
            .length(text_width + (self.margin * 2 * output.scale()) as f64)
            .color(color::TRANSPARENT)
            .border(0.0);

        (self.alignment, self.ids[0], canvas)
    }

    fn render(&mut self, output: &Output, height: f64, ui: &mut UiCell) {
        let text = {
            let lock = output.get::<ActiveWorkspace>();
            let result = if let Some(active) = lock.as_ref().and_then(|x| x.read().ok()) {
                format!("{} {}", active.num(), active.name())
            } else {
                String::new()
            };
            result
        };

        let mut size = 2;
        while text_height(1, size, 0.0) <
              ((height as u32 * output.scale()) - (self.margin * 2 * output.scale())) as f64 {
            size += 1;
        }
        size -= 1;

        let text_height = text_height(1, size, 0.0);
        let text_width = text_width(&text, &self.font, size);

        let builder = Text::new(&text)
            .font_id(self.font_id)
            .justify(match self.alignment {
                         ConrodAlign::Start => Justify::Left,
                         ConrodAlign::Middle => Justify::Center,
                         ConrodAlign::End => Justify::Right,
                     })
            .font_size(size)
            .h(text_height)
            .w(text_width)
            .color(self.color);

        match self.alignment {
                ConrodAlign::Start => {
                    builder.mid_left_with_margin_on(self.ids[0], (self.margin * output.scale()) as f64)
                }
                ConrodAlign::Middle => builder.middle_of(self.ids[0]),
                ConrodAlign::End => {
                    builder.mid_right_with_margin_on(self.ids[0], (self.margin * output.scale()) as f64)
                }
            }
            .set(self.ids[1], ui);
    }
}
