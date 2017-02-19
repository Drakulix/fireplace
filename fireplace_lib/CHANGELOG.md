# Changelog

## 2.0.1

- Fixed an issue, when switching into a fullscreened workspace, the old geometry got restored

## 2.0.0

- Fix typo in `handlers::render::conrod::providers::statusbar::Location`. (`Buttom` instead of `Bottom`).

=> Breaks public api. Bump to 2.0.0.

## 1.0.3

- Fix floating `View`s keeping their old adjusted geometry when switching modes instead of falling back to their initial.

## 1.0.2

- Fix fullscreening messing up layout by readding `View` instead of remembering it's old position.


## 1.0.1

- Accidentially pushed - identical to 1.0.0

## 1.0.0

- Initial release
