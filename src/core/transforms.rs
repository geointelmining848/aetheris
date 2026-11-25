use std::collections::HashSet;
use std::io::Cursor;

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use md5::Md5;
use quick_xml::Reader;
use quick_xml::Writer;
use quick_xml::events::Event;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};

/// All built-in Notepad++ / developer Swiss-Army text and data transformations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransformOp {
    // Line surgery
    SortLinesAsc,
    SortLinesDesc,
    SortLinesNumeric,
    SortLinesByLength,
    ReverseLines,
    DeduplicateLines,
    RemoveEmptyLines,
    RemoveWhitespaceOnlyLines,
    TrimTrailingWhitespace,
    TrimLeadingWhitespace,
    TrimBothWhitespace,
    AddLineNumbers,
    JoinLinesWithSpace,
    JoinLinesWithComma,

    // Case transformations
    Uppercase,
    Lowercase,
    TitleCase,
    CamelCase,
    SnakeCase,
    KebabCase,
    InvertCase,

    // Data & code formatters
    JsonPretty,
    JsonMinify,
    XmlPretty,
    XmlMinify,

    // Encodings & web utilities
    Base64Encode,
    Base64Decode,
    UrlEncode,
    UrlDecode,
    HtmlEscape,
    HtmlUnescape,
    JwtDecode,

    // Hashes & timestamps
    HashMd5,
    HashSha256,
    HashSha512,
    UnixTimestampToIso,
    InsertCurrentIsoUtc,
}

impl TransformOp {
    pub const ALL: &'static [TransformOp] = &[
        TransformOp::SortLinesAsc,
        TransformOp::SortLinesDesc,
        TransformOp::SortLinesNumeric,
        TransformOp::SortLinesByLength,
        TransformOp::ReverseLines,
        TransformOp::DeduplicateLines,
        TransformOp::RemoveEmptyLines,
        TransformOp::RemoveWhitespaceOnlyLines,
        TransformOp::TrimTrailingWhitespace,
        TransformOp::TrimLeadingWhitespace,
        TransformOp::TrimBothWhitespace,
        TransformOp::AddLineNumbers,
        TransformOp::JoinLinesWithSpace,
        TransformOp::JoinLinesWithComma,
        TransformOp::Uppercase,
        TransformOp::Lowercase,
        TransformOp::TitleCase,
        TransformOp::CamelCase,
        TransformOp::SnakeCase,
        TransformOp::KebabCase,
        TransformOp::InvertCase,
        TransformOp::JsonPretty,
        TransformOp::JsonMinify,
        TransformOp::XmlPretty,
        TransformOp::XmlMinify,
        TransformOp::Base64Encode,
        TransformOp::Base64Decode,
        TransformOp::UrlEncode,
        TransformOp::UrlDecode,
        TransformOp::HtmlEscape,
        TransformOp::HtmlUnescape,
        TransformOp::JwtDecode,
        TransformOp::HashMd5,
        TransformOp::HashSha256,
        TransformOp::HashSha512,
        TransformOp::UnixTimestampToIso,
        TransformOp::InsertCurrentIsoUtc,
    ];

    pub fn label(self) -> &'static str {
        match self {
            TransformOp::SortLinesAsc => "Lines: Sort A → Z",
            TransformOp::SortLinesDesc => "Lines: Sort Z → A",
            TransformOp::SortLinesNumeric => "Lines: Sort Numerically",
            TransformOp::SortLinesByLength => "Lines: Sort by Length",
            TransformOp::ReverseLines => "Lines: Reverse Order",
            TransformOp::DeduplicateLines => "Lines: Remove Duplicates",
            TransformOp::RemoveEmptyLines => "Lines: Remove Empty Lines",
            TransformOp::RemoveWhitespaceOnlyLines => "Lines: Remove Blank/Whitespace Lines",
            TransformOp::TrimTrailingWhitespace => "Lines: Trim Trailing Spaces",
            TransformOp::TrimLeadingWhitespace => "Lines: Trim Leading Spaces",
            TransformOp::TrimBothWhitespace => "Lines: Trim Leading & Trailing Spaces",
            TransformOp::AddLineNumbers => "Lines: Prefix Line Numbers",
            TransformOp::JoinLinesWithSpace => "Lines: Join with Space",
            TransformOp::JoinLinesWithComma => "Lines: Join with Comma",
            TransformOp::Uppercase => "Case: UPPERCASE",
            TransformOp::Lowercase => "Case: lowercase",
            TransformOp::TitleCase => "Case: Title Case",
            TransformOp::CamelCase => "Case: camelCase",
            TransformOp::SnakeCase => "Case: snake_case",
            TransformOp::KebabCase => "Case: kebab-case",
            TransformOp::InvertCase => "Case: iNVERT cASE",
            TransformOp::JsonPretty => "JSON: Pretty-Print (2 spaces)",
            TransformOp::JsonMinify => "JSON: Minify",
            TransformOp::XmlPretty => "XML: Pretty-Print",
            TransformOp::XmlMinify => "XML: Minify",
            TransformOp::Base64Encode => "Base64: Encode",
            TransformOp::Base64Decode => "Base64: Decode",
            TransformOp::UrlEncode => "URL: Percent-Encode",
            TransformOp::UrlDecode => "URL: Percent-Decode",
            TransformOp::HtmlEscape => "HTML: Escape Entities",
            TransformOp::HtmlUnescape => "HTML: Unescape Entities",
            TransformOp::JwtDecode => "JWT: Inspect Header & Payload",
            TransformOp::HashMd5 => "Hash: Compute MD5",
            TransformOp::HashSha256 => "Hash: Compute SHA-256",
            TransformOp::HashSha512 => "Hash: Compute SHA-512",
            TransformOp::UnixTimestampToIso => "Time: Unix Timestamp → ISO-8601 UTC",
            TransformOp::InsertCurrentIsoUtc => "Time: Current ISO-8601 UTC",
        }
    }
}

