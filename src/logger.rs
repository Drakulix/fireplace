//! Compositor Logging Configuration

use slog::Drain;
use serde::Deserialize;

/// Configuration for fireplace's Logger
#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Logging {
    #[serde(default)]
    /// Style of the terminal logging
    pub style: Mode,
    #[serde(default)]
    /// Enabling of colored terminal output
    pub color: Color,
}

/// Terminal color output options
#[derive(Deserialize, Debug)]
pub enum Color
{
    Auto,
    Always,
    Never,
}

impl Default for Color {
    fn default() -> Color {
        Color::Auto
    }
}

/// Style of the logging output
#[derive(Deserialize, Debug)]
pub enum Mode
{
    Compact,
    Full,
}

impl Default for Mode {
    fn default() -> Mode {
        Mode::Compact
    }
}

/// Initialize fireplace's logging system
pub fn init(config: &Logging) -> slog_scope::GlobalLoggerGuard {
    let builder = slog_term::TermDecorator::new().stderr();
    let decorator = match config.color {
        Color::Always => builder.force_color(),
        Color::Never => builder.force_plain(),
        Color::Auto => builder,
    }.build();

    let params = slog::o!();
    let logger = match config.style {
        Mode::Compact => slog::Logger::root(slog_async::Async::new(slog_term::CompactFormat::new(decorator).build().ignore_res()).build().fuse(), params),
        Mode::Full => slog::Logger::root(slog_async::Async::new(slog_term::FullFormat::new(decorator).build().ignore_res()).build().fuse(), params),
    };

    let result = slog_scope::set_global_logger(logger);
    slog_stdlog::init().expect("Unable to set log backend");
    result
}
