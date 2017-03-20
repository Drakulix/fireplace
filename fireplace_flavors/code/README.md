# <img src="https://cdn.rawgit.com/Drakulix/fireplace/v1.0.0/assets/fireplace.svg" width="128"> Fireplace [![Build Status](https://travis-ci.org/Drakulix/fireplace.svg?branch=master)](https://travis-ci.org/Drakulix/fireplace) [![Crates.io](https://img.shields.io/crates/v/fireplace_lib.svg)](https://crates.io/crates/fireplace_lib) [![Crates.io](https://img.shields.io/crates/l/fireplace_lib.svg)](https://github.com/Drakulix/fireplace_lib/blob/master/LICENSE) [![Lines of Code](https://tokei.rs/b1/github/Drakulix/fireplace)](https://github.com/Aaronepower/tokei)

> He who wants to warm himself in old age must build a fireplace in his youth.


### A modular wayland window manager


## Fireplace Binary Code Flavor

This folder is one of many fireplace flavors. For an overview of flavors and what
they are look [here](https://github.com/Drakulix/fireplace/blob/master/fireplace_flavors).

For information on the actual reference window manager "Fireplace" please go [here](https://github.com/Drakulix/fireplace).

This flavor initializes all `fireplace_lib` handlers without a single configuration
file all in code.
It serves as both

- a way to use code as your configuration file (basically what you need to change is all in main.rs)
- as an example on where to start with `fireplace_lib`, as it shows the most basic way to get it running.
