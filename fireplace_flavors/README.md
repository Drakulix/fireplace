# <img src="https://cdn.rawgit.com/Drakulix/fireplace/bf10b919/assets/fireplace.svg" width="128"> Fireplace [![Build Status](https://travis-ci.org/Drakulix/fireplace.svg?branch=master)](https://travis-ci.org/Drakulix/fireplace) [![Crates.io](https://img.shields.io/crates/v/fireplace_lib.svg)](https://crates.io/crates/fireplace_lib) [![Crates.io](https://img.shields.io/crates/l/fireplace_lib.svg)](https://github.com/Drakulix/fireplace_lib/blob/master/LICENSE) [![Lines of Code](https://tokei.rs/b1/github/Drakulix/fireplace)](https://github.com/Aaronepower/tokei)

> He who wants to warm himself in old age must build a fireplace in his youth.


### A modular wayland window manager


## Fireplace Flavors

This folder is about the flavors of the fireplace binary.

For information on the actual reference window manager "Fireplace" please go [here](https://github.com/Drakulix/fireplace).

## Flavors

Flavors are variants which can not be merged into the `fireplace` library, because
they offer a different way to configure and manage the fireplace window manager.

They are alternatives or replacements.


Currently two flavors exist:

- [`json`](https://github.com/Drakulix/fireplace/blob/master/fireplace_flavors/json) - Uses the json format instead of yaml for configuration parsing
- [`code`](https://github.com/Drakulix/fireplace/blob/master/fireplace_flavors/code) - Uses a static in-code configuration
