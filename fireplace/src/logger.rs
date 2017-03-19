//! Compositor Logging Configuration

use slog;
use slog::{DrainExt, Duplicate, Level, LevelFilter};
use slog_html;
use slog_journald;
use slog_scope;
use slog_stdlog;
use slog_stream;
use slog_term;

use std::fs;
use std::fs::OpenOptions;
use std::path::PathBuf;

/// Configuration for fireplace's Logger
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Logging {
    #[serde(default)]
    /// Style of the terminal logging
    pub style: Mode,
    #[serde(default)]
    /// Enabling of colored terminal output
    pub color: Color,
    #[serde(default)]
    /// Optionally create a logging file
    pub file: Option<PathBuf>,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[allow(missing_docs)]
/// Terminal color output options
enum_str!(pub enum Color
{
    Auto,
    Always,
    Never,
});

impl Default for Color {
    fn default() -> Color {
        Color::Auto
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[allow(missing_docs)]
/// Style of the logging output
enum_str!(pub enum Mode
{
    Compact,
    Full,
});

impl Default for Mode {
    fn default() -> Mode {
        Mode::Compact
    }
}

/// Initialize fireplace's logging system
pub fn init(config: Logging) {
    let a = slog_term::streamer().stderr().async();
    let b = match config.style {
        Mode::Compact => a.compact(),
        Mode::Full => a.full(),
    };
    let c = match config.color {
        Color::Always => b.color(),
        Color::Never => b.plain(),
        Color::Auto => b.auto_color(),
    };

    let always = Duplicate::new(LevelFilter::new(slog_journald::JournaldDrain, Level::Debug).ignore_err(),
                                LevelFilter::new(c.build(), Level::Info).ignore_err())
            .ignore_err();

    let root = if let Some(path) = config.file {
        fs::create_dir_all(path.parent().unwrap()).expect("Failed to create log file directory");
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap();

        let stream = slog_stream::stream(file, slog_html::new().build());

        slog::Logger::root(Duplicate::new(always, LevelFilter::new(stream.fuse(), Level::Debug)).ignore_err(),
                           o!("version" => env!("CARGO_PKG_VERSION")))
    } else {
        slog::Logger::root(always.ignore_err(),
                           o!("version" => env!("CARGO_PKG_VERSION")))
    };

    let _ = slog_stdlog::set_logger(root.new(o!("Library" => "WLC")));
    slog_scope::set_global_logger(root.new(o!("Library" => "Fireplace")));
}