/// Apply any built-in transform to the given input string.
pub fn apply_transform(input: &str, op: TransformOp) -> Result<String, String> {
    match op {
        TransformOp::SortLinesAsc => {
            let mut lines: Vec<&str> = input.lines().collect();
            lines.sort();
            Ok(lines.join("\n"))
        }
        TransformOp::SortLinesDesc => {
            let mut lines: Vec<&str> = input.lines().collect();
            lines.sort_by(|a, b| b.cmp(a));
            Ok(lines.join("\n"))
        }
        TransformOp::SortLinesNumeric => {
            let mut lines: Vec<&str> = input.lines().collect();
            lines.sort_by(|a, b| {
                let fa = leading_number(a);
                let fb = leading_number(b);
                fa.partial_cmp(&fb)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.cmp(b))
            });
            Ok(lines.join("\n"))
        }
        TransformOp::SortLinesByLength => {
            let mut lines: Vec<&str> = input.lines().collect();
            lines.sort_by(|a, b| {
                a.chars()
                    .count()
                    .cmp(&b.chars().count())
                    .then_with(|| a.cmp(b))
            });
            Ok(lines.join("\n"))
        }
        TransformOp::ReverseLines => {
            let mut lines: Vec<&str> = input.lines().collect();
            lines.reverse();
            Ok(lines.join("\n"))
        }
        TransformOp::DeduplicateLines => {
            let mut seen = HashSet::new();
            let lines: Vec<&str> = input.lines().filter(|line| seen.insert(*line)).collect();
            Ok(lines.join("\n"))
        }
        TransformOp::RemoveEmptyLines => {
            let lines: Vec<&str> = input.lines().filter(|l| !l.is_empty()).collect();
            Ok(lines.join("\n"))
        }
        TransformOp::RemoveWhitespaceOnlyLines => {
            let lines: Vec<&str> = input.lines().filter(|l| !l.trim().is_empty()).collect();
            Ok(lines.join("\n"))
        }
        TransformOp::TrimTrailingWhitespace => {
            let lines: Vec<&str> = input.lines().map(str::trim_end).collect();
            Ok(lines.join("\n"))
        }
        TransformOp::TrimLeadingWhitespace => {
            let lines: Vec<&str> = input.lines().map(str::trim_start).collect();
            Ok(lines.join("\n"))
        }
        TransformOp::TrimBothWhitespace => {
            let lines: Vec<&str> = input.lines().map(str::trim).collect();
            Ok(lines.join("\n"))
        }
        TransformOp::AddLineNumbers => {
            let total = input.lines().count().max(1);
            let width = total.to_string().len();
            let lines: Vec<String> = input
                .lines()
                .enumerate()
                .map(|(i, l)| format!("{:>width$}: {l}", i + 1))
                .collect();
            Ok(lines.join("\n"))
        }
        TransformOp::JoinLinesWithSpace => {
            let joined: Vec<&str> = input
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect();
            Ok(joined.join(" "))
        }
        TransformOp::JoinLinesWithComma => {
            let joined: Vec<&str> = input
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect();
            Ok(joined.join(", "))
        }
        TransformOp::Uppercase => Ok(input.to_uppercase()),
        TransformOp::Lowercase => Ok(input.to_lowercase()),
        TransformOp::TitleCase => Ok(to_title_case(input)),
        TransformOp::CamelCase => Ok(transform_lines(input, to_camel_case)),
        TransformOp::SnakeCase => Ok(transform_lines(input, |s| to_delimited_case(s, '_'))),
        TransformOp::KebabCase => Ok(transform_lines(input, |s| to_delimited_case(s, '-'))),
        TransformOp::InvertCase => Ok(input
            .chars()
            .flat_map(|c| {
                if c.is_uppercase() {
                    c.to_lowercase().collect::<Vec<_>>()
                } else if c.is_lowercase() {
                    c.to_uppercase().collect::<Vec<_>>()
                } else {
                    vec![c]
                }
            })
            .collect()),
        TransformOp::JsonPretty => {
            let value: serde_json::Value =
                serde_json::from_str(input).map_err(|e| format!("Invalid JSON: {e}"))?;
            serde_json::to_string_pretty(&value).map_err(|e| format!("JSON format error: {e}"))
        }
        TransformOp::JsonMinify => {
            let value: serde_json::Value =
                serde_json::from_str(input).map_err(|e| format!("Invalid JSON: {e}"))?;
            serde_json::to_string(&value).map_err(|e| format!("JSON minify error: {e}"))
        }
        TransformOp::XmlPretty => format_xml(input, true),
        TransformOp::XmlMinify => format_xml(input, false),
        TransformOp::Base64Encode => Ok(STANDARD.encode(input.as_bytes())),
        TransformOp::Base64Decode => {
            let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();
            let bytes = STANDARD
                .decode(&cleaned)
                .or_else(|_| URL_SAFE_NO_PAD.decode(&cleaned))
                .map_err(|e| format!("Invalid Base64: {e}"))?;
            String::from_utf8(bytes).map_err(|e| format!("Decoded bytes are not valid UTF-8: {e}"))
        }
        TransformOp::UrlEncode => Ok(urlencoding::encode(input).into_owned()),
        TransformOp::UrlDecode => urlencoding::decode(input)
            .map(|c| c.into_owned())
            .map_err(|e| format!("Invalid URL encoding: {e}")),
        TransformOp::HtmlEscape => Ok(html_escape::encode_text(input).into_owned()),
        TransformOp::HtmlUnescape => Ok(html_escape::decode_html_entities(input).into_owned()),
        TransformOp::JwtDecode => decode_jwt(input),
        TransformOp::HashMd5 => Ok(hex::encode(Md5::digest(input.as_bytes()))),
        TransformOp::HashSha256 => Ok(hex::encode(Sha256::digest(input.as_bytes()))),
        TransformOp::HashSha512 => Ok(hex::encode(Sha512::digest(input.as_bytes()))),
        TransformOp::UnixTimestampToIso => convert_unix_timestamp(input),
        TransformOp::InsertCurrentIsoUtc => Ok(Utc::now().to_rfc3339()),
    }
}

