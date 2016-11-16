# <img src="https://cdn.rawgit.com/Drakulix/fireplace/bf10b919/assets/fireplace.svg" width="128"> Fireplace [![Build Status](https://travis-ci.org/Drakulix/fireplace.svg)](https://travis-ci.org/Drakulix/fireplace) [![Crates.io](https://img.shields.io/crates/v/fireplace_lib.svg)](https://crates.io/crates/fireplace_lib) [![Crates.io](https://img.shields.io/crates/l/fireplace_lib.svg)](https://github.com/Drakulix/fireplace_lib/blob/master/LICENSE) [![Lines of Code](https://tokei.rs/b1/github/Drakulix/fireplace)](https://github.com/Aaronepower/tokei)

> He who wants to warm himself in old age must build a fireplace in his youth.


### A modular wayland window manager


## Fireplace Binary

The folder is about details of the fireplace binary.

For general information on the window manager "Fireplace" please go [here](https://github.com/Drakulix/fireplace).


## Purpose

The fireplace binary is responsable for initializing the logging framework,
parsing the config file and passing the parameter into the [`fireplace_lib`](https://github.com/Drakulix/fireplace/blob/master/fireplace_lib)
library and starting the compositor via `wlc`.

Additional variants fulfilling the same role in different ways (e.g. different configuration
format or a static configuration) can be found in [`fireplace_flavors`](https://github.com/Drakulix/fireplace/blob/master/fireplace_flavors)
in this repo.


## Dependencies

Build & Runtime dependencies are:

- wlc (only without `static` feature)
- pixman
- wayland 1.7+
- wayland-protocols 1.7+
- libxkbcommon
- udev
- libinput
- libx11 (X11-xcb, Xfixes)
- libxcb (xcb-ewmh, xcb-composite, xcb-xkb, xcb-image, xcb-xfixes)
- libgbm (usually provided by mesa in most distros)
- libdrm
- libEGL (GPU drivers and mesa provide this)
- libGLESv2 (GPU drivers and mesa provide this)
- libfontconfig1 (with feature `ui`)
- libfreetype6 (with feature `ui`)

And optionally:

- dbus (for logind support)
- systemd (for logind support)

Build-only Dependencies:

- fontconfig (with feature `ui`)
- libclang (>=3.8)


## Building

For building fireplace with it's full feature set you can just use:
```
git clone https://github.com/Drakulix/fireplace.git
cd fireplace
cargo build --release
```
which will result in a binary at `target/release/fireplace`.


fireplace offers multiple features, which are all enabled by default:
- `static` - Compile `wlc` and link statically
- `ui` - Enable Ui rendering.


To disable features you may use:
```
cargo build --release --no-default-features
```

To re-enable certain features, e.g. to do dynamic linking for a distribution package use:
```
cargo build --release --no-default-features --features "ui"
```

If you have `libclang` in another path then `/usr/lib` you have to provide it when building your binary:
```
LIBCLANG_PATH=/usr/lib64 cargo build --release
```

See https://github.com/KyleMayes/clang-sys#environment-variables for more options.


## Contributing

Contributions are highly welcome, just note that you usally will not make contributions
to this binary, as it is pretty slim and quite straight forward, but to the underlying
[`fireplace_lib`](https://github.com/Drakulix/fireplace/blob/master/fireplace_lib)
which provides most functionality and is also far better documentated.
