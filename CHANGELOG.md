# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added `app cleanup` command for removing old versions and cache entries

### Changed

- Major refactor of `Structured` output struct
  - This includes an updated design for the output string

## [1.16.0] - 2025-19-01

### Fixes

- Fix match arms on disabled commands with certain feature flags

### Added

- Added config validations.
  - sfsu will now crash with an error message if `no_junction` is enabled.
- Added `app download --outdated` flag to download new versions of all outdated apps
- Returned logs in debug assertions going into current directory
- Warnings in search command for deprecated usage
- Support `json` flag in `search` command
- Warning to help message for `json` flag calling out that it only works for certain commands
- Progress reporting for `bucket add` command when not using `git` command

### Changed

- Removed `json` flag from `app download` command
- Download progress bars now show app name instead of url leaf
- Download hash checks now report to a progress bar rather than a print message for each
- Renamed `packages` parameter to `apps` in `app download` command (this should not affect usage at all)
- Logs will now go into `<PWD>/logs` if running with debug assertions
- `search` command no longer hides `[installed]` label if only searching for installed apps
- Removed `disable_git` flag from `bucket add` command
  - `bucket add` command now always uses gitoxide to clone the bucket
- `update` command renamed to `bucket update`

### Removed

- `sfsu_macros` crate

## [1.15.1] - 2025-18-01

- Update `sprinkles` to v0.20
- Update various other dependencies

## [1.15.0] - 2024-03-11

### Fixed

- Deprecation warning typo

### Added

- When passed no apps, the purge command will now offer to purge all uninstalled apps
- Purge command now has a dry run option

### Changed

- Minor performance improvements by removing `Cow` -> `String` conversion in `update` command
- Internal: Remove `Deref` from `Author`
- Updated dependencies
- Renamed `cache show` to `cache list` (alias to `show` added to avoid breaking change)
- Purge confirmation now shows both bucket and app name
- Purge command can now handle multiple apps
- Renamed `--verbose` to `--debug`
- `--verbose` flag help info changed to more accurately represent what it does
- Updated information does not show up by default in `app info` command as gathering the updated info is very slow
  - The user must pass `--verbose` to see the updated information
- Minor internal changes to `VTable` struct
- Change `app` subcommands hooks to use `app` subcommands

### Removed

- Short `-d` flags for debug and dry-run flags

## [1.14.0] - 2024-06-12

### Added

- Purge command for removing persist folders
- Added dependabot config
- `MinInfo` struct from sprinkles library
- MIT license option in addition to Apache-2.0 license
- More detailed sprinkles version in clap output
- Added sprinkles contributors to credits
- Enable `contexts` feature by default
- Logs are now moved to the new logging directory if any are found in the old location
- `app` command for managing apps

### Changed

- Moved sprinkles library to seperate repo
- Renamed sfsu-derive to sfsu-macros
- Updated sprinkles library
- Use Rust nightly toolchain
- Logs now go into `LocalAppData\sfsu\logs` instead of `<sfsu install folder>\logs`
- Run debug build on push and only run release build on release
- Internal: Do not make `wrappers` module public
- Moved `purge` command into `app` subcommand
- Internal: allow dead code in `Signature` impl (functions reserved for future use)
- Moved all app related commands into `app` subcommand, and added aliases in root command
- Internal: move command docs to structs for modularity
- Use spinner for manifest gen

### Removed

- `info-difftrees` feature flag
- Bot contributions from contributors list

### Fixed

- CI builds
- Re-run build.rs if executable manifest changes
- Remove redundant features of `bat` crate

For older version's changelogs, see the [releases](https://github.com/winpax/sfsu/releases) page.

[Unreleased]: https://github.com/winpax/sfsu/compare/v1.16.0...HEAD
[1.16.0]: https://github.com/winpax/sfsu/releases/tag/v1.16.0
[1.15.1]: https://github.com/winpax/sfsu/releases/tag/v1.15.1
[1.15.0]: https://github.com/winpax/sfsu/releases/tag/v1.15.0
[1.14.0]: https://github.com/winpax/sfsu/releases/tag/v1.14.0