/// Filter lines containing (or excluding) a query substring or regex.
pub fn filter_lines(
    input: &str,
    pattern: &str,
    use_regex: bool,
    invert: bool,
) -> Result<String, String> {
    if pattern.is_empty() {
        return Ok(input.to_owned());
    }
    if use_regex {
        let re = regex::Regex::new(pattern).map_err(|e| format!("Invalid regex: {e}"))?;
        let kept: Vec<&str> = input
            .lines()
            .filter(|line| re.is_match(line) ^ invert)
            .collect();
        Ok(kept.join("\n"))
    } else {
        let kept: Vec<&str> = input
            .lines()
            .filter(|line| line.contains(pattern) ^ invert)
            .collect();
        Ok(kept.join("\n"))
    }
}

/// Extract or delete bookmarked lines (0-based indices).
pub fn extract_bookmarked_lines(input: &str, bookmarks: &[usize]) -> String {
    let set: HashSet<usize> = bookmarks.iter().copied().collect();
    input
        .lines()
        .enumerate()
        .filter(|(idx, _)| set.contains(idx))
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn delete_bookmarked_lines(input: &str, bookmarks: &[usize]) -> String {
    let set: HashSet<usize> = bookmarks.iter().copied().collect();
    input
        .lines()
        .enumerate()
        .filter(|(idx, _)| !set.contains(idx))
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn leading_number(line: &str) -> f64 {
    let trimmed = line.trim_start();
    let num_str: String = trimmed
        .chars()
        .enumerate()
        .take_while(|&(i, c)| c.is_ascii_digit() || c == '.' || (i == 0 && (c == '-' || c == '+')))
        .map(|(_, c)| c)
        .collect();
    num_str.parse::<f64>().unwrap_or(f64::INFINITY)
}

fn to_title_case(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut start_of_word = true;
    for ch in input.chars() {
        if ch.is_alphanumeric() {
            if start_of_word {
                for u in ch.to_uppercase() {
                    out.push(u);
                }
                start_of_word = false;
            } else {
                for l in ch.to_lowercase() {
                    out.push(l);
                }
            }
        } else {
            out.push(ch);
            start_of_word = true;
        }
    }
    out
}

fn transform_lines(input: &str, f: impl Fn(&str) -> String) -> String {
    input.lines().map(f).collect::<Vec<_>>().join("\n")
}

fn split_identifier_words(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut prev_lower = false;

    for ch in input.chars() {
        if !ch.is_alphanumeric() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            prev_lower = false;
        } else {
            if ch.is_uppercase() && prev_lower && !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            for l in ch.to_lowercase() {
                current.push(l);
            }
            prev_lower = ch.is_lowercase() || ch.is_ascii_digit();
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

fn to_camel_case(input: &str) -> String {
    let words = split_identifier_words(input);
    let mut out = String::new();
    for (i, w) in words.into_iter().enumerate() {
        if i == 0 {
            out.push_str(&w);
        } else {
            let mut chars = w.chars();
            if let Some(first) = chars.next() {
                for u in first.to_uppercase() {
                    out.push(u);
                }
                out.push_str(chars.as_str());
            }
        }
    }
    out
}

fn to_delimited_case(input: &str, sep: char) -> String {
    split_identifier_words(input).join(&sep.to_string())
}

fn format_xml(input: &str, pretty: bool) -> Result<String, String> {
    let mut reader = Reader::from_str(input);
    reader.config_mut().trim_text(true);

    let mut writer = if pretty {
        Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2)
    } else {
        Writer::new(Cursor::new(Vec::new()))
    };

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(ev) => writer
                .write_event(ev)
                .map_err(|e| format!("XML write error: {e}"))?,
            Err(e) => {
                return Err(format!(
                    "Invalid XML at byte {}: {e}",
                    reader.error_position()
                ));
            }
        }
    }

    let bytes = writer.into_inner().into_inner();
    String::from_utf8(bytes).map_err(|e| format!("UTF-8 error in XML output: {e}"))
}

fn decode_jwt(input: &str) -> Result<String, String> {
    let token = input.trim();
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return Err("Expected a JWT with at least header.payload segments".into());
    }

    let decode_seg = |seg: &str, name: &str| -> Result<serde_json::Value, String> {
        let bytes = URL_SAFE_NO_PAD
            .decode(seg)
            .or_else(|_| STANDARD.decode(seg))
            .map_err(|e| format!("Invalid Base64 in JWT {name}: {e}"))?;
        let text = String::from_utf8(bytes).map_err(|e| format!("Non-UTF-8 JWT {name}: {e}"))?;
        serde_json::from_str(&text).map_err(|e| format!("Invalid JSON in JWT {name}: {e}"))
    };

    let header = decode_seg(parts[0], "header")?;
    let payload = decode_seg(parts[1], "payload")?;
    let signature = parts.get(2).copied().unwrap_or("");

    let summary = serde_json::json!({
        "header": header,
        "payload": payload,
        "signature_present": !signature.is_empty(),
    });
    serde_json::to_string_pretty(&summary).map_err(|e| format!("JWT format error: {e}"))
}

