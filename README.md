# rshelp

**Enhanced Rust documentation helper with beautiful terminal output.**

`rshelp` is the Rust sibling of [`pyhelp`](https://github.com/cumulus13/pyhelp):
look up documentation for the Rust standard library, any crate on
[crates.io](https://crates.io) (via [docs.rs](https://docs.rs)), and view
syntax-highlighted source code, without leaving your terminal.

```
$ rshelp std::vec::Vec
┌──────────────────────────────────────────────────────────┐
│ 🦀 rshelp v0.1.0 🚀 📚                                   │
│ Beautiful terminal help for the Rust ecosystem           │
└──────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────┐
│ 🎯 Target: std::vec::Vec                                 │
└──────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────┐
│ ✅ Found: Vec in std::vec - Rust                         │
│ https://doc.rust-lang.org/stable/std/vec/struct.Vec.html │
└──────────────────────────────────────────────────────────┘
┌── 📄Signature ───────────────────────────────────────────┐
│ pub struct Vec<T, A = Global> { ... }                    │
└──────────────────────────────────────────────────────────┘
┌── 📖Documentation ───────────────────────────────────────┐
│ A contiguous growable array type, written as Vec<T> ...  │
└──────────────────────────────────────────────────────────┘
```

## Features

- 📚 **Look up anything** -- `std`/`core`/`alloc`, or any published crate, by
  path: `rshelp serde::Deserialize`, `rshelp tokio::spawn`, `rshelp clap`.
- 🔤 **Dots or `::`, your choice** -- `std.collections.HashMap` works exactly
  like `std::collections::HashMap`.
- 🧠 **Common-type shortcuts** -- `rshelp vec`, `rshelp hashmap`, `rshelp arc`
  resolve straight to their `std` types.
- 📄 **Source view** -- `-s/--source` fetches and syntax-highlights the real
  source code, jumping to (and framing) the relevant line range.
- 🔧 **Full method listing** -- `-a/--show-all` lists every associated
  item/trait impl instead of a truncated summary.
- 🔁 **Interactive mode** -- `-i/--interactive` keeps a REPL open; type `c
  <query>` to clear the screen and look something else up, `q`/`quit`/`x`/
  `exit` to leave.
- 💾 **Smart caching** -- pages are cached locally (`--cache-ttl`,
  `--no-cache`, `--clear-cache`), and `--offline` works entirely from cache.
- 🎨 **Themeable, scriptable output** -- 24-bit hex colors throughout, with
  `--no-emoji`, `--plain`, and `--quiet` for logs, CI, and piping; color and
  emoji auto-disable when stdout isn't a terminal.
- 🖌️ **Configurable theme** -- pick a built-in preset or override individual
  colors in a TOML config file; see [Configuration](#configuration) below.

## Configuration

`rshelp` needs zero configuration to work, but every color is customizable.
Generate an editable, annotated config file with:

```sh
rshelp --init-config
```

This writes to your platform's config directory (e.g.
`~/.config/rshelp/config.toml` on Linux, `~/Library/Application
Support/rshelp/config.toml` on macOS, `%APPDATA%\rshelp\config.toml` on
Windows) -- or pass `--config <path>` to use a different file, either to
generate it there or to have any run of `rshelp` read from it.

```toml
[theme]
preset = "dracula"      # default, dracula, nord, monokai, gruvbox
# primary = "#00FFFF"   # override any individual color on top of the preset

[defaults]
cache_ttl = 86400
timeout   = 15
```

Try a preset for one run without touching the file: `rshelp --preset nord
serde::Deserialize`. Individual color fields (`primary`, `accent`,
`success`, `warning`, `error`, `info`, `dim`, `keyword`, `type_name`,
`string`, `comment`, `macro`, `attribute`, `number`) accept any `#RRGGBB`
hex value and are applied on top of the chosen preset.

## Installation

### From crates.io

```sh
cargo install rshelp
```

### From a GitHub release

Download the archive for your platform from the
[releases page](https://github.com/cumulus13/rshelp/releases), extract it,
and put the `rshelp` binary on your `PATH`.

### From source

```sh
git clone https://github.com/cumulus13/rshelp.git
cd rshelp
cargo install --path .
```

## Usage

```
rshelp [OPTIONS] <PATH>...

Examples:
  rshelp std::vec::Vec                 Show help for std::vec::Vec
  rshelp std.collections.HashMap       Dots work just like ::
  rshelp -s serde::Deserialize         Show source code for a trait
  rshelp -a tokio::process::Command    Show every method, unabridged
  rshelp -i clap                       Interactive mode; keep querying
  rshelp --crate-version 1.0.4 anyhow  Look up docs for a pinned version
  rshelp --clear-cache                 Wipe the local documentation cache
```

<details>
<summary>Full flag reference</summary>

| Flag | Description |
| --- | --- |
| `-s`, `--source` | Show syntax-highlighted source code instead of documentation |
| `-a`, `--show-all` | Show every associated item instead of a truncated summary |
| `-i`, `--interactive` | Stay in an interactive REPL after showing a result |
| `--crate-version <VERSION>` | Pin a specific crate version instead of `latest` |
| `--offline` | Only use the local cache; never touch the network |
| `--no-cache` | Bypass the cache for this lookup |
| `--clear-cache` | Delete all locally cached documentation pages and exit |
| `--cache-ttl <SECS>` | How long cached pages stay fresh (default `86400`) |
| `--timeout <SECS>` | Network request timeout (default `15`) |
| `--no-emoji` | Disable emoji in output |
| `--plain` | Disable colors and emoji entirely |
| `-q`, `--quiet` | Suppress banner/status panels; print only the content |
| `--config <PATH>` | Use a specific config file instead of the default location |
| `--init-config` | Write an annotated default config file and exit |
| `--preset <NAME>` | Use a built-in color preset for this run |
| `-V`, `--version` | Print colorful version information and exit |
| `-h`, `--help` | Print help |

</details>

## How it works

There's no compiler introspection here -- `rshelp` resolves an item path the
way a human browsing documentation in a browser would: it builds the most
likely `docs.rs`/`doc.rust-lang.org` URL(s) for the path you gave it (trying
`struct.X.html`, `trait.X.html`, `fn.x.html`, a module index, and so on, in an
order biased by Rust naming conventions), fetches the first one that exists,
and extracts the signature, docs, methods, and source link straight out of
the rendered HTML. If a crate's docs use a page layout `rshelp` doesn't
recognize, it falls back to a readable whole-page text dump rather than
failing outright.

## License

MIT © [Hadi Cahyadi](mailto:cumulus13@gmail.com)

---

## 👤 Author
        
[Hadi Cahyadi](mailto:cumulus13@gmail.com)
    

[![Buy Me a Coffee](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/cumulus13)

[![Donate via Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/cumulus13)
 
[Support me on Patreon](https://www.patreon.com/cumulus13)
