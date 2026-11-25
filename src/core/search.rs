use std::fs;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use regex::{Regex, RegexBuilder};

/// Configuration for both single-document and workspace Find / Replace.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
}

/// One match inside a text buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextMatch {
    pub line: usize,
    pub column: usize,
    pub byte_start: usize,
    pub byte_end: usize,
    pub line_preview: String,
}

/// One match inside a workspace directory (Ripgrep-class Find in Files).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMatch {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub snippet: String,
}

/// Compile a user search query into a `Regex` honoring plain/regex, case, and whole-word flags.
pub fn build_regex(query: &str, opts: &SearchOptions) -> Result<Regex, String> {
    if query.is_empty() {
        return Err("Search query is empty".into());
    }
    let body = if opts.use_regex {
        query.to_owned()
    } else {
        regex::escape(query)
    };
    let pattern = if opts.whole_word {
        format!(r"\b(?:{body})\b")
    } else {
        body
    };
    RegexBuilder::new(&pattern)
        .case_insensitive(!opts.case_sensitive)
        .multi_line(true)
        .build()
        .map_err(|e| format!("Invalid search pattern: {e}"))
}

/// Find all matches in a single document.
pub fn find_all(text: &str, query: &str, opts: &SearchOptions) -> Result<Vec<TextMatch>, String> {
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let re = build_regex(query, opts)?;
    let mut results = Vec::new();
    let mut line_starts = vec![0usize];
    for (idx, b) in text.bytes().enumerate() {
        if b == b'\n' {
            line_starts.push(idx + 1);
        }
    }

    for m in re.find_iter(text) {
        let start = m.start();
        let line_idx = match line_starts.binary_search(&start) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let line_start = line_starts[line_idx];
        let line_end = text[line_start..]
            .find('\n')
            .map(|off| line_start + off)
            .unwrap_or(text.len());
        let col = text[line_start..start].chars().count();
        let preview = text[line_start..line_end]
            .trim()
            .chars()
            .take(120)
            .collect();

        results.push(TextMatch {
            line: line_idx,
            column: col,
            byte_start: start,
            byte_end: m.end(),
            line_preview: preview,
        });
    }
    Ok(results)
}

/// Replace all occurrences in a document; returns `(new_text, replacement_count)`.
pub fn replace_all(
    text: &str,
    query: &str,
    replacement: &str,
    opts: &SearchOptions,
) -> Result<(String, usize), String> {
    if query.is_empty() {
        return Ok((text.to_owned(), 0));
    }
    let re = build_regex(query, opts)?;
    let count = re.find_iter(text).count();
    if count == 0 {
        return Ok((text.to_owned(), 0));
    }
    let replaced = if opts.use_regex {
        re.replace_all(text, replacement).into_owned()
    } else {
        re.replace_all(text, regex::NoExpand(replacement))
            .into_owned()
    };
    Ok((replaced, count))
}

/// Replace a single match by index; returns `(new_text, did_replace)`.
pub fn replace_one(
    text: &str,
    query: &str,
    replacement: &str,
    match_index: usize,
    opts: &SearchOptions,
) -> Result<(String, bool), String> {
    let matches = find_all(text, query, opts)?;
    let Some(target) = matches.get(match_index) else {
        return Ok((text.to_owned(), false));
    };

    let mut out = String::with_capacity(text.len() + replacement.len());
    out.push_str(&text[..target.byte_start]);
    if opts.use_regex {
        let re = build_regex(query, opts)?;
        let segment = &text[target.byte_start..target.byte_end];
        out.push_str(&re.replace(segment, replacement));
    } else {
        out.push_str(replacement);
    }
    out.push_str(&text[target.byte_end..]);
    Ok((out, true))
}

/// Ripgrep-class `.gitignore`-aware recursive workspace search.
pub fn find_in_files(
    root: &Path,
    query: &str,
    opts: &SearchOptions,
    max_results: usize,
) -> Result<Vec<FileMatch>, String> {
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let re = build_regex(query, opts)?;
    let mut hits = Vec::new();
    let max_file_bytes: u64 = 5 * 1024 * 1024;

    let walker = WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    for entry in walker.flatten() {
        if hits.len() >= max_results {
            break;
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.len() > max_file_bytes {
            continue;
        }
        let Ok(bytes) = fs::read(path) else {
            continue;
        };
        if crate::core::encoding::looks_binary(&bytes) {
            continue;
        }
        let text = String::from_utf8_lossy(&bytes);
        for (line_idx, line) in text.lines().enumerate() {
            if let Some(m) = re.find(line) {
                let col = line[..m.start()].chars().count();
                let snippet: String = line.trim().chars().take(140).collect();
                hits.push(FileMatch {
                    path: path.to_path_buf(),
                    line: line_idx,
                    column: col,
                    snippet,
                });
                if hits.len() >= max_results {
                    break;
                }
            }
        }
    }

    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_and_replaces_with_regex_and_whole_word() {
        let doc = "alpha beta alphabet\nALPHA 123\nbeta 456";
        let opts = SearchOptions {
            case_sensitive: false,
            whole_word: true,
            use_regex: false,
        };
        let matches = find_all(doc, "alpha", &opts).unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line, 0);
        assert_eq!(matches[1].line, 1);

        let regex_opts = SearchOptions {
            case_sensitive: true,
            whole_word: false,
            use_regex: true,
        };
        let (replaced, count) = replace_all(doc, r"(\d+)", "[$1]", &regex_opts).unwrap();
        assert_eq!(count, 2);
        assert!(replaced.contains("[123]"));
        assert!(replaced.contains("[456]"));
    }

    #[test]
    fn find_in_files_scans_directory_cleanly() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("a.txt"), "hello world\naetheris editor\n").unwrap();
        fs::write(tmp.path().join("b.rs"), "fn main() { /* aetheris */ }\n").unwrap();
        let hits = find_in_files(tmp.path(), "aetheris", &SearchOptions::default(), 50).unwrap();
        assert_eq!(hits.len(), 2);
    }
}
