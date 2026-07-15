# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-07-15

### Added

- Initial release.
- Documentation lookup for `std`/`core`/`alloc` and any crates.io crate via docs.rs.
- `-s/--source` syntax-highlighted source viewer with line-range highlighting.
- `-a/--show-all` full associated-item listing.
- `-i/--interactive` REPL mode with the `c <query>` clear-and-query shortcut.
- Local disk cache with configurable TTL, `--offline`, `--no-cache`, `--clear-cache`.
- 24-bit hex-colored terminal UI (panels, tables, spinner) via `make_colors`.
- `--no-emoji` / `--plain` / `--quiet` output modes for CI and piping.
- Colorful `--version` via `clap-version-flag`, colorful `--help` via `clap-color-help`.
