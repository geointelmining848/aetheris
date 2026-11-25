use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use memmap2::Mmap;

/// Files above this byte threshold automatically default to Fast View (`memmap2`) mode.
pub const LARGE_FILE_THRESHOLD_BYTES: u64 = 10 * 1024 * 1024; // 10 MiB

/// Default window size per Fast View page (64 KiB).
pub const DEFAULT_CHUNK_BYTES: usize = 64 * 1024;

/// A single row in the 16-byte Hex / ASCII inspector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexRow {
    pub offset: usize,
    pub hex_columns: String,
    pub ascii_column: String,
}

/// Memory-mapped chunk reader for instant inspection of multi-hundred-MB / multi-GB files.
#[derive(Debug)]
pub struct LargeFileReader {
    path: PathBuf,
    file_len: u64,
    mmap: Option<Mmap>,
}

impl LargeFileReader {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        let file_len = file.metadata()?.len();
        let mmap = if file_len == 0 {
            None
        } else {
            // SAFETY: Read-only mapping for chunked viewing; file length verified > 0.
            Some(unsafe { Mmap::map(&file)? })
        };
        Ok(Self {
            path,
            file_len,
            mmap,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn len(&self) -> u64 {
        self.file_len
    }

    pub fn is_empty(&self) -> bool {
        self.file_len == 0
    }

    /// Read a byte slice starting at `offset` up to `len` bytes.
    pub fn slice(&self, offset: usize, len: usize) -> &[u8] {
        let Some(ref map) = self.mmap else {
            return &[];
        };
        if offset >= map.len() {
            return &[];
        }
        let end = offset.saturating_add(len).min(map.len());
        &map[offset..end]
    }

    /// Read a UTF-8 lossy text chunk aligned cleanly to newline boundaries when possible.
    pub fn read_text_window(&self, offset: usize, max_bytes: usize) -> (usize, usize, String) {
        let Some(ref map) = self.mmap else {
            return (0, 0, String::new());
        };
        if offset >= map.len() {
            return (map.len(), map.len(), String::new());
        }
        let mut start = offset;
        if start > 0
            && map[start - 1] != b'\n'
            && let Some(nl) = map[start..].iter().position(|&b| b == b'\n')
            && nl < 4096
        {
            start += nl + 1;
        }
        let raw_end = start.saturating_add(max_bytes).min(map.len());
        let end = if raw_end < map.len() {
            map[start..raw_end]
                .iter()
                .rposition(|&b| b == b'\n')
                .map(|pos| start + pos + 1)
                .unwrap_or(raw_end)
        } else {
            raw_end
        };
        let text = String::from_utf8_lossy(&map[start..end]).into_owned();
        (start, end, text)
    }

    /// Streaming case-sensitive substring search starting from `from_offset` across the mapped file.
    pub fn find_next_bytes(&self, needle: &[u8], from_offset: usize) -> Option<usize> {
        let map = self.mmap.as_ref()?;
        if needle.is_empty() || from_offset >= map.len() || needle.len() > map.len() {
            return None;
        }
        map[from_offset..]
            .windows(needle.len())
            .position(|w| w == needle)
            .map(|rel| from_offset + rel)
    }

    /// Render a 16-bytes-per-row Hex + ASCII view for `[offset .. offset + len]`.
    pub fn hex_rows(&self, offset: usize, max_bytes: usize) -> Vec<HexRow> {
        format_hex_dump(self.slice(offset, max_bytes), offset)
    }
}

/// Format any byte slice (in-memory buffer or mmap chunk) into aligned 16-byte Hex + ASCII rows.
pub fn format_hex_dump(bytes: &[u8], base_offset: usize) -> Vec<HexRow> {
    let mut rows = Vec::with_capacity(bytes.len().div_ceil(16));
    for (chunk_idx, chunk) in bytes.chunks(16).enumerate() {
        let offset = base_offset + chunk_idx * 16;
        let mut hex_str = String::with_capacity(50);
        let mut ascii_str = String::with_capacity(16);

        for i in 0..16 {
            if i == 8 {
                hex_str.push(' ');
            }
            if let Some(&b) = chunk.get(i) {
                hex_str.push_str(&format!("{b:02X} "));
                let ch = if (0x20..=0x7E).contains(&b) {
                    b as char
                } else {
                    '.'
                };
                ascii_str.push(ch);
            } else {
                hex_str.push_str("   ");
                ascii_str.push(' ');
            }
        }

        rows.push(HexRow {
            offset,
            hex_columns: hex_str.trim_end().to_owned(),
            ascii_column: ascii_str,
        });
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn memory_mapped_reader_paginates_searches_and_dumps_hex() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        for i in 0..500 {
            writeln!(tmp, "2026-11-25 INFO record number {i:04}").unwrap();
        }
        tmp.flush().unwrap();

        let reader = LargeFileReader::open(tmp.path()).unwrap();
        assert!(reader.len() > 10_000);

        let (start, end, chunk) = reader.read_text_window(0, 1024);
        assert_eq!(start, 0);
        assert!(end <= 1024);
        assert!(chunk.contains("record number 0000"));

        let hit = reader.find_next_bytes(b"record number 0420", 0).unwrap();
        let (_, _, around) = reader.read_text_window(hit.saturating_sub(32), 256);
        assert!(around.contains("record number 0420"));

        let hex_rows = reader.hex_rows(0, 32);
        assert_eq!(hex_rows.len(), 2);
        assert_eq!(hex_rows[0].offset, 0);
        assert!(hex_rows[0].ascii_column.starts_with("2026-11-25"));
    }
}
