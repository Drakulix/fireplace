use chrono::Local;

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
use std::time::Duration;

use wlc::{Output, WeakOutput};
use wlc::event_loop::{self, Timer, TimerCallback};

/// `StatusbarItem` for displaying the current time
pub struct Time {
    ids: [Id; 2],
    font_id: FontId,
    font: Font,
    format: String,
    alignment: ConrodAlign,
    margin: u32,
    color: ConrodColor,
    timer: Timer,
}

/// Configuration for `Time`
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TimeConfig {
    /// Font to be used for the rendered time
    #[serde(default = "::handlers::render::conrod::provider::statusbar::time::default_font")]
    pub font: ConfigFont,
    /// Format of the rendered time. See
    /// [chrono](https://lifthrasiir.github.
    /// io/rust-chrono/chrono/format/strftime/index.html)
    #[serde(default = "::handlers::render::conrod::provider::statusbar::time::default_format")]
    pub format: String,
    /// Alignment of the time on the statusbar
    #[serde(default = "::handlers::render::conrod::provider::statusbar::time::default_alignment")]
    pub alignment: Align,
    /// Text color of the rendered time
    #[serde(default = "::handlers::render::conrod::provider::statusbar::time::default_color")]
    pub color: Color,
    /// Margin to other `StatusbarItem`s
    #[serde(default = "::handlers::render::conrod::provider::statusbar::time::default_margin")]
    pub margin: u32,
}

impl Default for TimeConfig {
    fn default() -> TimeConfig {
        TimeConfig {
            font: default_font(),
            format: default_format(),
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

fn default_format() -> String {
    String::from("%a, %d-%m-%Y  %T")
}

fn default_alignment() -> Align {
    Align(ConrodAlign::End)
}

fn default_color() -> Color {
    Color(color::WHITE)
}

fn default_margin() -> u32 {
    4
}

impl Time {
    /// Initialize a new `Workspace` indicator, usually done by a
    /// `StatusbarHandler`
    pub fn new(output: &Output, ui: &mut ConrodInstance, arguments: TimeConfig) -> Self {
        let (bytes, index) = font_get(&arguments.font.property()).expect("No font could be loaded");

        let font = FontCollection::from_bytes(bytes).into_fonts().nth(index as usize).unwrap();

        let font_id = ui.fonts.insert(font.clone());

        struct Rerender {
            output: WeakOutput,
        }
        impl TimerCallback for Rerender {
            fn fire(&mut self) {
                self.output.run(|output| output.schedule_render());
            }
        }

        let timer = event_loop::event_loop_add_timer(Rerender { output: output.weak_reference() });

        Time {
            ids: [ui.widget_id_generator().next(), ui.widget_id_generator().next()],
            font_id: font_id,
            font: font,
            format: arguments.format,
            alignment: *arguments.alignment,
            color: *arguments.color,
            margin: arguments.margin,
            timer: timer,
        }
    }
}

impl StatusbarItem for Time {
    fn positionable(&mut self, output: &Output, height: f64) -> (ConrodAlign, Id, Canvas) {
        self.timer.update(&Duration::from_millis(500));
        let text = format!("{}", Local::now().format(&self.format));

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
        let text = format!("{}", Local::now().format(&self.format));

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
