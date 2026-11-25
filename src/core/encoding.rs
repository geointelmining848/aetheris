use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use encoding_rs::{EUC_KR, Encoding, GB18030, SHIFT_JIS, UTF_8, UTF_16BE, UTF_16LE, WINDOWS_1252};
use serde::{Deserialize, Serialize};

/// Supported text character encodings for lossless open, inspect, and conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TextEncoding {
    #[default]
    Utf8,
    Utf8Bom,
    Utf16Le,
    Utf16Be,
    Windows1252,
    Iso88591,
    Gb18030,
    ShiftJis,
    EucKr,
}

impl TextEncoding {
    pub const ALL: &'static [TextEncoding] = &[
        TextEncoding::Utf8,
        TextEncoding::Utf8Bom,
        TextEncoding::Utf16Le,
        TextEncoding::Utf16Be,
        TextEncoding::Windows1252,
        TextEncoding::Iso88591,
        TextEncoding::Gb18030,
        TextEncoding::ShiftJis,
        TextEncoding::EucKr,
    ];

    pub fn label(self) -> &'static str {
        match self {
            TextEncoding::Utf8 => "UTF-8",
            TextEncoding::Utf8Bom => "UTF-8 BOM",
            TextEncoding::Utf16Le => "UTF-16 LE",
            TextEncoding::Utf16Be => "UTF-16 BE",
            TextEncoding::Windows1252 => "Windows-1252",
            TextEncoding::Iso88591 => "ISO-8859-1",
            TextEncoding::Gb18030 => "GB18030",
            TextEncoding::ShiftJis => "Shift-JIS",
            TextEncoding::EucKr => "EUC-KR",
        }
    }

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&e| e == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    fn encoding_rs(self) -> &'static Encoding {
        match self {
            TextEncoding::Utf8 | TextEncoding::Utf8Bom => UTF_8,
            TextEncoding::Utf16Le => UTF_16LE,
            TextEncoding::Utf16Be => UTF_16BE,
            TextEncoding::Windows1252 | TextEncoding::Iso88591 => WINDOWS_1252,
            TextEncoding::Gb18030 => GB18030,
            TextEncoding::ShiftJis => SHIFT_JIS,
            TextEncoding::EucKr => EUC_KR,
        }
    }
}

impl fmt::Display for TextEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Supported line ending formats (LF, CRLF, CR).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum LineEnding {
    #[default]
    Lf,
    CrLf,
    Cr,
}

impl LineEnding {
    pub const ALL: &'static [LineEnding] = &[LineEnding::Lf, LineEnding::CrLf, LineEnding::Cr];

    pub fn label(self) -> &'static str {
        match self {
            LineEnding::Lf => "LF (Unix)",
            LineEnding::CrLf => "CRLF (Windows)",
            LineEnding::Cr => "CR (Classic Mac)",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            LineEnding::Lf => "LF",
            LineEnding::CrLf => "CRLF",
            LineEnding::Cr => "CR",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::CrLf => "\r\n",
            LineEnding::Cr => "\r",
        }
    }

    pub fn next(self) -> Self {
        match self {
            LineEnding::Lf => LineEnding::CrLf,
            LineEnding::CrLf => LineEnding::Cr,
            LineEnding::Cr => LineEnding::Lf,
        }
    }

    pub fn platform_default() -> Self {
        if cfg!(windows) {
            LineEnding::CrLf
        } else {
            LineEnding::Lf
        }
    }
}

impl fmt::Display for LineEnding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.short_label())
    }
}

/// Result of decoding raw file bytes into normalized LF text plus detected metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedText {
    /// Text normalized to `\n` for internal editing.
    pub text: String,
    pub encoding: TextEncoding,
    pub line_ending: LineEnding,
    pub had_errors: bool,
    pub looks_binary: bool,
}

/// Inspect raw bytes and detect whether a file looks binary (e.g., NUL bytes in non-UTF-16 data).
pub fn looks_binary(bytes: &[u8]) -> bool {
    if bytes.starts_with(&[0xFF, 0xFE]) || bytes.starts_with(&[0xFE, 0xFF]) {
        return false;
    }
    let sample_len = bytes.len().min(8192);
    let sample = &bytes[..sample_len];
    if sample.is_empty() {
        return false;
    }
    let nul_count = sample.iter().filter(|&&b| b == 0).count();
    if nul_count > 0 {
        return true;
    }
    let control_count = sample
        .iter()
        .filter(|&&b| b < 0x09 || (b > 0x0D && b < 0x20 && b != 0x1B))
        .count();
    control_count * 10 > sample_len
}

