# <img src="https://cdn.rawgit.com/Drakulix/fireplace/bf10b919/assets/fireplace.svg" width="128"> Fireplace [![Build Status](https://travis-ci.org/Drakulix/fireplace.svg)](https://travis-ci.org/Drakulix/fireplace) [![Crates.io](https://img.shields.io/crates/v/fireplace_lib.svg)](https://crates.io/crates/fireplace_lib) [![Crates.io](https://img.shields.io/crates/l/fireplace_lib.svg)](https://github.com/Drakulix/fireplace_lib/blob/master/LICENSE) [![Lines of Code](https://tokei.rs/b1/github/Drakulix/fireplace)](https://github.com/Aaronepower/tokei)

> He who wants to warm himself in old age must build a fireplace in his youth.


### A modular wayland window manager


## Fireplace Library

This folder is about the modular fireplace_lib library for creating your own
window manager / wayland compositor based the functionality provided by this library.

For information on the actual reference window manager "Fireplace" please go [here](https://github.com/Drakulix/fireplace).


## Documentation

[Documentation](https://Drakulix.github.io/fireplace) is hosted on GitHub pages,
because special build parameters are required - see "Usage" - which makes it
unable to build on docs.rs


## Usage

In your Cargo.toml

```
fireplace_lib = "^1.0.0"
```

fireplace_lib offers multiple features, which are all enabled by default:
- `static` - Compile `wlc` and link statically
- `render` - Enable rendering at all
- `gl` - Implies `render`, provides functionality for OpenGL ES rendering
- `graphics` - Implies `render` and `gl`, provides functionality for rendering with [`piston2d-graphics`](https://crates.io/crates/piston2d-graphics)
- `conrod_ui` - Implies `render`, `gl` and `graphics`, provides functionality for rendering with [`conrod`](https://crates.io/crates/conrod)

E.g. to use only `gl`:
```
fireplace_lib = { version = "^1.0.0", default-features = false, features = ["gl"] }
```


## Building

Runtime and Build dependencies:

- wlc (without `static`)
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

If you have `libclang` in another path then `/usr/lib` you have to provide it when building your binary:
```
LIBCLANG_PATH=/usr/lib64 cargo build --release
```

See https://github.com/KyleMayes/clang-sys#environment-variables


## Examples

The most basic examples initializing **all** `fireplace_lib` features is keeped in [`fireplace_flavors/code`](https://github.com/Drakulix/fireplace/blob/master/fireplace_flavors/code)


# Contributing

I will happily accept new features or bug fixes, given that they are:

- easily configurable via a struct that implements `Deserialize` via [`serde`](https://serde.rs/)
- is split into a seperate `feature` if it increases compile time substancially and/or will most likely not be used by a large amount of users
- passes `cargo clippy` except for well explained exceptions
- was formated with `rustfmt` and the config in the repository's root
- roughly follows the design of existing code

If you are not sure, if your contribution does match these rules, open an issue or a
pull request and lets discuss the subject.

Also note you can easily keep your personal changes out-of-tree by creating another
rust library crate, that can be included in other forks, if you still want to make it
available to the public or if you just add your code directly into your personal binary.

Like this you will never need to modify `fireplace_lib` in the first place,
you can easier maintain your fork and still be up to date with new functionality.
