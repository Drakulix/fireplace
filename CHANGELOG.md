# Changelog

## 3.0.0

- Update wlc dependency (considered public api)
- Add screenshot feature
- Move ui configuration into output config

## 2.0.1

- Fixed an issue, when switching into a fullscreened workspace, the old geometry got restored

## 2.0.0

- Fix typo in statusbar->location. (Buttom instead of Bottom).

=> Breaks old config files. Bump to 2.0.0.

## 1.0.3

- Fix folder creation for the log file. Credit @IntrepidPig

## 1.0.2

- Fix floating `View`s keeping their old adjusted geometry when switching modes instead of falling back to their initial.

## 1.0.1

- Fix fullscreening messing up layout by readding `View` instead of remembering it's old position.

## 1.0.0

- Initial release
