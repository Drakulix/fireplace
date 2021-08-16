# <img src="https://cdn.rawgit.com/Drakulix/fireplace/v1.0.0/assets/fireplace.svg" width="128"> Fireplace  [![Crates.io](https://img.shields.io/crates/l/fireplace_lib.svg)](https://github.com/Drakulix/fireplace_lib/blob/master/LICENSE) 

## Old Codebase: https://github.com/Drakulix/fireplace/tree/old_codebase

______

## Active development currently at Matrix
[![Chatroom on Matrix](https://img.shields.io/badge/chatroom-on%20matrix-green.svg)](https://matrix.to/#/#smithay:matrix.org)


### A modular wayland window manager

Fireplace strives to be a slim and fast playground for a full-featured tiling-based wayland compositor for now.
Eventually I plan to implement a gnome-compatible desktop environment on top of fireplace
with resonable amounts of eye-candy to be appealing to the general user,
but without sacrificing on performance or keyboard based workflows.

The goal is to be gnome-abi compatible to applications (including the vast cast of dbus interfaces, but notably not gtk-specific styling or the horrific extension-api), while remaining composible and extendible through e.g. the wlr-protocols (most notable layer-shell).

What that transition means for fireplace and if the additional components of a full-featured desktop environment can be implemented without assimilating the original window manager is not yet decided. If possible fireplace should still be usable independently as a barebones wayland compositor.

Currently fireplace is getting reimplemented on top of [Smithay](https://github.com/Smithay/smithay),
which is not "done" in any sense of the word as well. Expect development to be slow and new features to be sparse.
As such fireplace will likely depend on git-versions of smithay from time to time and might not always compile.

## Status

Fireplace

- [x] Floating windows
- [-] Workspaces
- [ ] BSP-style window tiling
- [ ] Basic UI rendering using layer-shell
- [ ] ...

## Installation / Development

You are currently expected to know how to compile rust programs and how to start custom compositors.
Integrations for login managers will be provided at a later stage, when fireplace is deemed usable *enough*.

## Configuration

Configuration is done in [YAML](http://www.yaml.org/spec/1.2/spec.html) format.

You can see a detailed example at [fireplace.yaml](https://github.com/Drakulix/fireplace/blob/master/fireplace.yaml)

The configuration file should be placed into the `$XDG_CONFIG_DIR` - if set - or
into `$HOME/.config` otherwise. The name may either be `fireplace.yaml` or
`.fireplace.yaml`. You can also add a folder called `fireplace` and store the config file there, if you happen to like folders.

A global configuration can be provided in `/etc/fireplace/fireplace.yaml`.


## Contributing

Pull requests, feature requests, bug reports, every contribution is highly appreciated,
but please note, that I do this in my free time and your request maybe be given a very low
priority and postponed for quite some time.

The best way to participate is start hacking on the codebase. I will give my best to answer
any questions related to documentation and the core library as quickly as possible to assist
you!