/// Detect the character encoding of raw bytes using BOM signatures, UTF-8 validity, and `chardetng`.
pub fn detect_encoding(bytes: &[u8]) -> TextEncoding {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return TextEncoding::Utf8Bom;
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return TextEncoding::Utf16Le;
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return TextEncoding::Utf16Be;
    }
    if std::str::from_utf8(bytes).is_ok() {
        return TextEncoding::Utf8;
    }

    let mut detector = chardetng::EncodingDetector::new(chardetng::Iso2022JpDetection::Deny);
    let sample_len = bytes.len().min(65536);
    detector.feed(&bytes[..sample_len], sample_len == bytes.len());
    let guessed = detector.guess(None, chardetng::Utf8Detection::Allow);

    if guessed == GB18030 {
        TextEncoding::Gb18030
    } else if guessed == SHIFT_JIS {
        TextEncoding::ShiftJis
    } else if guessed == EUC_KR {
        TextEncoding::EucKr
    } else if guessed == UTF_16LE {
        TextEncoding::Utf16Le
    } else if guessed == UTF_16BE {
        TextEncoding::Utf16Be
    } else if guessed == UTF_8 {
        TextEncoding::Utf8
    } else {
        TextEncoding::Windows1252
    }
}

/// Detect dominant line ending in decoded text.
pub fn detect_line_ending(text: &str) -> LineEnding {
    let mut crlf = 0usize;
    let mut lf = 0usize;
    let mut cr = 0usize;

    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\r' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    crlf += 1;
                    i += 2;
                } else {
                    cr += 1;
                    i += 1;
                }
            }
            b'\n' => {
                lf += 1;
                i += 1;
            }
            _ => i += 1,
        }
    }

    if crlf == 0 && lf == 0 && cr == 0 {
        LineEnding::platform_default()
    } else if crlf >= lf && crlf >= cr {
        LineEnding::CrLf
    } else if lf >= cr {
        LineEnding::Lf
    } else {
        LineEnding::Cr
    }
}

/// Normalize all line endings to `\n` for uniform internal editor handling.
pub fn normalize_to_lf(text: &str) -> String {
    if !text.contains('\r') {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            out.push('\n');
        } else {
            out.push(ch);
        }
    }
    out
}

/// Convert LF-normalized text to the requested line ending format.
pub fn apply_line_ending(lf_text: &str, line_ending: LineEnding) -> String {
    match line_ending {
        LineEnding::Lf => normalize_to_lf(lf_text),
        LineEnding::CrLf => normalize_to_lf(lf_text).replace('\n', "\r\n"),
        LineEnding::Cr => normalize_to_lf(lf_text).replace('\n', "\r"),
    }
}

/// Auto-detect encoding and decode raw bytes into normalized text.
pub fn decode_bytes(bytes: &[u8]) -> DecodedText {
    let binary = looks_binary(bytes);
    let encoding = detect_encoding(bytes);
    let (raw_text, had_errors) = decode_with_encoding(bytes, encoding);
    let line_ending = detect_line_ending(&raw_text);
    let text = normalize_to_lf(&raw_text);
    DecodedText {
        text,
        encoding,
        line_ending,
        had_errors,
        looks_binary: binary,
    }
}

/// Decode raw bytes with an explicitly requested character encoding.
pub fn decode_with_encoding(bytes: &[u8], encoding: TextEncoding) -> (String, bool) {
    match encoding {
        TextEncoding::Utf8 => {
            let (cow, _, had_errors) = UTF_8.decode(bytes);
            (cow.into_owned(), had_errors)
        }
        TextEncoding::Utf8Bom => {
            let slice = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
            let (cow, _, had_errors) = UTF_8.decode(slice);
            (cow.into_owned(), had_errors)
        }
        TextEncoding::Utf16Le => {
            let slice = bytes.strip_prefix(&[0xFF, 0xFE]).unwrap_or(bytes);
            let (cow, had_errors) = UTF_16LE.decode_without_bom_handling(slice);
            (cow.into_owned(), had_errors)
        }
        TextEncoding::Utf16Be => {
            let slice = bytes.strip_prefix(&[0xFE, 0xFF]).unwrap_or(bytes);
            let (cow, had_errors) = UTF_16BE.decode_without_bom_handling(slice);
            (cow.into_owned(), had_errors)
        }
        TextEncoding::Iso88591 => {
            let s: String = bytes.iter().map(|&b| b as char).collect();
            (s, false)
        }
        _ => {
            let (cow, _, had_errors) = encoding.encoding_rs().decode(bytes);
            (cow.into_owned(), had_errors)
        }
    }
}