fn convert_unix_timestamp(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    let raw: i64 = trimmed
        .parse()
        .map_err(|_| format!("'{trimmed}' is not an integer Unix timestamp"))?;
    let dt: DateTime<Utc> = if raw.abs() > 100_000_000_000 {
        DateTime::from_timestamp_millis(raw).ok_or_else(|| "Timestamp out of range".to_string())?
    } else {
        DateTime::from_timestamp(raw, 0).ok_or_else(|| "Timestamp out of range".to_string())?
    };
    Ok(dt.to_rfc3339())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_surgery_and_case_transforms_work() {
        assert_eq!(
            apply_transform("b\na\nc\na", TransformOp::DeduplicateLines).unwrap(),
            "b\na\nc"
        );
        assert_eq!(
            apply_transform("10 items\n2 items\n1 item", TransformOp::SortLinesNumeric).unwrap(),
            "1 item\n2 items\n10 items"
        );
        assert_eq!(
            apply_transform("hello_world_test", TransformOp::CamelCase).unwrap(),
            "helloWorldTest"
        );
        assert_eq!(
            apply_transform("helloWorldTest", TransformOp::SnakeCase).unwrap(),
            "hello_world_test"
        );
        assert_eq!(
            apply_transform("helloWorldTest", TransformOp::KebabCase).unwrap(),
            "hello-world-test"
        );
    }

    #[test]
    fn developer_tools_json_xml_base64_jwt_hash_work() {
        let pretty = apply_transform(r#"{"b":1,"a":[2,3]}"#, TransformOp::JsonPretty).unwrap();
        assert!(pretty.contains('\n'));
        assert_eq!(
            apply_transform(&pretty, TransformOp::JsonMinify).unwrap(),
            r#"{"a":[2,3],"b":1}"#
        );

        let xml_pretty =
            apply_transform("<root><item>42</item></root>", TransformOp::XmlPretty).unwrap();
        assert!(xml_pretty.contains('\n'));
        assert_eq!(
            apply_transform(&xml_pretty, TransformOp::XmlMinify).unwrap(),
            "<root><item>42</item></root>"
        );

        let b64 = apply_transform("Aetheris Editor", TransformOp::Base64Encode).unwrap();
        assert_eq!(
            apply_transform(&b64, TransformOp::Base64Decode).unwrap(),
            "Aetheris Editor"
        );

        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkFldGhlcmlzIn0.sig";
        let decoded = apply_transform(jwt, TransformOp::JwtDecode).unwrap();
        assert!(decoded.contains("Aetheris"));
        assert!(decoded.contains("HS256"));

        assert_eq!(
            apply_transform("hello", TransformOp::HashSha256).unwrap(),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
        assert_eq!(
            apply_transform("1700000000", TransformOp::UnixTimestampToIso).unwrap(),
            "2023-11-14T22:13:20+00:00"
        );
    }

    #[test]
    fn bookmark_extraction_and_deletion_work() {
        let doc = "zero\none\ntwo\nthree";
        assert_eq!(extract_bookmarked_lines(doc, &[1, 3]), "one\nthree");
        assert_eq!(delete_bookmarked_lines(doc, &[1, 3]), "zero\ntwo");
    }
}
