# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.5.0](https://github.com/JayanAXHF/templatex/compare/v0.4.0...v0.5.0) - 2025-12-08

### Added

- [**breaking**] Added a build script for proper version messaging, and added documentatino to CLI

### Other

- *(release)* Fixed version number

## [0.3.0] - 2025-12-07

### Features
- *(TUI)* Added template search (by @JayanAXHF)
- *(template)* [**breaking**] Changed template syntax to `<~{ ... }~>` to avoid latex weirdness (by @JayanAXHF)
- Added support for copying over image files from the template (by @JayanAXHF)
- *(config)* Added support for custom themes (by @JayanAXHF)
- *(templates)* [**breaking**] Added include, exclude and ignore options to templatex.toml files (by @JayanAXHF)


### Bug Fixes
- Updated README and sample TOML file with new syntax (by @JayanAXHF)


### Refactor
- [**breaking**] Updated TUI state to use get_dirs() helper method instead of direct access (by @JayanAXHF)


### Styling
- *(CHANGELOG)* Changed CHANGELOG format (by @JayanAXHF)


### Miscellaneous Tasks
- release v0.2.0 (by @JayanAXHF)
- release v0.3.0 (by @JayanAXHF)

## [0.1.2] - 2025-12-05

### Bug Fixes
- *(README)* Fixed the variable syntax description in the README (by @JayanAXHF)


### Miscellaneous Tasks
- release v0.1.2 (by @JayanAXHF)

## [0.1.1] - 2025-12-05

### Features
- finished initial POC (by @JayanAXHF)
- Added README (by @JayanAXHF)


### Bug Fixes
- removed extra tag (by @JayanAXHF)
- added `--locked` to install command (by @JayanAXHF)


### Miscellaneous Tasks
- added changelog and added optimisations (by @JayanAXHF)
- Added metadata to Cargo.toml (by @JayanAXHF)
- release v0.1.1 (by @JayanAXHF)