/// Encode LF-normalized text to raw bytes with the requested encoding and line endings.
pub fn encode_text(lf_text: &str, encoding: TextEncoding, line_ending: LineEnding) -> Vec<u8> {
    let converted = apply_line_ending(lf_text, line_ending);
    match encoding {
        TextEncoding::Utf8 => converted.into_bytes(),
        TextEncoding::Utf8Bom => {
            let mut out = Vec::with_capacity(3 + converted.len());
            out.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
            out.extend_from_slice(converted.as_bytes());
            out
        }
        TextEncoding::Utf16Le => {
            let mut out = Vec::with_capacity(2 + converted.len() * 2);
            out.extend_from_slice(&[0xFF, 0xFE]);
            for unit in converted.encode_utf16() {
                out.extend_from_slice(&unit.to_le_bytes());
            }
            out
        }
        TextEncoding::Utf16Be => {
            let mut out = Vec::with_capacity(2 + converted.len() * 2);
            out.extend_from_slice(&[0xFE, 0xFF]);
            for unit in converted.encode_utf16() {
                out.extend_from_slice(&unit.to_be_bytes());
            }
            out
        }
        TextEncoding::Iso88591 => converted
            .chars()
            .map(|c| {
                let code = c as u32;
                if code <= 0xFF { code as u8 } else { b'?' }
            })
            .collect(),
        _ => {
            let (cow, _, _) = encoding.encoding_rs().encode(&converted);
            cow.into_owned()
        }
    }
}

/// Read and decode a file from disk with auto-detected encoding and line endings.
pub fn read_file_decoded(path: &Path) -> io::Result<DecodedText> {
    let bytes = fs::read(path)?;
    Ok(decode_bytes(&bytes))
}

/// Re-read a file from disk forcing a specific encoding.
pub fn read_file_with_encoding(path: &Path, encoding: TextEncoding) -> io::Result<DecodedText> {
    let bytes = fs::read(path)?;
    let binary = looks_binary(&bytes);
    let (raw_text, had_errors) = decode_with_encoding(&bytes, encoding);
    let line_ending = detect_line_ending(&raw_text);
    let text = normalize_to_lf(&raw_text);
    Ok(DecodedText {
        text,
        encoding,
        line_ending,
        had_errors,
        looks_binary: binary,
    })
}

/// Encode and write a document to disk atomically (best-effort direct write).
pub fn write_file_encoded(
    path: &Path,
    lf_text: &str,
    encoding: TextEncoding,
    line_ending: LineEnding,
) -> io::Result<()> {
    let bytes = encode_text(lf_text, encoding, line_ending);
    fs::write(path, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_utf8_bom_and_crlf() {
        let original = "line1\nline2\n你好 Aetheris";
        let bytes = encode_text(original, TextEncoding::Utf8Bom, LineEnding::CrLf);
        assert!(bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
        let decoded = decode_bytes(&bytes);
        assert_eq!(decoded.encoding, TextEncoding::Utf8Bom);
        assert_eq!(decoded.line_ending, LineEnding::CrLf);
        assert_eq!(decoded.text, original);
        assert!(!decoded.had_errors);
    }

    #[test]
    fn roundtrip_utf16_le_and_be() {
        let original = "Hello\nWorld — αβγ";
        for enc in [TextEncoding::Utf16Le, TextEncoding::Utf16Be] {
            let bytes = encode_text(original, enc, LineEnding::Lf);
            let decoded = decode_bytes(&bytes);
            assert_eq!(decoded.encoding, enc);
            assert_eq!(decoded.line_ending, LineEnding::Lf);
            assert_eq!(decoded.text, original);
            assert!(!decoded.looks_binary);
        }
    }

    #[test]
    fn roundtrip_gb18030_and_shift_jis() {
        let zh = "中文编码测试 Aetheris";
        let bytes_gb = encode_text(zh, TextEncoding::Gb18030, LineEnding::Lf);
        let (decoded_gb, err_gb) = decode_with_encoding(&bytes_gb, TextEncoding::Gb18030);
        assert!(!err_gb);
        assert_eq!(decoded_gb, zh);

        let jp = "日本語テスト";
        let bytes_sjis = encode_text(jp, TextEncoding::ShiftJis, LineEnding::CrLf);
        let (decoded_sjis, err_sjis) = decode_with_encoding(&bytes_sjis, TextEncoding::ShiftJis);
        assert!(!err_sjis);
        assert_eq!(normalize_to_lf(&decoded_sjis), jp);
    }

    #[test]
    fn detects_line_endings_and_binary() {
        assert_eq!(detect_line_ending("a\r\nb\r\nc"), LineEnding::CrLf);
        assert_eq!(detect_line_ending("a\nb\nc"), LineEnding::Lf);
        assert_eq!(detect_line_ending("a\rb\rc"), LineEnding::Cr);
        assert!(looks_binary(&[0x89, 0x50, 0x4E, 0x47, 0x00, 0x01]));
        assert!(!looks_binary(b"plain text file\nwith lines\n"));
    }
}
