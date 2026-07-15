# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5] - 2026-07-16

### Fixed

- Panel/box borders no longer drift out of alignment when a line contains
  emoji (double-width) or ANSI color codes -- padding is now computed with
  real terminal column width (`unicode-width`), not `.chars().count()`.
- The signature panel, method/trait-impl list, and inline `` `code` ``
  spans inside documentation prose are now actually syntax-highlighted
  (previously only `-s/--source` output was).
- Multi-line fenced code examples in documentation (rustdoc's `<pre><code>`
  blocks, rendered by `html2text` with a lone backtick on the opening line
  and a lone backtick on the closing line) are now detected as whole blocks
  and fully highlighted, instead of showing as unhighlighted text with two
  stray backtick characters.
- Removed rustdoc's noisy reference-style link footnotes
  (`[label][3]` / `[3]: https://...`) from documentation text.
- Fixed stray extra spaces in extracted signatures and method lists
  (`Global >`, `len (&self)`) caused by joining HTML text nodes with `" "`
  instead of concatenating them directly.
- Fixed a missing space between the crab and rocket emoji in the startup
  banner.

### Added

- TOML config file support: `--init-config` writes an annotated template,
  `--config <PATH>` points at a custom location. Supports built-in color
  presets (`default`, `dracula`, `nord`, `monokai`, `gruvbox`, selectable
  via `[theme] preset` or `--preset <NAME>`), per-color hex overrides, and
  default values for `cache_ttl`/`timeout`/`crate_version`/emoji-color-quiet
  flags that apply only when the matching CLI flag isn't passed.

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
