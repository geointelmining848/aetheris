# Aetheris

> **Feather-light. Truly standalone. Uncompromisingly versatile.**  
> A pure-Rust, zero-WebView, cross-platform text editor for **Windows, macOS, and Linux** — unifying Notepad++'s everyday utility, Sublime's speed, Kate/BBEdit's built-in diff, and EmEditor/UltraEdit's data & binary tools inside one self-contained native binary.

[![CI](https://github.com/geointelmining848/aetheris/actions/workflows/ci.yml/badge.svg)](https://github.com/geointelmining848/aetheris/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Why Aetheris?

Most developers end up juggling 3–4 tools because every editor leaves a gap:
- **VS Code / Electron editors** consume 500 MB–1.5 GB of RAM and choke on large files.
- **Notepad++** is fast and indispensable, but locked to Windows.
- **BBEdit / CotEditor** shine on macOS, but don't run on Windows or Linux.
- **EmEditor / UltraEdit / Sublime Text** charge steep commercial licenses.
- **WebView-based apps** still depend on external OS browser engines (`WebView2`, `WebKitGTK`).

**Aetheris** compiles to a **single, standalone native executable** powered by **Rust + `iced` (GPU-accelerated via Metal / DX12 / Vulkan with automatic CPU fallback)** — zero Electron, zero WebView, zero runtime dependencies.

---

## Key Features

### Notepad++ Ergonomics on Every OS
- **Zero-Prompt Session Vault**: Unsaved scratchpads, open file tabs, cursor positions, and language/encoding settings are automatically persisted and restored on relaunch — no "Save changes?" nag on exit.
- **One-Click Encoding & Line-Ending Mastery**: Auto-detects and converts `UTF-8`, `UTF-8 BOM`, `UTF-16 LE/BE`, `Windows-1252`, `ISO-8859-1`, `GB18030`, `Shift-JIS`, and `EUC-KR` via `chardetng` + `encoding_rs`, with instant `LF` / `CRLF` / `CR` toggling.
- **Macro Recorder & Player**: Record transformation/editing macros, replay once, replay *N* times, or save named reusable macros to disk.
- **Line Surgery & Bookmarks**: Sort (A–Z, Z–A, numeric, line length), deduplicate, strip empty/whitespace lines, trim trailing spaces, reverse lines, add line numbers, toggle line bookmarks, and extract or delete bookmarked lines.

### Developer Swiss-Army Knife (Built-In, Zero Plugins Needed)
- **Syntax Highlighting for 40+ Languages with Live Auto-Detection** powered by `syntect`: automatically recognizes code as you type or paste even in unsaved scratchpads (no file extension required), plus manual language switching via the status bar or Command Palette and curated Dark & Light themes (*Aetheris Midnight*, *Tokyo Night*, *Catppuccin Mocha*, *Nord*, *Solarized Dark*, *Paper Light*).
- **Split-Pane Editing & Multi-Tab Workbench**: Side-by-side editing of two tabs simultaneously.
- **Inline Find & Replace + Ripgrep-Class Find in Files**: Fast plain-text, case-sensitive, whole-word, and Rust Regex search/replace within a tab or across an entire workspace folder (respecting `.gitignore`).
- **Built-In Side-by-Side Diff View**: Compare any two open tabs or files on disk with color-coded additions, deletions, and change summaries.
- **Native Live Markdown Preview**: Split-pane native Markdown rendering without any embedded browser.
- **CSV / TSV Structured Grid View**: Auto-detects `,`, `\t`, `;`, `|` delimiters, renders aligned tabular data, and supports live row filtering and column sorting.
- **Hex / Binary Inspector**: Synchronized byte-offset, hexadecimal, and printable ASCII view with instant offset jump.
- **Large-File Fast View (`memmap2`)**: Memory-mapped chunked pager so massive multi-hundred-MB or multi-GB logs open immediately without exhausting RAM.
- **One-Click Text & Data Transforms**:
  - JSON Pretty-Print, Minify & Validate
  - XML Pretty-Print & Minify
  - Base64, URL, and HTML-Entity Encode / Decode
  - JWT Header & Payload Inspector
  - MD5, SHA-256, and SHA-512 Hash Generator
  - Case Converters (`UPPER`, `lower`, `Title`, `camelCase`, `snake_case`, `kebab-case`)
  - Unix Timestamp ↔ ISO-8601 UTC Converter
- **Fuzzy Command Palette (`Cmd/Ctrl + Shift + P`)**: Instant keyboard access to every editor command, transform, encoding, theme, and view mode.

---

## Keyboard Shortcuts

| Action | macOS | Windows / Linux |
| :--- | :--- | :--- |
| New Scratchpad Tab | `Cmd + N` | `Ctrl + N` |
| Open File | `Cmd + O` | `Ctrl + O` |
| Open Folder as Workspace | `Cmd + Shift + O` | `Ctrl + Shift + O` |
| Save / Save As | `Cmd + S` / `Cmd + Shift + S` | `Ctrl + S` / `Ctrl + Shift + S` |
| Close Active Tab | `Cmd + W` | `Ctrl + W` |
| Find / Replace in File | `Cmd + F` / `Cmd + H` | `Ctrl + F` / `Ctrl + H` |
| Find in Files (Workspace) | `Cmd + Shift + F` | `Ctrl + Shift + F` |
| Command Palette | `Cmd + Shift + P` | `Ctrl + Shift + P` |
| Toggle Sidebar | `Cmd + B` | `Ctrl + B` |
| Toggle Split View | `Cmd + \` | `Ctrl + \` |
| Go to Line | `Cmd + G` | `Ctrl + G` |
| Toggle Line Bookmark | `Cmd + K` | `Ctrl + K` |
| Zoom In / Out / Reset | `Cmd + =` / `Cmd + -` / `Cmd + 0` | `Ctrl + =` / `Ctrl + -` / `Ctrl + 0` |

---

## Building From Source

Requires Rust 1.85+ (Edition 2024):

```bash
git clone https://github.com/geointelmining848/aetheris.git
cd aetheris

# Run the full test suite
cargo test

# Build and launch optimized standalone binary
cargo run --release

# Open specific files or a workspace folder directly from CLI
cargo run --release -- path/to/file.rs path/to/folder
```

### Cross-Platform Binaries

GitHub Actions (`.github/workflows/release.yml`) automatically builds standalone executables for:
- **macOS** (Apple Silicon `aarch64` & Intel `x86_64`)
- **Windows** (`x86_64-pc-windows-msvc`)
- **Linux** (`x86_64-unknown-linux-gnu`)

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for module details and design notes.

---

## License

MIT © [geointelmining848](https://github.com/geointelmining848)
