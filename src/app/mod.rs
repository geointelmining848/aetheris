pub mod document;
pub mod palette;
pub mod theme;
pub mod view;

use std::path::{Path, PathBuf};

use iced::keyboard::{self, Key, Modifiers};
use iced::widget::text_editor;
use iced::{Event, Size, Subscription, Task, Theme, event, window};

use crate::app::document::{DocumentTab, ViewMode};
use crate::app::theme::AetherisTheme;
use crate::core::diff::{compute_diff, unified_diff};
use crate::core::encoding::{
    LineEnding, TextEncoding, read_file_decoded, read_file_with_encoding, write_file_encoded,
};
use crate::core::language::Language;
use crate::core::large_file::{DEFAULT_CHUNK_BYTES, LARGE_FILE_THRESHOLD_BYTES, LargeFileReader};
use crate::core::macro_engine::{MacroRecorder, MacroStep};
use crate::core::search::{
    FileMatch, SearchOptions, TextMatch, find_all, find_in_files, replace_all, replace_one,
};
use crate::core::session::{SessionState, SessionTab, SessionVault};
use crate::core::transforms::{
    TransformOp, apply_transform, delete_bookmarked_lines, extract_bookmarked_lines,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarPanel {
    #[default]
    Explorer,
    Search,
    Bookmarks,
}

#[derive(Debug, Clone)]
pub struct WorkspaceEntry {
    pub path: PathBuf,
    pub display_name: String,
    pub is_dir: bool,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Tab & editor interactions
    EditorAction(text_editor::Action),
    SplitEditorAction(text_editor::Action),
    SelectTab(usize),
    SelectSplitTab(usize),
    NewTab,
    CloseTab(usize),
    CloseActiveTab,

    // File & workspace operations
    OpenFileDialog,
    OpenLargeFileDialog,
    OpenFolderDialog,
    OpenPath(PathBuf),
    OpenLargePath(PathBuf),
    FileDropped(PathBuf),
    SaveActiveTab,
    SaveActiveTabAs,
    WorkspaceFilterChanged(String),
    RefreshWorkspace,

    // View & chrome controls
    SetViewMode(ViewMode),
    ToggleSidebar,
    SelectSidebarPanel(SidebarPanel),
    ToggleSplitView,
    ToggleWordWrap,
    ToggleLineNumbers,
    ToggleToolsDrawer,
    CycleTheme,
    SetTheme(AetherisTheme),
    ZoomIn,
    ZoomOut,
    ZoomReset,

    // Language, encoding, EOL
    CycleLanguage,
    SetLanguage(Language),
    AutoDetectLanguage,
    CycleEncoding,
    SetEncoding(TextEncoding),
    ReopenWithEncoding(TextEncoding),
    CycleLineEnding,
    SetLineEnding(LineEnding),

    // Find / Replace in active document
    ToggleFindBar(bool),
    CloseOverlays,
    FindQueryChanged(String),
    ReplaceQueryChanged(String),
    ToggleSearchCase,
    ToggleSearchWord,
    ToggleSearchRegex,
    FindNext,
    FindPrev,
    ReplaceCurrent,
    ReplaceAllInTab,

    // Workspace Find in Files
    OpenWorkspaceSearch,
    WorkspaceSearchQueryChanged(String),
    RunWorkspaceSearch,
    OpenSearchHit(PathBuf, usize),

    // Go to line
    ToggleGotoLine,
    GotoLineInputChanged(String),
    ExecuteGotoLine,

    // Command Palette
    ToggleCommandPalette,
    PaletteQueryChanged(String),
    ExecutePaletteTop,

    // Transforms & Developer Toolkit
    ApplyTransform(TransformOp),

    // Bookmarks
    ToggleBookmark,
    JumpToBookmark(usize),
    ExtractBookmarkedToNewTab,
    DeleteBookmarkedLines,
    ClearBookmarks,

    // Macros
    ToggleMacroRecording,
    ReplayMacro(usize),
    SaveCurrentMacro,
    MacroNameChanged(String),
    ReplaySavedMacro(usize),

    // Diff View
    SetDiffTarget(u64),
    RefreshDiff,
    ExportDiffPatch,

    // Markdown link click
    MarkdownLinkClicked(String),

    // CSV Grid controls
    CsvFilterChanged(String),
    CsvSortColumn(usize),

    // Hex Viewer controls
    HexPrevPage,
    HexNextPage,
    HexOffsetInputChanged(String),
    HexJumpOffset,

    // Large-File Fast View controls
    LargeFilePrevPage,
    LargeFileNextPage,
    LargeFileJumpPercent(u8),
    LargeFileSearchInputChanged(String),
    LargeFileFindNext,

    // Runtime window / close handling
    WindowCloseRequested(window::Id),
    Noop,
}

pub struct AetherisApp {
    pub tabs: Vec<DocumentTab>,
    pub active_tab: usize,
    pub split_enabled: bool,
    pub split_tab: Option<usize>,
    pub next_tab_id: u64,

    // Workspace explorer
    pub workspace_root: Option<PathBuf>,
    pub workspace_entries: Vec<WorkspaceEntry>,
    pub workspace_filter: String,

    // UI layout & theme
    pub theme: AetherisTheme,
    pub font_size: u16,
    pub word_wrap: bool,
    pub show_line_numbers: bool,
    pub show_sidebar: bool,
    pub sidebar_panel: SidebarPanel,
    pub show_tools_drawer: bool,

    // Find & replace bar
    pub show_find_bar: bool,
    pub show_replace_row: bool,
    pub find_query: String,
    pub replace_query: String,
    pub search_opts: SearchOptions,
    pub find_matches: Vec<TextMatch>,
    pub active_match_idx: usize,

    // Workspace search
    pub workspace_search_query: String,
    pub workspace_search_hits: Vec<FileMatch>,

    // Go to line
    pub show_goto_line: bool,
    pub goto_line_input: String,

    // Command palette
    pub show_palette: bool,
    pub palette_query: String,

    // Macros
    pub macro_recorder: MacroRecorder,
    pub macro_name_input: String,

    // Hex / Large file controls
    pub hex_jump_input: String,
    pub large_file_search_input: String,

    // Session vault & status line
    pub vault: SessionVault,
    pub status_message: String,
}

const WELCOME_MARKDOWN: &str = r#"# Welcome to Aetheris

**Aetheris** is a standalone, pure-Rust, zero-WebView text editor for **Windows, macOS, and Linux**.

## Quick Tour — Try These Built-In Tools Right Here

1. **Live Markdown Preview**: Click **Markdown** in the top bar to preview this document side-by-side (native rendering — zero browser).
2. **Command Palette**: Press `Cmd/Ctrl + Shift + P` to search 50+ built-in commands, transforms, themes, and encodings.
3. **Developer Swiss-Army Knife**: Click **Tools** in the top bar for instant:
   - Line sorting, deduplication, whitespace trimming, and bookmarking (`Cmd/Ctrl + K`)
   - JSON & XML Pretty-Print / Minify, Base64 / URL / HTML / JWT decode, MD5 / SHA-256 / SHA-512 hashes
   - Side-by-side **Diff** between any two open tabs, **CSV/TSV** grid view, **Hex** inspector, and **memmap2 Fast View** for multi-GB files
4. **Zero-Prompt Session Vault**: Unsaved scratchpads like this tab survive restarts automatically without nagging.
5. **Encoding & Line Endings**: Click `UTF-8` or `LF` in the bottom status bar to switch encodings or line endings on the fly.

```rust
fn main() {
    println!("Feather-light. Truly standalone. Uncompromisingly versatile.");
}
```
"#;

pub fn run(cli_paths: Vec<PathBuf>) -> iced::Result {
    iced::application(
        move || AetherisApp::boot(cli_paths.clone()),
        AetherisApp::update,
        view::view,
    )
    .title(AetherisApp::title)
    .theme(AetherisApp::iced_theme)
    .subscription(AetherisApp::subscription)
    .window_size(Size::new(1360.0, 860.0))
    .centered()
    .exit_on_close_request(false)
    .run()
}

impl AetherisApp {
    fn boot(cli_paths: Vec<PathBuf>) -> (Self, Task<Message>) {
        let vault = SessionVault::default_location();
        let mut macro_recorder = MacroRecorder::new();
        let _ = macro_recorder.load_from(&MacroRecorder::default_storage_path());

        let mut app = Self {
            tabs: Vec::new(),
            active_tab: 0,
            split_enabled: false,
            split_tab: None,
            next_tab_id: 1,
            workspace_root: None,
            workspace_entries: Vec::new(),
            workspace_filter: String::new(),
            theme: AetherisTheme::Midnight,
            font_size: 14,
            word_wrap: true,
            show_line_numbers: true,
            show_sidebar: true,
            sidebar_panel: SidebarPanel::Explorer,
            show_tools_drawer: false,
            show_find_bar: false,
            show_replace_row: false,
            find_query: String::new(),
            replace_query: String::new(),
            search_opts: SearchOptions::default(),
            find_matches: Vec::new(),
            active_match_idx: 0,
            workspace_search_query: String::new(),
            workspace_search_hits: Vec::new(),
            show_goto_line: false,
            goto_line_input: String::new(),
            show_palette: false,
            palette_query: String::new(),
            macro_recorder,
            macro_name_input: "My Macro".into(),
            hex_jump_input: String::new(),
            large_file_search_input: String::new(),
            vault,
            status_message: "Ready — Pure Rust standalone engine".into(),
        };

        // 1. Restore previous session automatically (Notepad++ zero-prompt persistence)
        app.restore_session();

        // 2. Open any CLI paths passed on launch
        for path in cli_paths {
            if path.is_dir() {
                app.set_workspace_root(path);
            } else if path.is_file() {
                app.open_file_path(path, false);
            }
        }

        // 3. First-time welcome buffer if nothing was restored or passed
        if app.tabs.is_empty() {
            let id = app.alloc_tab_id();
            app.tabs.push(DocumentTab::new_scratch(
                id,
                "Welcome.md".into(),
                WELCOME_MARKDOWN,
                Language::Markdown,
            ));
            app.active_tab = 0;
        }

        (app, Task::none())
    }

    pub fn title(&self) -> String {
        if let Some(tab) = self.tabs.get(self.active_tab) {
            let dirty = if tab.is_dirty { "● " } else { "" };
            format!("{dirty}{} — Aetheris", tab.title)
        } else {
            "Aetheris".into()
        }
    }

    pub fn iced_theme(&self) -> Theme {
        self.theme.to_iced_theme()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen_with(handle_runtime_event)
    }

    fn alloc_tab_id(&mut self) -> u64 {
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        id
    }

    pub fn active_doc(&self) -> Option<&DocumentTab> {
        self.tabs.get(self.active_tab)
    }

    pub fn active_doc_mut(&mut self) -> Option<&mut DocumentTab> {
        self.tabs.get_mut(self.active_tab)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EditorAction(action) => {
                let is_edit = action.is_edit();
                let mut auto_switched = None;
                if let Some(doc) = self.active_doc_mut() {
                    doc.content.perform(action);
                    if is_edit {
                        doc.is_dirty = true;
                        auto_switched = doc.auto_detect_language();
                        doc.refresh_active_view_cache();
                    }
                }
                if let Some(lang) = auto_switched {
                    self.status_message = format!("Auto-detected syntax: {}", lang.label());
                }
                if is_edit && self.show_find_bar && !self.find_query.is_empty() {
                    self.recompute_find_matches();
                }
            }
            Message::SplitEditorAction(action) => {
                let is_edit = action.is_edit();
                if let Some(split_idx) = self.split_tab
                    && let Some(doc) = self.tabs.get_mut(split_idx)
                {
                    doc.content.perform(action);
                    if is_edit {
                        doc.is_dirty = true;
                        doc.auto_detect_language();
                        doc.refresh_active_view_cache();
                    }
                }
            }
            Message::SelectTab(idx) => {
                if idx < self.tabs.len() {
                    self.active_tab = idx;
                    if let Some(doc) = self.active_doc_mut() {
                        doc.refresh_active_view_cache();
                    }
                    if self.show_find_bar {
                        self.recompute_find_matches();
                    }
                }
            }
            Message::SelectSplitTab(idx) => {
                if idx < self.tabs.len() {
                    self.split_tab = Some(idx);
                }
            }
            Message::NewTab => {
                let id = self.alloc_tab_id();
                let title = format!("Scratch {id}");
                self.tabs
                    .push(DocumentTab::new_scratch(id, title, "", Language::PlainText));
                self.active_tab = self.tabs.len() - 1;
                self.status_message = "Created new scratchpad tab (auto-persisted)".into();
                self.persist_session();
            }
            Message::CloseTab(idx) => {
                self.close_tab_at(idx);
            }
            Message::CloseActiveTab => {
                self.close_tab_at(self.active_tab);
            }
            Message::OpenFileDialog => {
                if let Some(paths) = rfd::FileDialog::new().pick_files() {
                    for p in paths {
                        self.open_file_path(p, false);
                    }
                    self.persist_session();
                }
            }
            Message::OpenLargeFileDialog => {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.open_file_path(path, true);
                    self.persist_session();
                }
            }
            Message::OpenFolderDialog => {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    self.set_workspace_root(folder);
                    self.show_sidebar = true;
                    self.sidebar_panel = SidebarPanel::Explorer;
                    self.persist_session();
                }
            }
            Message::OpenPath(path) => {
                if path.is_dir() {
                    self.set_workspace_root(path);
                } else {
                    self.open_file_path(path, false);
                }
                self.persist_session();
            }
            Message::OpenLargePath(path) => {
                self.open_file_path(path, true);
                self.persist_session();
            }
            Message::FileDropped(path) => {
                if path.is_dir() {
                    self.set_workspace_root(path);
                } else {
                    self.open_file_path(path, false);
                }
                self.persist_session();
            }
            Message::SaveActiveTab => {
                self.save_active_tab(false);
            }
            Message::SaveActiveTabAs => {
                self.save_active_tab(true);
            }
            Message::WorkspaceFilterChanged(q) => {
                self.workspace_filter = q;
            }
            Message::RefreshWorkspace => {
                if let Some(root) = self.workspace_root.clone() {
                    self.set_workspace_root(root);
                }
            }
            Message::SetViewMode(mode) => {
                if mode == ViewMode::DiffView {
                    self.prepare_diff_for_active_tab();
                } else if let Some(doc) = self.active_doc_mut() {
                    if mode == ViewMode::LargeFileFastView && doc.large_file.is_none() {
                        if let Some(path) = doc.path.clone() {
                            if let Ok(reader) = LargeFileReader::open(&path) {
                                doc.large_file = Some(reader);
                            }
                        } else {
                            self.status_message = "Fast View requires a file saved on disk".into();
                            return Task::none();
                        }
                    }
                    doc.view_mode = mode;
                    doc.refresh_active_view_cache();
                    self.status_message = format!("Switched view mode to {}", mode.label());
                }
            }
            Message::ToggleSidebar => {
                self.show_sidebar = !self.show_sidebar;
            }
            Message::SelectSidebarPanel(panel) => {
                if self.show_sidebar && self.sidebar_panel == panel {
                    self.show_sidebar = false;
                } else {
                    self.show_sidebar = true;
                    self.sidebar_panel = panel;
                }
            }
            Message::ToggleSplitView => {
                self.split_enabled = !self.split_enabled;
                if self.split_enabled && self.split_tab.is_none() {
                    let other = if self.tabs.len() > 1 {
                        (self.active_tab + 1) % self.tabs.len()
                    } else {
                        self.active_tab
                    };
                    self.split_tab = Some(other);
                }
            }
            Message::ToggleWordWrap => {
                self.word_wrap = !self.word_wrap;
                self.persist_session();
            }
            Message::ToggleLineNumbers => {
                self.show_line_numbers = !self.show_line_numbers;
                self.persist_session();
            }
            Message::ToggleToolsDrawer => {
                self.show_tools_drawer = !self.show_tools_drawer;
            }
            Message::CycleTheme => {
                self.theme = self.theme.next();
                self.status_message = format!("Theme: {}", self.theme.name());
                self.persist_session();
            }
            Message::SetTheme(theme) => {
                self.theme = theme;
                self.status_message = format!("Theme: {}", self.theme.name());
                self.show_palette = false;
                self.persist_session();
            }
            Message::ZoomIn => {
                self.font_size = (self.font_size + 1).min(32);
                self.persist_session();
            }
            Message::ZoomOut => {
                self.font_size = self.font_size.saturating_sub(1).max(9);
                self.persist_session();
            }
            Message::ZoomReset => {
                self.font_size = 14;
                self.persist_session();
            }
            Message::CycleLanguage => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.language = doc.language.next();
                    doc.language_locked = true;
                    self.status_message = format!("Language: {}", doc.language.label());
                }
            }
            Message::SetLanguage(lang) => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.language = lang;
                    doc.language_locked = true;
                    self.status_message = format!("Language: {}", lang.label());
                }
                self.show_palette = false;
            }
            Message::AutoDetectLanguage => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.language_locked = false;
                    let detected = Language::detect_from_content(&doc.content.text())
                        .unwrap_or(Language::PlainText);
                    doc.language = detected;
                    self.status_message = format!("Auto-detect enabled: {}", doc.language.label());
                }
                self.show_palette = false;
            }
            Message::CycleEncoding => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.encoding = doc.encoding.next();
                    doc.is_dirty = true;
                    self.status_message =
                        format!("Output encoding set to {}", doc.encoding.label());
                }
            }
            Message::SetEncoding(enc) => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.encoding = enc;
                    doc.is_dirty = true;
                    self.status_message = format!("Converted target encoding to {}", enc.label());
                }
                self.show_palette = false;
            }
            Message::ReopenWithEncoding(enc) => {
                if let Some(path) = self.active_doc().and_then(|d| d.path.clone()) {
                    match read_file_with_encoding(&path, enc) {
                        Ok(decoded) => {
                            if let Some(doc) = self.active_doc_mut() {
                                doc.content = text_editor::Content::with_text(&decoded.text);
                                doc.encoding = decoded.encoding;
                                doc.line_ending = decoded.line_ending;
                                doc.is_dirty = false;
                                doc.refresh_active_view_cache();
                            }
                            self.status_message = format!("Reopened file as {}", enc.label());
                        }
                        Err(e) => {
                            self.status_message = format!("Reopen failed: {e}");
                        }
                    }
                } else {
                    self.status_message = "Cannot reopen unsaved scratchpad from disk".into();
                }
                self.show_palette = false;
            }
            Message::CycleLineEnding => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.line_ending = doc.line_ending.next();
                    doc.is_dirty = true;
                    self.status_message =
                        format!("Line endings set to {}", doc.line_ending.label());
                }
            }
            Message::SetLineEnding(eol) => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.line_ending = eol;
                    doc.is_dirty = true;
                    self.status_message = format!("Line endings set to {}", eol.label());
                }
                self.show_palette = false;
            }
            Message::ToggleFindBar(with_replace) => {
                if self.show_find_bar && self.show_replace_row == with_replace {
                    self.show_find_bar = false;
                } else {
                    self.show_find_bar = true;
                    self.show_replace_row = with_replace;
                    self.show_palette = false;
                    self.recompute_find_matches();
                }
            }
            Message::CloseOverlays => {
                self.show_palette = false;
                self.show_find_bar = false;
                self.show_goto_line = false;
            }
            Message::FindQueryChanged(q) => {
                self.find_query = q;
                self.active_match_idx = 0;
                self.recompute_find_matches();
            }
            Message::ReplaceQueryChanged(q) => {
                self.replace_query = q;
            }
            Message::ToggleSearchCase => {
                self.search_opts.case_sensitive = !self.search_opts.case_sensitive;
                self.recompute_find_matches();
            }
            Message::ToggleSearchWord => {
                self.search_opts.whole_word = !self.search_opts.whole_word;
                self.recompute_find_matches();
            }
            Message::ToggleSearchRegex => {
                self.search_opts.use_regex = !self.search_opts.use_regex;
                self.recompute_find_matches();
            }
            Message::FindNext => {
                if !self.find_matches.is_empty() {
                    self.active_match_idx = (self.active_match_idx + 1) % self.find_matches.len();
                    let line = self.find_matches[self.active_match_idx].line + 1;
                    if let Some(doc) = self.active_doc_mut() {
                        doc.jump_to_line(line);
                    }
                    self.status_message = format!(
                        "Match {} / {} (line {line})",
                        self.active_match_idx + 1,
                        self.find_matches.len()
                    );
                }
            }
            Message::FindPrev => {
                if !self.find_matches.is_empty() {
                    self.active_match_idx = if self.active_match_idx == 0 {
                        self.find_matches.len() - 1
                    } else {
                        self.active_match_idx - 1
                    };
                    let line = self.find_matches[self.active_match_idx].line + 1;
                    if let Some(doc) = self.active_doc_mut() {
                        doc.jump_to_line(line);
                    }
                    self.status_message = format!(
                        "Match {} / {} (line {line})",
                        self.active_match_idx + 1,
                        self.find_matches.len()
                    );
                }
            }
            Message::ReplaceCurrent => {
                if let Some(current_text) = self.active_doc().map(|d| d.content.text()) {
                    match replace_one(
                        &current_text,
                        &self.find_query,
                        &self.replace_query,
                        self.active_match_idx,
                        &self.search_opts,
                    ) {
                        Ok((new_text, true)) => {
                            if let Some(doc) = self.active_doc_mut() {
                                doc.replace_full_text(&new_text);
                            }
                            self.recompute_find_matches();
                            self.status_message = "Replaced 1 match".into();
                        }
                        Ok((_, false)) => {
                            self.status_message = "No matching occurrence to replace".into();
                        }
                        Err(e) => self.status_message = e,
                    }
                }
            }
            Message::ReplaceAllInTab => {
                if let Some(current_text) = self.active_doc().map(|d| d.content.text()) {
                    match replace_all(
                        &current_text,
                        &self.find_query,
                        &self.replace_query,
                        &self.search_opts,
                    ) {
                        Ok((new_text, count)) => {
                            if count > 0 {
                                if let Some(doc) = self.active_doc_mut() {
                                    doc.replace_full_text(&new_text);
                                }
                                self.macro_recorder.record(MacroStep::ReplaceAll {
                                    query: self.find_query.clone(),
                                    replacement: self.replace_query.clone(),
                                    case_sensitive: self.search_opts.case_sensitive,
                                    whole_word: self.search_opts.whole_word,
                                    use_regex: self.search_opts.use_regex,
                                });
                            }
                            self.recompute_find_matches();
                            self.status_message = format!("Replaced {count} occurrence(s)");
                        }
                        Err(e) => self.status_message = e,
                    }
                }
            }
            Message::OpenWorkspaceSearch => {
                self.show_sidebar = true;
                self.sidebar_panel = SidebarPanel::Search;
                self.show_palette = false;
            }
            Message::WorkspaceSearchQueryChanged(q) => {
                self.workspace_search_query = q;
            }
            Message::RunWorkspaceSearch => {
                if let Some(root) = self.workspace_root.clone() {
                    match find_in_files(&root, &self.workspace_search_query, &self.search_opts, 250)
                    {
                        Ok(hits) => {
                            let count = hits.len();
                            self.workspace_search_hits = hits;
                            self.status_message = format!("Workspace search: {count} hit(s) found");
                        }
                        Err(e) => self.status_message = e,
                    }
                } else {
                    self.status_message =
                        "Open a folder as workspace first (Cmd/Ctrl+Shift+O)".into();
                }
            }
            Message::OpenSearchHit(path, zero_based_line) => {
                self.open_file_path(path, false);
                if let Some(doc) = self.active_doc_mut() {
                    doc.jump_to_line(zero_based_line + 1);
                }
                self.status_message = format!("Jumped to line {}", zero_based_line + 1);
            }
            Message::ToggleGotoLine => {
                self.show_goto_line = !self.show_goto_line;
                self.show_palette = false;
            }
            Message::GotoLineInputChanged(val) => {
                self.goto_line_input = val;
            }
            Message::ExecuteGotoLine => {
                if let Ok(line) = self.goto_line_input.trim().parse::<usize>() {
                    if let Some(doc) = self.active_doc_mut() {
                        doc.jump_to_line(line.max(1));
                    }
                    self.status_message = format!("Jumped to line {}", line.max(1));
                    self.show_goto_line = false;
                } else {
                    self.status_message = "Enter a valid positive line number".into();
                }
            }
            Message::ToggleCommandPalette => {
                self.show_palette = !self.show_palette;
                if self.show_palette {
                    self.palette_query.clear();
                }
            }
            Message::PaletteQueryChanged(q) => {
                self.palette_query = q;
            }
            Message::ExecutePaletteTop => {
                let matches = palette::filter_commands(&self.palette_query);
                self.show_palette = false;
                if let Some(first) = matches.into_iter().next() {
                    return self.update(first.message);
                }
            }
            Message::ApplyTransform(op) => {
                self.show_palette = false;
                if let Some(doc) = self.active_doc_mut() {
                    let src = doc.content.text();
                    match apply_transform(&src, op) {
                        Ok(transformed) => {
                            doc.replace_full_text(&transformed);
                            self.macro_recorder.record(MacroStep::Transform(op));
                            self.status_message = format!("Applied: {}", op.label());
                        }
                        Err(e) => self.status_message = e,
                    }
                }
            }
            Message::ToggleBookmark => {
                if let Some(doc) = self.active_doc_mut() {
                    let (line, added) = doc.toggle_bookmark_at_cursor();
                    self.status_message = if added {
                        format!("Bookmarked line {line}")
                    } else {
                        format!("Removed bookmark at line {line}")
                    };
                }
            }
            Message::JumpToBookmark(zero_based) => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.jump_to_line(zero_based + 1);
                    self.status_message = format!("Jumped to bookmark on line {}", zero_based + 1);
                }
            }
            Message::ExtractBookmarkedToNewTab => {
                let extracted = self.active_doc().map(|doc| {
                    let marks: Vec<usize> = doc.bookmarks.iter().copied().collect();
                    extract_bookmarked_lines(&doc.content.text(), &marks)
                });
                if let Some(text) = extracted {
                    if text.is_empty() {
                        self.status_message = "No bookmarked lines in active tab".into();
                    } else {
                        let id = self.alloc_tab_id();
                        self.tabs.push(DocumentTab::new_scratch(
                            id,
                            format!("Bookmarked {id}.txt"),
                            &text,
                            Language::PlainText,
                        ));
                        self.active_tab = self.tabs.len() - 1;
                        self.status_message = "Extracted bookmarked lines to new tab".into();
                    }
                }
                self.show_palette = false;
            }
            Message::DeleteBookmarkedLines => {
                if let Some(doc) = self.active_doc_mut() {
                    if doc.bookmarks.is_empty() {
                        self.status_message = "No bookmarked lines to delete".into();
                    } else {
                        let marks: Vec<usize> = doc.bookmarks.iter().copied().collect();
                        let updated = delete_bookmarked_lines(&doc.content.text(), &marks);
                        let count = marks.len();
                        doc.bookmarks.clear();
                        doc.replace_full_text(&updated);
                        self.status_message = format!("Deleted {count} bookmarked line(s)");
                    }
                }
                self.show_palette = false;
            }
            Message::ClearBookmarks => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.bookmarks.clear();
                    self.status_message = "Cleared bookmarks in active tab".into();
                }
                self.show_palette = false;
            }
            Message::ToggleMacroRecording => {
                if self.macro_recorder.is_recording() {
                    self.macro_recorder.stop_recording();
                    self.status_message = format!(
                        "Stopped macro recording ({} step(s) captured)",
                        self.macro_recorder.current_steps().len()
                    );
                } else {
                    self.macro_recorder.start_recording();
                    self.status_message =
                        "● Recording macro — apply transforms or Replace All, then click Stop"
                            .into();
                }
                self.show_palette = false;
            }
            Message::ReplayMacro(times) => {
                if let Some(src) = self.active_doc().map(|d| d.content.text()) {
                    match self.macro_recorder.replay_current(&src, times) {
                        Ok(out) => {
                            if let Some(doc) = self.active_doc_mut() {
                                doc.replace_full_text(&out);
                            }
                            self.status_message = format!("Replayed macro ({times}x)");
                        }
                        Err(e) => self.status_message = e,
                    }
                }
                self.show_palette = false;
            }
            Message::MacroNameChanged(name) => {
                self.macro_name_input = name;
            }
            Message::SaveCurrentMacro => {
                let name = if self.macro_name_input.trim().is_empty() {
                    format!("Macro {}", self.macro_recorder.saved_macros().len() + 1)
                } else {
                    self.macro_name_input.trim().to_owned()
                };
                if self.macro_recorder.save_current(name.clone()) {
                    let _ = self
                        .macro_recorder
                        .persist_to(&MacroRecorder::default_storage_path());
                    self.status_message = format!("Saved macro '{name}'");
                } else {
                    self.status_message = "Record at least one step before saving a macro".into();
                }
                self.show_palette = false;
            }
            Message::ReplaySavedMacro(idx) => {
                let steps = self
                    .macro_recorder
                    .saved_macros()
                    .get(idx)
                    .map(|m| (m.name.clone(), m.steps.clone()));
                if let Some((name, steps)) = steps
                    && let Some(doc) = self.active_doc_mut()
                {
                    let src = doc.content.text();
                    match MacroRecorder::replay_steps(&src, &steps, 1) {
                        Ok(out) => {
                            doc.replace_full_text(&out);
                            self.status_message = format!("Executed saved macro '{name}'");
                        }
                        Err(e) => self.status_message = e,
                    }
                }
            }
            Message::SetDiffTarget(target_id) => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.diff_target_id = Some(target_id);
                }
                self.recompute_active_diff();
            }
            Message::RefreshDiff => {
                self.recompute_active_diff();
            }
            Message::ExportDiffPatch => {
                if let Some(active) = self.active_doc() {
                    let left_name = active.title.clone();
                    let left_text = active.content.text();
                    if let Some(target_id) = active.diff_target_id
                        && let Some(other) = self.tabs.iter().find(|t| t.id == target_id)
                    {
                        let patch = unified_diff(
                            &left_name,
                            &left_text,
                            &other.title,
                            &other.content.text(),
                        );
                        let id = self.alloc_tab_id();
                        self.tabs.push(DocumentTab::new_scratch(
                            id,
                            format!("diff-{id}.patch"),
                            &patch,
                            Language::Diff,
                        ));
                        self.active_tab = self.tabs.len() - 1;
                        self.status_message = "Exported unified .patch to new tab".into();
                    }
                }
            }
            Message::MarkdownLinkClicked(uri) => {
                self.status_message = format!("Markdown link: {uri}");
            }
            Message::CsvFilterChanged(f) => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.csv_filter = f;
                }
            }
            Message::CsvSortColumn(col) => {
                if let Some(doc) = self.active_doc_mut() {
                    if doc.csv_sort_col == Some(col) {
                        doc.csv_sort_asc = !doc.csv_sort_asc;
                    } else {
                        doc.csv_sort_col = Some(col);
                        doc.csv_sort_asc = true;
                    }
                }
            }
            Message::HexPrevPage => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.hex_offset = doc.hex_offset.saturating_sub(2048);
                    doc.refresh_hex();
                }
            }
            Message::HexNextPage => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.hex_offset = doc.hex_offset.saturating_add(2048);
                    doc.refresh_hex();
                }
            }
            Message::HexOffsetInputChanged(s) => {
                self.hex_jump_input = s;
            }
            Message::HexJumpOffset => {
                let raw = self.hex_jump_input.trim();
                let parsed = if let Some(hex_part) =
                    raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X"))
                {
                    usize::from_str_radix(hex_part, 16)
                } else {
                    raw.parse::<usize>()
                };
                match parsed {
                    Ok(off) => {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.hex_offset = off;
                            doc.refresh_hex();
                        }
                        self.status_message = format!("Hex inspector jumped to 0x{off:08X}");
                    }
                    Err(_) => {
                        self.status_message = "Enter decimal or 0x-prefixed hex byte offset".into();
                    }
                }
            }
            Message::LargeFilePrevPage => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.large_file_offset =
                        doc.large_file_offset.saturating_sub(DEFAULT_CHUNK_BYTES);
                    doc.refresh_large_file_chunk();
                }
            }
            Message::LargeFileNextPage => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.large_file_offset = doc.large_file_window.1;
                    doc.refresh_large_file_chunk();
                }
            }
            Message::LargeFileJumpPercent(pct) => {
                if let Some(doc) = self.active_doc_mut() {
                    if let Some(ref reader) = doc.large_file {
                        let total = reader.len() as usize;
                        let target = if pct >= 100 {
                            total.saturating_sub(DEFAULT_CHUNK_BYTES)
                        } else {
                            (total / 100) * (pct as usize)
                        };
                        doc.large_file_offset = target;
                    }
                    doc.refresh_large_file_chunk();
                }
            }
            Message::LargeFileSearchInputChanged(s) => {
                self.large_file_search_input = s;
            }
            Message::LargeFileFindNext => {
                let needle = self.large_file_search_input.clone();
                if needle.is_empty() {
                    return Task::none();
                }
                if let Some(doc) = self.active_doc_mut()
                    && let Some(ref reader) = doc.large_file
                {
                    let start_from = doc.large_file_window.0.saturating_add(1);
                    if let Some(hit) = reader
                        .find_next_bytes(needle.as_bytes(), start_from)
                        .or_else(|| reader.find_next_bytes(needle.as_bytes(), 0))
                    {
                        doc.large_file_offset = hit.saturating_sub(128);
                        doc.refresh_large_file_chunk();
                        self.status_message = format!("Found '{needle}' at byte offset {hit}");
                    } else {
                        self.status_message = format!("'{needle}' not found in mapped file");
                    }
                }
            }
            Message::WindowCloseRequested(id) => {
                self.persist_session();
                return window::close(id);
            }
            Message::Noop => {}
        }

        Task::none()
    }

    fn open_file_path(&mut self, path: PathBuf, force_fast_view: bool) {
        // Focus tab if already open
        if !force_fast_view
            && let Some(existing_idx) = self
                .tabs
                .iter()
                .position(|t| t.path.as_ref() == Some(&path))
        {
            self.active_tab = existing_idx;
            self.status_message = format!("Focused {}", path.display());
            return;
        }

        let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        if force_fast_view || file_size >= LARGE_FILE_THRESHOLD_BYTES {
            match LargeFileReader::open(&path) {
                Ok(reader) => {
                    let mb = file_size as f64 / (1024.0 * 1024.0);
                    let id = self.alloc_tab_id();
                    self.tabs.push(DocumentTab::from_large_file(id, reader));
                    self.active_tab = self.tabs.len() - 1;
                    self.status_message = format!(
                        "Opened {} ({mb:.1} MiB) in memory-mapped Fast View",
                        path.display()
                    );
                }
                Err(e) => {
                    self.status_message = format!("Failed to map file: {e}");
                }
            }
            return;
        }

        match read_file_decoded(&path) {
            Ok(decoded) => {
                let id = self.alloc_tab_id();
                let tab = DocumentTab::from_decoded_file(
                    id,
                    path.clone(),
                    &decoded.text,
                    decoded.encoding,
                    decoded.line_ending,
                    decoded.looks_binary,
                );
                let note = if decoded.looks_binary {
                    " (binary detected — Hex View)"
                } else {
                    ""
                };
                self.tabs.push(tab);
                self.active_tab = self.tabs.len() - 1;
                self.status_message = format!(
                    "Opened {} [{} · {}]{note}",
                    path.display(),
                    decoded.encoding.label(),
                    decoded.line_ending.short_label()
                );
            }
            Err(e) => {
                self.status_message = format!("Could not open {}: {e}", path.display());
            }
        }
    }

    fn save_active_tab(&mut self, force_dialog: bool) {
        let Some(doc) = self.tabs.get(self.active_tab) else {
            return;
        };
        if doc.view_mode == ViewMode::LargeFileFastView {
            self.status_message = "Fast View is read-only for multi-GB safety".into();
            return;
        }

        let target_path = if !force_dialog && let Some(p) = doc.path.clone() {
            Some(p)
        } else {
            let mut dialog = rfd::FileDialog::new().set_file_name(&doc.title);
            if let Some(ref root) = self.workspace_root {
                dialog = dialog.set_directory(root);
            }
            dialog.save_file()
        };

        let Some(path) = target_path else {
            return;
        };

        let text = doc.content.text();
        let encoding = doc.encoding;
        let line_ending = doc.line_ending;

        match write_file_encoded(&path, &text, encoding, line_ending) {
            Ok(()) => {
                if let Some(doc_mut) = self.tabs.get_mut(self.active_tab) {
                    doc_mut.path = Some(path.clone());
                    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                        doc_mut.title = name.to_owned();
                    }
                    let by_ext = Language::detect_from_path(&path);
                    if by_ext != Language::PlainText {
                        doc_mut.language = by_ext;
                        doc_mut.language_locked = true;
                    }
                    doc_mut.is_dirty = false;
                }
                self.status_message = format!(
                    "Saved {} [{} · {}]",
                    path.display(),
                    encoding.label(),
                    line_ending.short_label()
                );
                self.persist_session();
            }
            Err(e) => {
                self.status_message = format!("Save failed: {e}");
            }
        }
    }

    fn close_tab_at(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }
        self.tabs.remove(idx);
        if self.tabs.is_empty() {
            let id = self.alloc_tab_id();
            self.tabs.push(DocumentTab::new_scratch(
                id,
                format!("Scratch {id}"),
                "",
                Language::PlainText,
            ));
            self.active_tab = 0;
            self.split_enabled = false;
            self.split_tab = None;
        } else {
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
            if let Some(split_idx) = self.split_tab
                && split_idx >= self.tabs.len()
            {
                self.split_tab = Some(self.tabs.len() - 1);
            }
        }
        self.persist_session();
    }

    fn set_workspace_root(&mut self, root: PathBuf) {
        self.workspace_entries = scan_workspace_tree(&root, 350);
        self.status_message = format!(
            "Workspace: {} ({} items)",
            root.display(),
            self.workspace_entries.len()
        );
        self.workspace_root = Some(root);
    }

    fn recompute_find_matches(&mut self) {
        if self.find_query.is_empty() {
            self.find_matches.clear();
            self.active_match_idx = 0;
            return;
        }
        if let Some(doc) = self.active_doc() {
            match find_all(&doc.content.text(), &self.find_query, &self.search_opts) {
                Ok(matches) => {
                    self.find_matches = matches;
                    if self.active_match_idx >= self.find_matches.len() {
                        self.active_match_idx = 0;
                    }
                }
                Err(e) => {
                    self.find_matches.clear();
                    self.status_message = e;
                }
            }
        }
    }

    fn prepare_diff_for_active_tab(&mut self) {
        let active_id = match self.active_doc() {
            Some(d) => d.id,
            None => return,
        };
        let default_target = self
            .tabs
            .iter()
            .find(|t| t.id != active_id)
            .map(|t| t.id)
            .unwrap_or(active_id);

        if let Some(doc) = self.active_doc_mut() {
            doc.view_mode = ViewMode::DiffView;
            if doc.diff_target_id.is_none() {
                doc.diff_target_id = Some(default_target);
            }
        }
        self.recompute_active_diff();
    }

    fn recompute_active_diff(&mut self) {
        let Some(active) = self.active_doc() else {
            return;
        };
        let left_text = active.content.text();
        let target_id = active.diff_target_id.unwrap_or(active.id);
        let right_text = self
            .tabs
            .iter()
            .find(|t| t.id == target_id)
            .map(|t| t.content.text())
            .unwrap_or_default();

        let report = compute_diff(&left_text, &right_text);
        let added = report.added_lines;
        let removed = report.removed_lines;
        if let Some(doc) = self.active_doc_mut() {
            doc.diff_report = Some(report);
        }
        self.status_message = format!("Diff updated: +{added} added, -{removed} removed");
    }

    fn persist_session(&self) {
        let mut session_tabs = Vec::with_capacity(self.tabs.len());
        let mut buffers = Vec::new();

        for tab in &self.tabs {
            if tab.view_mode == ViewMode::LargeFileFastView {
                session_tabs.push(SessionTab {
                    id: tab.id,
                    title: tab.title.clone(),
                    path: tab.path.clone(),
                    encoding: tab.encoding,
                    line_ending: tab.line_ending,
                    language: tab.language,
                    is_dirty: false,
                    bookmarks: tab.bookmarks.iter().copied().collect(),
                    backup_file: None,
                });
                continue;
            }

            let needs_backup = tab.path.is_none() || tab.is_dirty;
            let backup_file = if needs_backup {
                let fname = format!("tab-{}.txt", tab.id);
                buffers.push((tab.id, tab.content.text()));
                Some(fname)
            } else {
                None
            };

            session_tabs.push(SessionTab {
                id: tab.id,
                title: tab.title.clone(),
                path: tab.path.clone(),
                encoding: tab.encoding,
                line_ending: tab.line_ending,
                language: tab.language,
                is_dirty: tab.is_dirty,
                bookmarks: tab.bookmarks.iter().copied().collect(),
                backup_file,
            });
        }

        let state = SessionState {
            active_tab_index: self.active_tab,
            workspace_root: self.workspace_root.clone(),
            theme_name: self.theme.name().to_owned(),
            font_size: self.font_size,
            word_wrap: self.word_wrap,
            show_line_numbers: self.show_line_numbers,
            tabs: session_tabs,
        };

        let _ = self.vault.save(&state, &buffers);
    }

    fn restore_session(&mut self) {
        let Ok(Some(state)) = self.vault.load() else {
            return;
        };

        self.theme = AetherisTheme::from_name(&state.theme_name);
        if state.font_size >= 9 && state.font_size <= 32 {
            self.font_size = state.font_size;
        }
        self.word_wrap = state.word_wrap;
        self.show_line_numbers = state.show_line_numbers;

        if let Some(root) = state.workspace_root
            && root.is_dir()
        {
            self.set_workspace_root(root);
        }

        let mut max_id = 0u64;
        for saved in state.tabs {
            max_id = max_id.max(saved.id);
            let text_opt = if let Some(ref backup) = saved.backup_file {
                self.vault.load_buffer(backup).ok()
            } else if let Some(ref path) = saved.path {
                read_file_decoded(path).ok().map(|d| d.text)
            } else {
                Some(String::new())
            };

            if let Some(text) = text_opt {
                let mut tab =
                    DocumentTab::new_scratch(saved.id, saved.title, &text, saved.language);
                tab.path = saved.path;
                tab.encoding = saved.encoding;
                tab.line_ending = saved.line_ending;
                tab.is_dirty = saved.is_dirty;
                tab.bookmarks = saved.bookmarks.into_iter().collect();
                self.tabs.push(tab);
            }
        }

        self.next_tab_id = max_id + 1;
        if !self.tabs.is_empty() {
            self.active_tab = state.active_tab_index.min(self.tabs.len() - 1);
            self.status_message = format!("Restored {} tab(s) from Session Vault", self.tabs.len());
        }
    }
}

