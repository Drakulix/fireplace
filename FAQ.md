# FAQ - kindly taken from [sway](https://github.com/SirCmpwn/sway/wiki)

## Nvidia users

You must use the open source Nouveau drivers.
Nvidia's proprietary binary drivers are not supported.

## Keyboard repeat delay and rate

Set the environment variables `WLC_REPEAT_DELAY`/`WLC_REPEAT_RATE` to the delay/rate in milliseconds before starting fireplace.

## Keyboard layout

You have to set the keyboard layout before starting fireplace, e.g. `XKB_DEFAULT_LAYOUT=de fireplace`.
It is also possible to set other options known from setxkbmap with the environment variables `XKB_DEFAULT_MODEL`, `XKB_DEFAULT_LAYOUT`, `XKB_DEFAULT_VARIANT`, `XKB_DEFAULT_OPTIONS`.
This example enables switching between the american layout, and the german layout without dead keys with Alt-Shift.
Supported parameters are defined in /usr/share/X11/xkb/symbols/\*.

```
export XKB_DEFAULT_LAYOUT=us,de
export XKB_DEFAULT_VARIANT=,nodeadkeys
export XKB_DEFAULT_OPTIONS=grp:alt_shift_toggle
fireplace
```

## Blank window in Java application.

Try to set `_JAVA_AWT_WM_NONREPARENTING=1` in your environment.
