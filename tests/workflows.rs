use std::fs;
use std::io::Write;

use aetheris::core::csv_view::{detect_delimiter, parse_csv};
use aetheris::core::diff::{compute_diff, unified_diff};
use aetheris::core::encoding::{
    LineEnding, TextEncoding, read_file_decoded, read_file_with_encoding, write_file_encoded,
};
use aetheris::core::language::Language;
use aetheris::core::large_file::LargeFileReader;
use aetheris::core::macro_engine::{MacroRecorder, MacroStep};
use aetheris::core::search::{SearchOptions, find_in_files, replace_all};
use aetheris::core::session::{SessionState, SessionTab, SessionVault};
use aetheris::core::transforms::{TransformOp, apply_transform, filter_lines};

#[test]
fn end_to_end_encoding_disk_roundtrip_and_eol_conversion() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.ini");

    let text = "[server]\nhost=localhost\nlabel=Café — 日本語\n";
    write_file_encoded(&path, text, TextEncoding::Utf16Le, LineEnding::CrLf).unwrap();

    let decoded = read_file_decoded(&path).unwrap();
    assert_eq!(decoded.encoding, TextEncoding::Utf16Le);
    assert_eq!(decoded.line_ending, LineEnding::CrLf);
    assert_eq!(decoded.text, text);
    assert!(!decoded.looks_binary);

    // Convert to UTF-8 + LF and verify disk bytes
    write_file_encoded(&path, &decoded.text, TextEncoding::Utf8, LineEnding::Lf).unwrap();
    let reread = read_file_with_encoding(&path, TextEncoding::Utf8).unwrap();
    assert_eq!(reread.line_ending, LineEnding::Lf);
    assert_eq!(reread.text, text);
}

#[test]
fn end_to_end_macro_pipeline_and_workspace_search() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("data.json"),
        r#"{"service":"aetheris","version":1,"cross_platform":true}"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("notes.md"),
        "# Aetheris\nFast standalone editor\n",
    )
    .unwrap();

    let hits = find_in_files(
        dir.path(),
        "aetheris",
        &SearchOptions {
            case_sensitive: false,
            whole_word: true,
            use_regex: false,
        },
        50,
    )
    .unwrap();
    assert_eq!(hits.len(), 2);

    // Record and replay a multi-step developer macro
    let mut rec = MacroRecorder::new();
    rec.start_recording();
    rec.record(MacroStep::Transform(TransformOp::TrimBothWhitespace));
    rec.record(MacroStep::Transform(TransformOp::RemoveWhitespaceOnlyLines));
    rec.record(MacroStep::ReplaceAll {
        query: r"ERROR:\s+".into(),
        replacement: "[ERR] ".into(),
        case_sensitive: true,
        whole_word: false,
        use_regex: true,
    });
    rec.record(MacroStep::Transform(TransformOp::DeduplicateLines));
    rec.stop_recording();

    let raw = "  ERROR:   timeout  \n\n  ERROR: timeout\n  INFO: ok\n";
    let result = rec.replay_current(raw, 1).unwrap();
    assert_eq!(result, "[ERR] timeout\nINFO: ok");
}

#[test]
fn end_to_end_diff_csv_filter_session_and_large_file_hex() {
    // 1. Diff + unified patch
    let before = "fn main() {\n    println!(\"v1\");\n}\n";
    let after = "fn main() {\n    println!(\"v2\");\n    println!(\"ready\");\n}\n";
    let report = compute_diff(before, after);
    assert_eq!(report.removed_lines, 1);
    assert_eq!(report.added_lines, 2);
    assert!(unified_diff("a.rs", before, "b.rs", after).contains("+    println!(\"ready\");"));

    // 2. CSV auto-delimiter + filtering + sorting
    let csv_src = "id;city;pop\n3;Tokyo;37400000\n1;New York;18800000\n2;London;9500000\n";
    assert_eq!(detect_delimiter(csv_src), b';');
    let table = parse_csv(csv_src, 100).unwrap();
    let sorted = table.filtered_rows("", Some(0), true, 10);
    assert_eq!(sorted[0][1], "New York");
    assert_eq!(sorted[2][1], "Tokyo");

    // 3. Filter lines + regex replace
    let logs = "INFO started\nWARN disk 80%\nERROR crashed\nINFO recovered";
    let only_err = filter_lines(logs, r"^(WARN|ERROR)", true, false).unwrap();
    assert_eq!(only_err, "WARN disk 80%\nERROR crashed");
    let (redacted, n) = replace_all(
        only_err.as_str(),
        r"\d+%",
        "XX%",
        &SearchOptions {
            case_sensitive: true,
            whole_word: false,
            use_regex: true,
        },
    )
    .unwrap();
    assert_eq!(n, 1);
    assert!(redacted.contains("disk XX%"));

    // 4. Zero-prompt session vault persistence + orphan buffer cleanup
    let tmp = tempfile::tempdir().unwrap();
    let vault = SessionVault::new(tmp.path().join("vault"));
    let state = SessionState {
        active_tab_index: 0,
        workspace_root: None,
        theme_name: "Aetheris Midnight".into(),
        font_size: 15,
        word_wrap: true,
        show_line_numbers: true,
        tabs: vec![SessionTab {
            id: 7,
            title: "Scratch 7".into(),
            path: None,
            encoding: TextEncoding::Utf8,
            line_ending: LineEnding::Lf,
            language: Language::Rust,
            is_dirty: true,
            bookmarks: vec![0],
            backup_file: Some("tab-7.txt".into()),
        }],
    };
    vault.save(&state, &[(7, "let x = 42;".into())]).unwrap();
    assert_eq!(vault.load().unwrap().unwrap().tabs[0].id, 7);
    assert_eq!(vault.load_buffer("tab-7.txt").unwrap(), "let x = 42;");

    // 5. Large file mmap chunk + hex dump
    let mut big = tempfile::NamedTempFile::new().unwrap();
    for i in 0..2000 {
        writeln!(big, "entry_{i:05} payload=aetheris_fast_view").unwrap();
    }
    big.flush().unwrap();
    let reader = LargeFileReader::open(big.path()).unwrap();
    let hit = reader.find_next_bytes(b"entry_01500", 0).unwrap();
    let (_, _, slice) = reader.read_text_window(hit.saturating_sub(20), 512);
    assert!(slice.contains("entry_01500"));
    assert!(!reader.hex_rows(hit, 64).is_empty());

    // 6. Developer hash & encoding smoke check
    assert_eq!(
        apply_transform("aetheris", TransformOp::Base64Encode).unwrap(),
        "YWV0aGVyaXM="
    );
}
