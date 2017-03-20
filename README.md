# <img src="https://cdn.rawgit.com/Drakulix/fireplace/v1.0.0/assets/fireplace.svg" width="128"> Fireplace [![Build Status](https://travis-ci.org/Drakulix/fireplace.svg?branch=master)](https://travis-ci.org/Drakulix/fireplace) [![Crates.io](https://img.shields.io/crates/v/fireplace_lib.svg)](https://crates.io/crates/fireplace_lib) [![Crates.io](https://img.shields.io/crates/l/fireplace_lib.svg)](https://github.com/Drakulix/fireplace_lib/blob/master/LICENSE) [![](https://tokei.rs/b1/github/Drakulix/fireplace)](https://github.com/Aaronepower/tokei)

> He who wants to warm himself in old age must build a fireplace in his youth.


### A modular wayland window manager

![Screenshot](https://github.com/Drakulix/fireplace/raw/master/assets/screenshot.png "Screenshot")

Fireplace strifes to be as feature-rich as possible without compromising its slim and fast codebase. It is written in [Rust](https://www.rust-lang.org) and is based on the great [wlc](https://github.com/Cloudef/wlc) library and does its rendering directly in OpenGL.


### Structure

This repository is devided into three parts

- fireplace_lib - The underlying library which can be used to modify fireplace to your personal needs!
- fireplace - The reference implementation as presented here
- fireplace_flavors - Alternative implementations and experiments not belonging to the core library or binary

This README is about the reference implementation for end-users, if you are interested in learning about it's implementation details or contributing, please take a look at one of their README files.


## Status

Fireplace just hit 1.0, so it is in a usable state, but a bit limited.

- [x] BSP-style window tiling
- [x] Floating windows
- [x] Basic UI rendering with statusbar
- [x] Screenshots
- [x] ... and many more!

But it still missed some rather important features for every day use:

- [ ] Fix some remaining application specific bugs
- [ ] Lock screen
- [ ] ...and also many more...


## Installation

Binaries are provided on the GitHub Release Page for Linux x86_64

Packages are not provided at it's current state, but will be added to this description once available.


## Running

Follow the build instructions and run:
`./target/release/fireplace`

Starting with an X Server running will run fireplace nested.

For easier start up a [session file](https://github.com/Drakulix/fireplace/blob/master/fireplace.desktop) is provided, just copy it to `/usr/share/wayland-sessions/` and fireplace to `/usr/bin/` and fireplace should be visible in your desktop manager.

For running as a user process logind is required to optain the required permissions.
Alternatively set the `suid` flag on the executable and fireplace will drop privileges
after opening the required hardware devices.


## Configuration

Configuration is done in [YAML](http://www.yaml.org/spec/1.2/spec.html) format.

You can see a detailed example at [fireplace.yaml](https://github.com/Drakulix/fireplace/blob/master/fireplace.yaml)

The configuration file should be placed into the `$XDG_CONFIG_DIR` - if set - or
into `$HOME/.config` otherwise. The name may either be `fireplace.yaml` or
`.fireplace.yaml`. You can also add a folder called `fireplace` and store the config file there, if you happen to like folders.

A global configuration can be provided in `/etc/fireplace/fireplace.yaml`.

## Building

fireplace is written in [Rust](https://www.rust-lang.org) and therefore requires `Cargo` to build, which is shipped with the rust compiler.

fireplace also needs a current (>=1.17) *nightly* version of [Rust](https://www.rust-lang.org), if you have no idea, where to start, this command should help you to bootstrap a working toolchain:

```
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly
```

Additional dependencies for building & running include:

- wlc
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
- libfontconfig1
- libfreetype6

And optionally:

- dbus (for logind support)
- systemd (for logind support)

Build Dependencies:

- fontconfig
- libclang (>=3.8)

By default [wlc](https://github.com/Cloudef/wlc) is automatically build and statically linked into fireplace as well as wayland-protocols.

Building then is as easy as:
```
git clone https://github.com/Drakulix/fireplace.git
cd fireplace/fireplace
cargo build --release
```
Please take note that you cannot build in the root directory.
The resulting binary will be at `<root>/target/release/fireplace`.

To avoid statically linking and disable the optional Ui code use
```
cargo build --release --no-default-features
```

If you have `libclang` in another path then `/usr/lib` you have to provide it:
```
LIBCLANG_PATH=/usr/lib64 cargo build --release
```

For more ways to disable and enable certain features and other quirks see the [README](https://github.com/Drakulix/fireplace/blob/master/fireplace/README.md) of the fireplace binary.

For additional flavors see the READMEs of a specific flavor.


## Contributing

Pull requests, feature requests, bug reports, every contribution is highly appreciated,
but please note, that I do this in my free time and your request maybe be given a very low
priority and postponed for quite some time.
The best way to participate is start hacking on the codebase. I will give my best to answer
any questions related to documentation and the core library as quickly as possible to assist
you! A window manager sounds like a pretty complicated project, but most of the hard work is
already done by the underlying `wlc` library! So believe me when I say: "it's not that hard" :)

Please also make sure to read through [CONTRIBUTING](https://github.com/Drakulix/fireplace/blob/master/CONTRIBUTING) for some very basic constraints to follow
