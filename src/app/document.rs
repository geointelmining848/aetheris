use std::collections::BTreeSet;
use std::path::PathBuf;

use iced::widget::{markdown, text_editor};

use crate::core::csv_view::{CsvTable, parse_csv};
use crate::core::diff::DiffReport;
use crate::core::encoding::{LineEnding, TextEncoding};
use crate::core::language::Language;
use crate::core::large_file::{DEFAULT_CHUNK_BYTES, HexRow, LargeFileReader, format_hex_dump};

/// Viewport presentation mode for a single tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Editor,
    MarkdownSplit,
    DiffView,
    CsvGrid,
    HexView,
    LargeFileFastView,
}

impl ViewMode {
    pub fn label(self) -> &'static str {
        match self {
            ViewMode::Editor => "Code",
            ViewMode::MarkdownSplit => "Markdown",
            ViewMode::DiffView => "Diff",
            ViewMode::CsvGrid => "CSV",
            ViewMode::HexView => "Hex",
            ViewMode::LargeFileFastView => "Fast View",
        }
    }
}

/// Complete live state for one open tab in Aetheris.
pub struct DocumentTab {
    pub id: u64,
    pub title: String,
    pub path: Option<PathBuf>,
    pub content: text_editor::Content,
    pub encoding: TextEncoding,
    pub line_ending: LineEnding,
    pub language: Language,
    /// When `false`, Aetheris automatically infers syntax language live from buffer content.
    pub language_locked: bool,
    pub is_dirty: bool,
    pub view_mode: ViewMode,
    pub bookmarks: BTreeSet<usize>,

    // Cached Markdown preview AST
    pub markdown_items: Vec<markdown::Item>,

    // Cached CSV / TSV table state
    pub csv_table: Option<CsvTable>,
    pub csv_error: Option<String>,
    pub csv_filter: String,
    pub csv_sort_col: Option<usize>,
    pub csv_sort_asc: bool,

    // Hex inspector state
    pub hex_offset: usize,
    pub hex_rows: Vec<HexRow>,

    // Side-by-side Diff state
    pub diff_target_id: Option<u64>,
    pub diff_report: Option<DiffReport>,

    // Memory-mapped Large-File Fast View state
    pub large_file: Option<LargeFileReader>,
    pub large_file_offset: usize,
    pub large_file_window: (usize, usize),
    pub large_file_chunk: String,
}

impl DocumentTab {
    pub fn new_scratch(id: u64, title: String, initial_text: &str, language: Language) -> Self {
        let mut tab = Self {
            id,
            title,
            path: None,
            content: text_editor::Content::with_text(initial_text),
            encoding: TextEncoding::Utf8,
            line_ending: LineEnding::platform_default(),
            language,
            language_locked: language != Language::PlainText,
            is_dirty: false,
            view_mode: ViewMode::Editor,
            bookmarks: BTreeSet::new(),
            markdown_items: Vec::new(),
            csv_table: None,
            csv_error: None,
            csv_filter: String::new(),
            csv_sort_col: None,
            csv_sort_asc: true,
            hex_offset: 0,
            hex_rows: Vec::new(),
            diff_target_id: None,
            diff_report: None,
            large_file: None,
            large_file_offset: 0,
            large_file_window: (0, 0),
            large_file_chunk: String::new(),
        };
        if language == Language::Markdown {
            tab.refresh_markdown();
        }
        tab
    }

    pub fn from_decoded_file(
        id: u64,
        path: PathBuf,
        text: &str,
        encoding: TextEncoding,
        line_ending: LineEnding,
        looks_binary: bool,
    ) -> Self {
        let title = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_owned();
        let mut language = Language::detect_from_path(&path);
        let locked = language != Language::PlainText;
        if !locked && let Some(detected) = Language::detect_from_content(text) {
            language = detected;
        }
        let mut tab = Self::new_scratch(id, title, text, language);
        tab.language_locked = locked;
        tab.path = Some(path);
        tab.encoding = encoding;
        tab.line_ending = line_ending;
        if looks_binary {
            tab.view_mode = ViewMode::HexView;
            tab.refresh_hex();
        }
        tab
    }

    pub fn from_large_file(id: u64, reader: LargeFileReader) -> Self {
        let path = reader.path().to_path_buf();
        let title = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Large File")
            .to_owned();
        let language = Language::detect_from_path(&path);
        let mut tab = Self::new_scratch(id, title, "", language);
        tab.path = Some(path);
        tab.large_file = Some(reader);
        tab.view_mode = ViewMode::LargeFileFastView;
        tab.refresh_large_file_chunk();
        tab
    }