fn scan_workspace_tree(root: &Path, max_entries: usize) -> Vec<WorkspaceEntry> {
    let mut entries = Vec::new();
    let walker = ignore::WalkBuilder::new(root)
        .max_depth(Some(4))
        .hidden(true)
        .git_ignore(true)
        .sort_by_file_path(|a, b| a.cmp(b))
        .build();

    for item in walker.flatten() {
        if entries.len() >= max_entries {
            break;
        }
        let path = item.path();
        if path == root {
            continue;
        }
        let depth = item.depth().saturating_sub(1);
        let is_dir = path.is_dir();
        let display_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_owned();
        entries.push(WorkspaceEntry {
            path: path.to_path_buf(),
            display_name,
            is_dir,
            depth,
        });
    }
    entries
}

fn handle_runtime_event(
    ev: Event,
    _status: event::Status,
    window_id: window::Id,
) -> Option<Message> {
    match ev {
        Event::Window(window::Event::CloseRequested) => {
            Some(Message::WindowCloseRequested(window_id))
        }
        Event::Window(window::Event::FileDropped(path)) => Some(Message::FileDropped(path)),
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
            shortcut_to_message(key, modifiers)
        }
        _ => None,
    }
}

fn shortcut_to_message(key: Key, modifiers: Modifiers) -> Option<Message> {
    if matches!(key, Key::Named(keyboard::key::Named::Escape)) {
        return Some(Message::CloseOverlays);
    }

    if !modifiers.command() {
        return None;
    }

    let shift = modifiers.shift();
    match key.as_ref() {
        Key::Character("n") | Key::Character("N") => Some(Message::NewTab),
        Key::Character("o") | Key::Character("O") => {
            if shift {
                Some(Message::OpenFolderDialog)
            } else {
                Some(Message::OpenFileDialog)
            }
        }
        Key::Character("s") | Key::Character("S") => {
            if shift {
                Some(Message::SaveActiveTabAs)
            } else {
                Some(Message::SaveActiveTab)
            }
        }
        Key::Character("w") | Key::Character("W") => Some(Message::CloseActiveTab),
        Key::Character("f") | Key::Character("F") => {
            if shift {
                Some(Message::OpenWorkspaceSearch)
            } else {
                Some(Message::ToggleFindBar(false))
            }
        }
        Key::Character("h") | Key::Character("H") => Some(Message::ToggleFindBar(true)),
        Key::Character("p") | Key::Character("P") if shift => Some(Message::ToggleCommandPalette),
        Key::Character("b") | Key::Character("B") => Some(Message::ToggleSidebar),
        Key::Character("\\") => Some(Message::ToggleSplitView),
        Key::Character("g") | Key::Character("G") => Some(Message::ToggleGotoLine),
        Key::Character("k") | Key::Character("K") => Some(Message::ToggleBookmark),
        Key::Character("=") | Key::Character("+") => Some(Message::ZoomIn),
        Key::Character("-") => Some(Message::ZoomOut),
        Key::Character("0") => Some(Message::ZoomReset),
        _ => None,
    }
}
