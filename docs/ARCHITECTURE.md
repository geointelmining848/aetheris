# Aetheris Architecture

Aetheris is designed as a **100% pure-Rust, zero-WebView, standalone native desktop application** split cleanly into a UI-agnostic **Core Engine (`src/core/`)** and a reactive GPU-accelerated **Workbench UI (`src/app/`)**.

## Design Principles

1. **Zero External Runtime Dependencies**: No Chromium, no WebView2, no WebKitGTK, no Node.js. Aetheris renders natively via `iced` (`wgpu` GPU backend with automatic `tiny-skia` software fallback).
2. **Testable Core Separation**: All encoding detection, line/data transforms, regex search, ripgrep workspace scanning, diffing, macro recording, CSV parsing, hex dumping, large-file memory mapping, and session persistence live in `src/core/` with zero GUI dependencies and comprehensive unit tests.
3. **Zero-Prompt Persistence**: Unsaved scratchpads and session metadata are stored under the OS user data directory (`dirs::data_local_dir()/aetheris/session/`), guaranteeing instant restoration after quits or unexpected shutdowns.
4. **Memory-Safe Large-File Support**: Files exceeding the configurable Fast View threshold (default 10 MB, or opened explicitly in Fast View) use `memmap2` read-only memory maps to paginate arbitrary byte/line windows and hex views without copying gigabytes into RAM.

## Module Map

| Module | Responsibility |
| :--- | :--- |
| `src/core/encoding.rs` | Character encoding detection (`chardetng`) + decoding/encoding (`encoding_rs`) across UTF-8, UTF-8 BOM, UTF-16 LE/BE, Windows-1252, ISO-8859-1, GB18030, Shift-JIS, EUC-KR; CRLF / LF / CR detection & lossless conversion |
| `src/core/language.rs` | Filename & extension detection for 45+ programming, config, and markup languages |
| `src/core/session.rs` | Atomic zero-prompt session vault persisting open tabs, unsaved scratchpads, bookmarks, encoding, EOL, and active workspace folder |
| `src/core/transforms.rs` | Line surgery (sort/dedupe/trim/reverse/filter/number/join), case conversions, JSON/XML pretty-print & minify, Base64/URL/HTML/JWT tools, MD5/SHA-256/SHA-512 hashing, Unix timestamp conversion |
| `src/core/search.rs` | Single-buffer plain/regex/whole-word find & replace + multi-file `.gitignore`-aware workspace search (`ignore` + `regex`) |
| `src/core/diff.rs` | Side-by-side & unified line diffing (`similar`) with change statistics |
| `src/core/macro_engine.rs` | Action-sequence recorder, single/repeated player, and JSON persistence for named user macros |
| `src/core/large_file.rs` | `memmap2` chunked viewer & streaming search for huge files + 16-byte Hex/ASCII dump formatter |
| `src/core/csv_view.rs` | Delimiter auto-detector, RFC-4180 CSV/TSV parser, column sorting, and substring/regex row filter |
| `src/app/` | `iced 0.14` application state, multi-tab & split-pane workbench, Command Palette, theme system, keyboard shortcuts, and multi-mode viewport |