    pub fn text(&self) -> String {
        if self.view_mode == ViewMode::LargeFileFastView {
            self.large_file_chunk.clone()
        } else {
            self.content.text()
        }
    }

    pub fn replace_full_text(&mut self, new_text: &str) {
        let prev_cursor = self.content.cursor();
        self.content = text_editor::Content::with_text(new_text);
        let max_line = self.content.line_count().saturating_sub(1);
        let target_line = prev_cursor.position.line.min(max_line);
        self.content.move_to(text_editor::Cursor {
            position: text_editor::Position {
                line: target_line,
                column: 0,
            },
            selection: None,
        });
        self.is_dirty = true;
        self.auto_detect_language();
        self.refresh_active_view_cache();
    }

    /// Infer and update syntax language live from content when not explicitly locked by extension or user.
    pub fn auto_detect_language(&mut self) -> Option<Language> {
        if self.language_locked {
            return None;
        }
        let detected =
            Language::detect_from_content(&self.content.text()).unwrap_or(Language::PlainText);
        if detected != self.language {
            self.language = detected;
            return Some(detected);
        }
        None
    }

    pub fn jump_to_line(&mut self, one_based_line: usize) {
        let line_idx = one_based_line
            .saturating_sub(1)
            .min(self.content.line_count().saturating_sub(1));
        self.content.move_to(text_editor::Cursor {
            position: text_editor::Position {
                line: line_idx,
                column: 0,
            },
            selection: None,
        });
    }

    pub fn cursor_line_col(&self) -> (usize, usize) {
        let pos = self.content.cursor().position;
        (pos.line + 1, pos.column + 1)
    }

    pub fn line_count(&self) -> usize {
        self.content.line_count()
    }

    pub fn toggle_bookmark_at_cursor(&mut self) -> (usize, bool) {
        let line = self.content.cursor().position.line;
        let added = if self.bookmarks.contains(&line) {
            self.bookmarks.remove(&line);
            false
        } else {
            self.bookmarks.insert(line);
            true
        };
        (line + 1, added)
    }

    pub fn refresh_markdown(&mut self) {
        let src = self.content.text();
        self.markdown_items = markdown::parse(&src).collect();
    }

    pub fn refresh_csv(&mut self) {
        let src = self.content.text();
        match parse_csv(&src, 500) {
            Ok(table) => {
                self.csv_table = Some(table);
                self.csv_error = None;
            }
            Err(err) => {
                self.csv_table = None;
                self.csv_error = Some(err);
            }
        }
    }

    pub fn refresh_hex(&mut self) {
        const HEX_PAGE_BYTES: usize = 2048;
        if let Some(ref reader) = self.large_file {
            let max_off = (reader.len() as usize).saturating_sub(1);
            self.hex_offset = self.hex_offset.min(max_off);
            self.hex_rows = reader.hex_rows(self.hex_offset, HEX_PAGE_BYTES);
        } else if let Some(ref path) = self.path
            && !self.is_dirty
            && let Ok(bytes) = std::fs::read(path)
        {
            let max_off = bytes.len().saturating_sub(1);
            self.hex_offset = self.hex_offset.min(max_off);
            let end = self
                .hex_offset
                .saturating_add(HEX_PAGE_BYTES)
                .min(bytes.len());
            self.hex_rows = format_hex_dump(&bytes[self.hex_offset..end], self.hex_offset);
        } else {
            let text = self.content.text();
            let bytes = text.as_bytes();
            let max_off = bytes.len().saturating_sub(1);
            self.hex_offset = self.hex_offset.min(max_off);
            let end = self
                .hex_offset
                .saturating_add(HEX_PAGE_BYTES)
                .min(bytes.len());
            self.hex_rows = format_hex_dump(&bytes[self.hex_offset..end], self.hex_offset);
        }
    }

    pub fn refresh_large_file_chunk(&mut self) {
        if let Some(ref reader) = self.large_file {
            let (s, e, chunk) =
                reader.read_text_window(self.large_file_offset, DEFAULT_CHUNK_BYTES);
            self.large_file_window = (s, e);
            self.large_file_chunk = chunk;
        }
    }

    pub fn refresh_active_view_cache(&mut self) {
        match self.view_mode {
            ViewMode::MarkdownSplit => self.refresh_markdown(),
            ViewMode::CsvGrid => self.refresh_csv(),
            ViewMode::HexView => self.refresh_hex(),
            ViewMode::LargeFileFastView => self.refresh_large_file_chunk(),
            ViewMode::Editor | ViewMode::DiffView => {}
        }
    }
}
