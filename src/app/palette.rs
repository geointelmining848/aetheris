use crate::app::Message;
use crate::app::document::ViewMode;
use crate::app::theme::AetherisTheme;
use crate::core::encoding::{LineEnding, TextEncoding};
use crate::core::language::Language;
use crate::core::transforms::TransformOp;

/// One executable entry in the fuzzy Command Palette (`Cmd/Ctrl + Shift + P`).
#[derive(Debug, Clone)]
pub struct PaletteCommand {
    pub category: &'static str,
    pub title: String,
    pub shortcut: &'static str,
    pub message: Message,
}

/// Build the full list of commands available in the Command Palette.
pub fn all_commands() -> Vec<PaletteCommand> {
    let mut cmds = vec![
        PaletteCommand {
            category: "File",
            title: "New Scratchpad Tab".into(),
            shortcut: "Cmd/Ctrl+N",
            message: Message::NewTab,
        },
        PaletteCommand {
            category: "File",
            title: "Open File...".into(),
            shortcut: "Cmd/Ctrl+O",
            message: Message::OpenFileDialog,
        },
        PaletteCommand {
            category: "File",
            title: "Open Large File in Fast View (memmap)...".into(),
            shortcut: "",
            message: Message::OpenLargeFileDialog,
        },
        PaletteCommand {
            category: "File",
            title: "Open Folder as Workspace...".into(),
            shortcut: "Cmd/Ctrl+Shift+O",
            message: Message::OpenFolderDialog,
        },
        PaletteCommand {
            category: "File",
            title: "Save Active Tab".into(),
            shortcut: "Cmd/Ctrl+S",
            message: Message::SaveActiveTab,
        },
        PaletteCommand {
            category: "File",
            title: "Save Active Tab As...".into(),
            shortcut: "Cmd/Ctrl+Shift+S",
            message: Message::SaveActiveTabAs,
        },
        PaletteCommand {
            category: "File",
            title: "Close Active Tab".into(),
            shortcut: "Cmd/Ctrl+W",
            message: Message::CloseActiveTab,
        },
        PaletteCommand {
            category: "View",
            title: "Mode: Code / Text Editor".into(),
            shortcut: "",
            message: Message::SetViewMode(ViewMode::Editor),
        },
        PaletteCommand {
            category: "View",
            title: "Mode: Live Markdown Preview".into(),
            shortcut: "",
            message: Message::SetViewMode(ViewMode::MarkdownSplit),
        },
        PaletteCommand {
            category: "View",
            title: "Mode: Side-by-Side Diff".into(),
            shortcut: "",
            message: Message::SetViewMode(ViewMode::DiffView),
        },
        PaletteCommand {
            category: "View",
            title: "Mode: CSV / TSV Table Grid".into(),
            shortcut: "",
            message: Message::SetViewMode(ViewMode::CsvGrid),
        },
        PaletteCommand {
            category: "View",
            title: "Mode: Hex / Binary Inspector".into(),
            shortcut: "",
            message: Message::SetViewMode(ViewMode::HexView),
        },
        PaletteCommand {
            category: "View",
            title: "Toggle Split Editor Pane".into(),
            shortcut: "Cmd/Ctrl+\\",
            message: Message::ToggleSplitView,
        },
        PaletteCommand {
            category: "View",
            title: "Toggle Sidebar".into(),
            shortcut: "Cmd/Ctrl+B",
            message: Message::ToggleSidebar,
        },
        PaletteCommand {
            category: "View",
            title: "Toggle Word Wrap".into(),
            shortcut: "",
            message: Message::ToggleWordWrap,
        },
        PaletteCommand {
            category: "View",
            title: "Toggle Line Numbers".into(),
            shortcut: "",
            message: Message::ToggleLineNumbers,
        },
        PaletteCommand {
            category: "View",
            title: "Toggle Developer Tools Drawer".into(),
            shortcut: "",
            message: Message::ToggleToolsDrawer,
        },
        PaletteCommand {
            category: "View",
            title: "Zoom In Font".into(),
            shortcut: "Cmd/Ctrl+=",
            message: Message::ZoomIn,
        },
        PaletteCommand {
            category: "View",
            title: "Zoom Out Font".into(),
            shortcut: "Cmd/Ctrl+-",
            message: Message::ZoomOut,
        },
        PaletteCommand {
            category: "View",
            title: "Reset Font Size".into(),
            shortcut: "Cmd/Ctrl+0",
            message: Message::ZoomReset,
        },
        PaletteCommand {
            category: "Search",
            title: "Find in Document".into(),
            shortcut: "Cmd/Ctrl+F",
            message: Message::ToggleFindBar(false),
        },
        PaletteCommand {
            category: "Search",
            title: "Find & Replace in Document".into(),
            shortcut: "Cmd/Ctrl+H",
            message: Message::ToggleFindBar(true),
        },
        PaletteCommand {
            category: "Search",
            title: "Find in Files (Workspace)".into(),
            shortcut: "Cmd/Ctrl+Shift+F",
            message: Message::OpenWorkspaceSearch,
        },
        PaletteCommand {
            category: "Search",
            title: "Go to Line...".into(),
            shortcut: "Cmd/Ctrl+G",
            message: Message::ToggleGotoLine,
        },
        PaletteCommand {
            category: "Bookmarks",
            title: "Toggle Bookmark on Current Line".into(),
            shortcut: "Cmd/Ctrl+K",
            message: Message::ToggleBookmark,
        },
        PaletteCommand {
            category: "Bookmarks",
            title: "Extract Bookmarked Lines to New Tab".into(),
            shortcut: "",
            message: Message::ExtractBookmarkedToNewTab,
        },
        PaletteCommand {
            category: "Bookmarks",
            title: "Delete All Bookmarked Lines".into(),
            shortcut: "",
            message: Message::DeleteBookmarkedLines,
        },
        PaletteCommand {
            category: "Bookmarks",
            title: "Clear Bookmarks in Active Tab".into(),
            shortcut: "",
            message: Message::ClearBookmarks,
        },
        PaletteCommand {
            category: "Macro",
            title: "Start / Stop Macro Recording".into(),
            shortcut: "",
            message: Message::ToggleMacroRecording,
        },
        PaletteCommand {
            category: "Macro",
            title: "Replay Recorded Macro (1x)".into(),
            shortcut: "",
            message: Message::ReplayMacro(1),
        },
        PaletteCommand {
            category: "Macro",
            title: "Replay Recorded Macro (5x)".into(),
            shortcut: "",
            message: Message::ReplayMacro(5),
        },
        PaletteCommand {
            category: "Macro",
            title: "Save Recorded Macro".into(),
            shortcut: "",
            message: Message::SaveCurrentMacro,
        },
    ];

    for &op in TransformOp::ALL {
        cmds.push(PaletteCommand {
            category: "Transform",
            title: op.label().to_owned(),
            shortcut: "",
            message: Message::ApplyTransform(op),
        });
    }

    for &enc in TextEncoding::ALL {
        cmds.push(PaletteCommand {
            category: "Encoding",
            title: format!("Convert Encoding to {}", enc.label()),
            shortcut: "",
            message: Message::SetEncoding(enc),
        });
        cmds.push(PaletteCommand {
            category: "Encoding",
            title: format!("Reopen File as {}", enc.label()),
            shortcut: "",
            message: Message::ReopenWithEncoding(enc),
        });
    }

    for &eol in LineEnding::ALL {
        cmds.push(PaletteCommand {
            category: "Line Ending",
            title: format!("Convert Line Endings to {}", eol.label()),
            shortcut: "",
            message: Message::SetLineEnding(eol),
        });
    }

    cmds.push(PaletteCommand {
        category: "Language",
        title: "Auto-Detect Language from Content".into(),
        shortcut: "",
        message: Message::AutoDetectLanguage,
    });
    for &lang in Language::ALL {
        cmds.push(PaletteCommand {
            category: "Language",
            title: format!("Set Syntax: {}", lang.label()),
            shortcut: "",
            message: Message::SetLanguage(lang),
        });
    }

    for &theme in AetherisTheme::ALL {
        cmds.push(PaletteCommand {
            category: "Theme",
            title: format!("Switch Theme: {}", theme.name()),
            shortcut: "",
            message: Message::SetTheme(theme),
        });
    }

    cmds
}

/// Filter palette commands with case-insensitive multi-token substring matching.
pub fn filter_commands(query: &str) -> Vec<PaletteCommand> {
    let tokens: Vec<String> = query
        .split_whitespace()
        .map(|t| t.to_ascii_lowercase())
        .collect();

    all_commands()
        .into_iter()
        .filter(|cmd| {
            if tokens.is_empty() {
                return true;
            }
            let hay = format!("{} {}", cmd.category, cmd.title).to_ascii_lowercase();
            tokens.iter().all(|tok| hay.contains(tok))
        })
        .collect()
}
