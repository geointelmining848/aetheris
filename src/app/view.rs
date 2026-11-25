use iced::widget::{
    button, column, container, markdown, row, rule, scrollable, space, text, text_editor,
    text_input,
};
use iced::{Alignment, Background, Border, Color, Element, Font, Length, Padding};

use crate::app::document::{DocumentTab, ViewMode};
use crate::app::palette;
use crate::app::theme::ChromeColors;
use crate::app::{AetherisApp, Message, SidebarPanel};
use crate::core::diff::DiffTag;
use crate::core::transforms::TransformOp;

pub fn view(app: &AetherisApp) -> Element<'_, Message> {
    let c = app.theme.chrome();

    let mut root = column![
        header_bar(app, c),
        rule::horizontal(1),
        tab_bar(app, c),
        rule::horizontal(1),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    if app.show_palette {
        root = root
            .push(command_palette_panel(app, c))
            .push(rule::horizontal(1));
    }
    if app.show_find_bar {
        root = root
            .push(find_replace_bar(app, c))
            .push(rule::horizontal(1));
    }
    if app.show_goto_line {
        root = root.push(goto_line_bar(app, c)).push(rule::horizontal(1));
    }
    if app.show_tools_drawer {
        root = root.push(tools_drawer(app, c)).push(rule::horizontal(1));
    }

    let mut body = row![].width(Length::Fill).height(Length::Fill);

    if app.show_sidebar {
        body = body.push(sidebar_view(app, c)).push(rule::vertical(1));
    }

    body = body.push(main_workspace(app, c));

    root = root
        .push(body)
        .push(rule::horizontal(1))
        .push(status_bar(app, c));

    container(root)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(c.app_bg)),
            ..Default::default()
        })
        .into()
}

fn header_bar(app: &AetherisApp, c: ChromeColors) -> Element<'_, Message> {
    let active_mode = app
        .active_doc()
        .map(|d| d.view_mode)
        .unwrap_or(ViewMode::Editor);

    let mode_btn = |label: &'static str, mode: ViewMode| {
        let selected = active_mode == mode;
        chip_button(label, selected, c, Message::SetViewMode(mode))
    };

    let macro_label = if app.macro_recorder.is_recording() {
        "● Stop Rec"
    } else {
        "Rec Macro"
    };

    let bar = row![
        text("✦ Aetheris").size(15).color(c.accent).font(Font {
            weight: iced::font::Weight::Bold,
            ..Font::DEFAULT
        }),
        space::horizontal().width(8),
        subtle_button("New", c, Message::NewTab),
        subtle_button("Open", c, Message::OpenFileDialog),
        subtle_button("Folder", c, Message::OpenFolderDialog),
        subtle_button("Save", c, Message::SaveActiveTab),
        subtle_button("Fast Open", c, Message::OpenLargeFileDialog),
        space::horizontal().width(10),
        mode_btn("Code", ViewMode::Editor),
        mode_btn("Markdown", ViewMode::MarkdownSplit),
        mode_btn("Diff", ViewMode::DiffView),
        mode_btn("CSV", ViewMode::CsvGrid),
        mode_btn("Hex", ViewMode::HexView),
        chip_button("Split", app.split_enabled, c, Message::ToggleSplitView),
        space::horizontal().width(Length::Fill),
        chip_button("Find", app.show_find_bar, c, Message::ToggleFindBar(false)),
        chip_button(
            "Tools",
            app.show_tools_drawer,
            c,
            Message::ToggleToolsDrawer
        ),
        chip_button(
            macro_label,
            app.macro_recorder.is_recording(),
            c,
            Message::ToggleMacroRecording,
        ),
        subtle_button("Play", c, Message::ReplayMacro(1)),
        chip_button(
            "Palette",
            app.show_palette,
            c,
            Message::ToggleCommandPalette
        ),
        subtle_button(app.theme.name(), c, Message::CycleTheme),
    ]
    .spacing(5)
    .align_y(Alignment::Center)
    .padding(Padding::from([6, 10]));

    container(bar)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(c.header_bg)),
            ..Default::default()
        })
        .into()
}

fn tab_bar(app: &AetherisApp, c: ChromeColors) -> Element<'_, Message> {
    let mut tabs_row = row![].spacing(4).align_y(Alignment::Center);

    for (idx, tab) in app.tabs.iter().enumerate() {
        let is_active = idx == app.active_tab;
        let dirty_dot = if tab.is_dirty { "● " } else { "" };
        let badge = match tab.view_mode {
            ViewMode::Editor => "",
            ViewMode::MarkdownSplit => " [MD]",
            ViewMode::DiffView => " [DIFF]",
            ViewMode::CsvGrid => " [CSV]",
            ViewMode::HexView => " [HEX]",
            ViewMode::LargeFileFastView => " [FAST]",
        };
        let label = format!("{dirty_dot}{}{badge}", tab.title);

        let bg = if is_active { c.tab_active_bg } else { c.tab_bg };
        let border_color = if is_active { c.accent } else { c.border };
        let fg = if is_active { c.text } else { c.text_muted };

        let tab_pill = container(
            row![
                button(text(label).size(12).color(fg))
                    .padding(Padding::from([4, 8]))
                    .style(move |_, _| button::Style {
                        background: None,
                        text_color: fg,
                        ..Default::default()
                    })
                    .on_press(Message::SelectTab(idx)),
                button(text("×").size(12).color(c.text_muted))
                    .padding(Padding::from([4, 6]))
                    .style(move |_, _| button::Style {
                        background: None,
                        text_color: c.text_muted,
                        ..Default::default()
                    })
                    .on_press(Message::CloseTab(idx)),
            ]
            .align_y(Alignment::Center),
        )
        .style(move |_| container::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 5.0.into(),
            },
            ..Default::default()
        });

        tabs_row = tabs_row.push(tab_pill);
    }

    tabs_row = tabs_row.push(subtle_button("+", c, Message::NewTab));

    let strip = scrollable(tabs_row.padding(Padding::from([5, 8]))).direction(
        scrollable::Direction::Horizontal(scrollable::Scrollbar::new().width(4).scroller_width(4)),
    );

    container(strip)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(c.panel_bg)),
            ..Default::default()
        })
        .into()
}

fn command_palette_panel(app: &AetherisApp, c: ChromeColors) -> Element<'_, Message> {
    let matches = palette::filter_commands(&app.palette_query);
    let mut list = column![].spacing(2);

    for cmd in matches.into_iter().take(12) {
        let row_content = row![
            text(cmd.category)
                .size(11)
                .color(c.accent)
                .width(Length::Fixed(86.0)),
            text(cmd.title).size(12).color(c.text).width(Length::Fill),
            text(cmd.shortcut).size(11).color(c.text_muted),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let item = button(row_content)
            .width(Length::Fill)
            .padding(Padding::from([5, 10]))
            .style(move |_, status| {
                let bg = match status {
                    button::Status::Hovered | button::Status::Pressed => c.tab_active_bg,
                    _ => c.panel_bg,
                };
                button::Style {
                    background: Some(Background::Color(bg)),
                    text_color: c.text,
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .on_press(cmd.message);

        list = list.push(item);
    }

    let panel = column![
        row![
            text("⌘ Command Palette").size(12).color(c.accent),
            text_input(
                "Type any command, transform, encoding, theme, or view mode...",
                &app.palette_query
            )
            .on_input(Message::PaletteQueryChanged)
            .on_submit(Message::ExecutePaletteTop)
            .size(13)
            .width(Length::Fill),
            subtle_button("Close (Esc)", c, Message::CloseOverlays),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        scrollable(list).height(Length::Fixed(220.0)),
    ]
    .spacing(6)
    .padding(Padding::from([8, 12]));

    container(panel)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(c.panel_bg)),
            ..Default::default()
        })
        .into()
}

fn find_replace_bar(app: &AetherisApp, c: ChromeColors) -> Element<'_, Message> {
    let count_text = if app.find_query.is_empty() {
        "0 matches".to_owned()
    } else if app.find_matches.is_empty() {
        "No matches".to_owned()
    } else {
        format!("{} / {}", app.active_match_idx + 1, app.find_matches.len())
    };

    let mut content = column![
        row![
            text("Find:")
                .size(12)
                .color(c.text_muted)
                .width(Length::Fixed(54.0)),
            text_input("Search text or regex...", &app.find_query)
                .on_input(Message::FindQueryChanged)
                .on_submit(Message::FindNext)
                .size(12)
                .width(Length::Fixed(260.0)),
            chip_button(
                "Aa",
                app.search_opts.case_sensitive,
                c,
                Message::ToggleSearchCase
            ),
            chip_button(
                "\\b",
                app.search_opts.whole_word,
                c,
                Message::ToggleSearchWord
            ),
            chip_button(
                ".*",
                app.search_opts.use_regex,
                c,
                Message::ToggleSearchRegex
            ),
            text(count_text).size(12).color(c.text_muted),
            subtle_button("Prev", c, Message::FindPrev),
            subtle_button("Next", c, Message::FindNext),
            chip_button(
                "Replace...",
                app.show_replace_row,
                c,
                Message::ToggleFindBar(!app.show_replace_row),
            ),
            space::horizontal().width(Length::Fill),
            subtle_button("×", c, Message::CloseOverlays),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    ]
    .spacing(5)
    .padding(Padding::from([6, 12]));

    if app.show_replace_row {
        content = content.push(
            row![
                text("Replace:")
                    .size(12)
                    .color(c.text_muted)
                    .width(Length::Fixed(54.0)),
                text_input(
                    "Replacement text (supports $1 regex groups)...",
                    &app.replace_query
                )
                .on_input(Message::ReplaceQueryChanged)
                .on_submit(Message::ReplaceCurrent)
                .size(12)
                .width(Length::Fixed(260.0)),
                subtle_button("Replace One", c, Message::ReplaceCurrent),
                subtle_button("Replace All", c, Message::ReplaceAllInTab),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        );
    }

    container(content)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(c.panel_bg)),
            ..Default::default()
        })
        .into()
}

fn goto_line_bar(app: &AetherisApp, c: ChromeColors) -> Element<'_, Message> {
    let content = row![
        text("Go to line:").size(12).color(c.text_muted),
        text_input("Line number...", &app.goto_line_input)
            .on_input(Message::GotoLineInputChanged)
            .on_submit(Message::ExecuteGotoLine)
            .size(12)
            .width(Length::Fixed(140.0)),
        subtle_button("Jump", c, Message::ExecuteGotoLine),
        space::horizontal().width(Length::Fill),
        subtle_button("×", c, Message::CloseOverlays),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding(Padding::from([6, 12]));

    container(content)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(c.panel_bg)),
            ..Default::default()
        })
        .into()
}

fn tools_drawer(_app: &AetherisApp, c: ChromeColors) -> Element<'_, Message> {
    let op_btn =
        |label: &'static str, op: TransformOp| subtle_button(label, c, Message::ApplyTransform(op));

    let row1 = row![
        text("Lines:")
            .size(11)
            .color(c.accent)
            .width(Length::Fixed(52.0)),
        op_btn("Sort A→Z", TransformOp::SortLinesAsc),
        op_btn("Sort Z→A", TransformOp::SortLinesDesc),
        op_btn("Sort Num", TransformOp::SortLinesNumeric),
        op_btn("Dedupe", TransformOp::DeduplicateLines),
        op_btn("No Empty", TransformOp::RemoveEmptyLines),
        op_btn("Trim End", TransformOp::TrimTrailingWhitespace),
        op_btn("Reverse", TransformOp::ReverseLines),
        op_btn("Number", TransformOp::AddLineNumbers),
        op_btn("Join ,", TransformOp::JoinLinesWithComma),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    let row2 = row![
        text("Case:")
            .size(11)
            .color(c.accent)
            .width(Length::Fixed(52.0)),
        op_btn("UPPER", TransformOp::Uppercase),
        op_btn("lower", TransformOp::Lowercase),
        op_btn("Title", TransformOp::TitleCase),
        op_btn("camelCase", TransformOp::CamelCase),
        op_btn("snake_case", TransformOp::SnakeCase),
        op_btn("kebab-case", TransformOp::KebabCase),
        space::horizontal().width(10),
        text("Format:").size(11).color(c.accent),
        op_btn("JSON Pretty", TransformOp::JsonPretty),
        op_btn("JSON Min", TransformOp::JsonMinify),
        op_btn("XML Pretty", TransformOp::XmlPretty),
        op_btn("XML Min", TransformOp::XmlMinify),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    let row3 = row![
        text("Dev:")
            .size(11)
            .color(c.accent)
            .width(Length::Fixed(52.0)),
        op_btn("B64 Enc", TransformOp::Base64Encode),
        op_btn("B64 Dec", TransformOp::Base64Decode),
        op_btn("URL Enc", TransformOp::UrlEncode),
        op_btn("URL Dec", TransformOp::UrlDecode),
        op_btn("HTML Esc", TransformOp::HtmlEscape),
        op_btn("JWT Inspect", TransformOp::JwtDecode),
        op_btn("MD5", TransformOp::HashMd5),
        op_btn("SHA-256", TransformOp::HashSha256),
        op_btn("Unix→ISO", TransformOp::UnixTimestampToIso),
        op_btn("Now UTC", TransformOp::InsertCurrentIsoUtc),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    container(
        column![row1, row2, row3]
            .spacing(5)
            .padding(Padding::from([7, 12])),
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(c.panel_bg)),
        ..Default::default()
    })
    .into()
}

fn sidebar_view(app: &AetherisApp, c: ChromeColors) -> Element<'_, Message> {
    let switcher = row![
        chip_button(
            "Files",
            app.sidebar_panel == SidebarPanel::Explorer,
            c,
            Message::SelectSidebarPanel(SidebarPanel::Explorer),
        ),
        chip_button(
            "Search",
            app.sidebar_panel == SidebarPanel::Search,
            c,
            Message::SelectSidebarPanel(SidebarPanel::Search),
        ),
        chip_button(
            "Marks",
            app.sidebar_panel == SidebarPanel::Bookmarks,
            c,
            Message::SelectSidebarPanel(SidebarPanel::Bookmarks),
        ),
    ]
    .spacing(4)
    .padding(Padding::from([6, 8]));

    let panel_body: Element<'_, Message> = match app.sidebar_panel {
        SidebarPanel::Explorer => sidebar_explorer(app, c),
        SidebarPanel::Search => sidebar_search(app, c),
        SidebarPanel::Bookmarks => sidebar_bookmarks_and_macros(app, c),
    };

    container(
        column![switcher, rule::horizontal(1), panel_body]
            .width(Length::Fixed(260.0))
            .height(Length::Fill),
    )
    .style(move |_| container::Style {
        background: Some(Background::Color(c.panel_bg)),
        ..Default::default()
    })
    .into()
}

fn sidebar_explorer(app: &AetherisApp, c: ChromeColors) -> Element<'_, Message> {
    let header_label = app
        .workspace_root
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("No Folder Open");

    let top = column![
        row![
            text(header_label)
                .size(12)
                .color(c.text)
                .width(Length::Fill),
            subtle_button("Open", c, Message::OpenFolderDialog),
            subtle_button("↻", c, Message::RefreshWorkspace),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
        text_input("Filter files...", &app.workspace_filter)
            .on_input(Message::WorkspaceFilterChanged)
            .size(12),
    ]
    .spacing(6)
    .padding(8);

    let filter = app.workspace_filter.trim().to_ascii_lowercase();
    let mut list = column![].spacing(1).padding(Padding::from([0, 6]));

    if app.workspace_entries.is_empty() {
        list = list.push(
            text("Open a folder to browse files as a workspace.")
                .size(12)
                .color(c.text_muted),
        );
    } else {
        for entry in &app.workspace_entries {
            if !filter.is_empty() && !entry.display_name.to_ascii_lowercase().contains(&filter) {
                continue;
            }
            let indent = "  ".repeat(entry.depth.min(6));
            let icon = if entry.is_dir { "▸ " } else { "  " };
            let label = format!("{indent}{icon}{}", entry.display_name);
            let fg = if entry.is_dir { c.accent } else { c.text };

            let btn = button(text(label).size(12).color(fg))
                .width(Length::Fill)
                .padding(Padding::from([3, 6]))
                .style(move |_, status| {
                    let bg = match status {
                        button::Status::Hovered => c.tab_active_bg,
                        _ => Color::TRANSPARENT,
                    };
                    button::Style {
                        background: Some(Background::Color(bg)),
                        text_color: fg,
                        border: Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                })
                .on_press(Message::OpenPath(entry.path.clone()));

            list = list.push(btn);
        }
    }

    column![top, scrollable(list).height(Length::Fill)]
        .height(Length::Fill)
        .into()
}

fn sidebar_search(app: &AetherisApp, c: ChromeColors) -> Element<'_, Message> {
    let controls = column![
        text("Find in Files (Ripgrep Core)")
            .size(12)
            .color(c.accent),
        text_input("Query or regex...", &app.workspace_search_query)
            .on_input(Message::WorkspaceSearchQueryChanged)
            .on_submit(Message::RunWorkspaceSearch)
            .size(12),
        row![
            chip_button(
                "Aa",
                app.search_opts.case_sensitive,
                c,
                Message::ToggleSearchCase
            ),
            chip_button(
                "\\b",
                app.search_opts.whole_word,
                c,
                Message::ToggleSearchWord
            ),
            chip_button(
                ".*",
                app.search_opts.use_regex,
                c,
                Message::ToggleSearchRegex
            ),
            space::horizontal().width(Length::Fill),
            subtle_button("Search", c, Message::RunWorkspaceSearch),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
    ]
    .spacing(6)
    .padding(8);

    let mut list = column![].spacing(4).padding(Padding::from([0, 8]));
    if app.workspace_search_hits.is_empty() {
        list = list.push(
            text("Press Search to scan the open workspace folder (respects .gitignore).")
                .size(11)
                .color(c.text_muted),
        );
    } else {
        for hit in &app.workspace_search_hits {
            let fname = hit
                .path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            let header = format!("{fname}:{}", hit.line + 1);
            let card = button(
                column![
                    text(header).size(11).color(c.accent),
                    text(hit.snippet.clone()).size(11).color(c.text),
                ]
                .spacing(2),
            )
            .width(Length::Fill)
            .padding(Padding::from([5, 6]))
            .style(move |_, status| {
                let bg = match status {
                    button::Status::Hovered => c.tab_active_bg,
                    _ => c.tab_bg,
                };
                button::Style {
                    background: Some(Background::Color(bg)),
                    text_color: c.text,
                    border: Border {
                        color: c.border,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
            .on_press(Message::OpenSearchHit(hit.path.clone(), hit.line));

            list = list.push(card);
        }
    }

    column![controls, scrollable(list).height(Length::Fill)]
        .height(Length::Fill)
        .into()
}

fn sidebar_bookmarks_and_macros(app: &AetherisApp, c: ChromeColors) -> Element<'_, Message> {
    let mut col = column![
        text("Bookmarks").size(12).color(c.accent),
        row![
            subtle_button("Toggle", c, Message::ToggleBookmark),
            subtle_button("Extract", c, Message::ExtractBookmarkedToNewTab),
            subtle_button("Delete", c, Message::DeleteBookmarkedLines),
        ]
        .spacing(4),
    ]
    .spacing(6)
    .padding(8);

    if let Some(doc) = app.active_doc() {
        if doc.bookmarks.is_empty() {
            col = col.push(
                text("No bookmarks yet. Press Cmd/Ctrl+K to mark the active line.")
                    .size(11)
                    .color(c.text_muted),
            );
        } else {
            for &line_idx in &doc.bookmarks {
                let preview = doc
                    .content
                    .line(line_idx)
                    .map(|l| l.text.trim().chars().take(28).collect::<String>())
                    .unwrap_or_default();
                let label = format!("★ L{}: {preview}", line_idx + 1);
                col = col.push(
                    button(text(label).size(11).color(c.text))
                        .width(Length::Fill)
                        .padding(Padding::from([4, 6]))
                        .style(move |_, _| button::Style {
                            background: Some(Background::Color(c.tab_bg)),
                            text_color: c.text,
                            border: Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .on_press(Message::JumpToBookmark(line_idx)),
                );
            }
        }
    }

    col = col
        .push(rule::horizontal(1))
        .push(text("Saved Macros").size(12).color(c.accent))
        .push(
            row![
                text_input("Macro name...", &app.macro_name_input)
                    .on_input(Message::MacroNameChanged)
                    .size(11)
                    .width(Length::Fill),
                subtle_button("Save", c, Message::SaveCurrentMacro),
            ]
            .spacing(4),
        );

    let saved = app.macro_recorder.saved_macros();
    if saved.is_empty() {
        col = col.push(
            text("Record a macro from the top bar, then save it here for 1-click replay.")
                .size(11)
                .color(c.text_muted),
        );
    } else {
        for (idx, m) in saved.iter().enumerate() {
            col = col.push(
                button(
                    text(format!("▶ {} ({} steps)", m.name, m.steps.len()))
                        .size(11)
                        .color(c.text),
                )
                .width(Length::Fill)
                .padding(Padding::from([4, 6]))
                .style(move |_, _| button::Style {
                    background: Some(Background::Color(c.tab_bg)),
                    text_color: c.text,
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .on_press(Message::ReplaySavedMacro(idx)),
            );
        }
    }

    scrollable(col).height(Length::Fill).into()
}

fn main_workspace(app: &AetherisApp, c: ChromeColors) -> Element<'_, Message> {
    let Some(doc) = app.active_doc() else {
        return container(text("No open tab").color(c.text_muted))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    };

    let primary: Element<'_, Message> = match doc.view_mode {
        ViewMode::Editor => code_editor_pane(app, doc, c, false),
        ViewMode::MarkdownSplit => markdown_split_pane(app, doc, c),
        ViewMode::DiffView => diff_view_pane(app, doc, c),
        ViewMode::CsvGrid => csv_grid_pane(doc, c),
        ViewMode::HexView => hex_view_pane(app, doc, c),
        ViewMode::LargeFileFastView => large_file_pane(app, doc, c),
    };

    if app.split_enabled
        && doc.view_mode == ViewMode::Editor
        && let Some(split_idx) = app.split_tab
        && let Some(split_doc) = app.tabs.get(split_idx)
    {
        let split_header = build_split_header(app, split_idx, c);
        let right_pane = column![
            split_header,
            rule::horizontal(1),
            code_editor_pane(app, split_doc, c, true),
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        return row![
            container(primary).width(Length::Fill).height(Length::Fill),
            rule::vertical(1),
            right_pane,
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    }

    primary
}

fn build_split_header(
    app: &AetherisApp,
    split_idx: usize,
    c: ChromeColors,
) -> Element<'_, Message> {
    let mut r = row![text("Split:").size(11).color(c.accent)]
        .spacing(4)
        .align_y(Alignment::Center)
        .padding(Padding::from([4, 8]));

    for (i, t) in app.tabs.iter().enumerate() {
        let selected = i == split_idx;
        let bg = if selected { c.accent } else { c.tab_bg };
        let fg = if selected {
            if app.theme.is_dark() {
                Color::from_rgb8(15, 17, 26)
            } else {
                Color::WHITE
            }
        } else {
            c.text_muted
        };
        r = r.push(
            button(text(t.title.clone()).size(11).color(fg))
                .padding(Padding::from([3, 7]))
                .style(move |_, _| button::Style {
                    background: Some(Background::Color(bg)),
                    text_color: fg,
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .on_press(Message::SelectSplitTab(i)),
        );
    }

    container(r)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(c.panel_bg)),
            ..Default::default()
        })
        .into()
}

fn code_editor_pane<'a>(
    app: &'a AetherisApp,
    doc: &'a DocumentTab,
    c: ChromeColors,
    is_split: bool,
) -> Element<'a, Message> {
    let wrapping = if app.word_wrap {
        iced::widget::text::Wrapping::WordOrGlyph
    } else {
        iced::widget::text::Wrapping::None
    };

    let editor = text_editor(&doc.content)
        .placeholder(
            "Start typing, drop a file here, or press Cmd/Ctrl+Shift+P for the Command Palette...",
        )
        .font(Font::MONOSPACE)
        .size(f32::from(app.font_size))
        .padding(Padding::from([10, 12]))
        .wrapping(wrapping)
        .height(Length::Fill)
        .highlight(
            doc.language.syntax_extension(),
            app.theme.highlighter_theme(),
        )
        .on_action(if is_split {
            Message::SplitEditorAction
        } else {
            Message::EditorAction
        });

    if !app.show_line_numbers {
        return container(editor)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    let line_count = doc.line_count().clamp(1, 5000);
    let mut gutter_str = String::with_capacity(line_count * 6);
    for i in 0..line_count {
        let mark = if doc.bookmarks.contains(&i) {
            "★"
        } else {
            " "
        };
        gutter_str.push_str(&format!("{mark}{:>4}\n", i + 1));
    }

    let gutter = container(
        scrollable(
            text(gutter_str)
                .font(Font::MONOSPACE)
                .size(f32::from(app.font_size))
                .color(c.text_muted),
        )
        .height(Length::Fill),
    )
    .width(Length::Fixed(58.0))
    .height(Length::Fill)
    .padding(Padding::from([10, 6]))
    .style(move |_| container::Style {
        background: Some(Background::Color(c.panel_bg)),
        ..Default::default()
    });

    row![gutter, rule::vertical(1), editor]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn markdown_split_pane<'a>(
    app: &'a AetherisApp,
    doc: &'a DocumentTab,
    c: ChromeColors,
) -> Element<'a, Message> {
    let left = code_editor_pane(app, doc, c, false);

    let md_settings =
        markdown::Settings::with_text_size(f32::from(app.font_size), app.iced_theme());
    let rendered =
        markdown::view(&doc.markdown_items, md_settings).map(Message::MarkdownLinkClicked);

    let preview = container(
        column![
            container(
                text("Live Native Markdown Preview (Zero WebView)")
                    .size(11)
                    .color(c.accent)
            )
            .padding(Padding::from([6, 12]))
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(c.panel_bg)),
                ..Default::default()
            }),
            rule::horizontal(1),
            scrollable(container(rendered).padding(16).width(Length::Fill)).height(Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    row![
        container(left).width(Length::Fill).height(Length::Fill),
        rule::vertical(1),
        preview,
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn diff_view_pane<'a>(
    app: &'a AetherisApp,
    doc: &'a DocumentTab,
    c: ChromeColors,
) -> Element<'a, Message> {
    let mut bar = row![text("Compare against:").size(12).color(c.accent)]
        .spacing(6)
        .align_y(Alignment::Center)
        .padding(Padding::from([6, 12]));

    for t in &app.tabs {
        let selected = doc.diff_target_id == Some(t.id);
        let bg = if selected { c.accent } else { c.tab_bg };
        let fg = if selected {
            if app.theme.is_dark() {
                Color::from_rgb8(15, 17, 26)
            } else {
                Color::WHITE
            }
        } else {
            c.text
        };
        bar = bar.push(
            button(text(t.title.clone()).size(11).color(fg))
                .padding(Padding::from([3, 8]))
                .style(move |_, _| button::Style {
                    background: Some(Background::Color(bg)),
                    text_color: fg,
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .on_press(Message::SetDiffTarget(t.id)),
        );
    }

    bar = bar
        .push(space::horizontal().width(Length::Fill))
        .push(subtle_button("Refresh", c, Message::RefreshDiff))
        .push(subtle_button("Export .patch", c, Message::ExportDiffPatch));

    let mut rows_col = column![].spacing(1).padding(8);

    if let Some(ref report) = doc.diff_report {
        rows_col = rows_col.push(
            row![
                text(format!("+{} added", report.added_lines))
                    .size(12)
                    .color(c.diff_add_fg),
                text(format!("-{} removed", report.removed_lines))
                    .size(12)
                    .color(c.diff_del_fg),
                text(format!("{} unchanged", report.unchanged_lines))
                    .size(12)
                    .color(c.text_muted),
            ]
            .spacing(14)
            .padding(Padding {
                top: 0.0,
                right: 4.0,
                bottom: 8.0,
                left: 4.0,
            }),
        );

        for r in report.rows.iter().take(1500) {
            let (bg, fg, sign) = match r.tag {
                DiffTag::Equal => (Color::TRANSPARENT, c.text, " "),
                DiffTag::Delete => (c.diff_del_bg, c.diff_del_fg, "-"),
                DiffTag::Insert => (c.diff_add_bg, c.diff_add_fg, "+"),
            };

            let lno = r
                .left_line_no
                .map(|n| format!("{n:>4}"))
                .unwrap_or_else(|| "    ".into());
            let rno = r
                .right_line_no
                .map(|n| format!("{n:>4}"))
                .unwrap_or_else(|| "    ".into());

            let diff_row = container(
                row![
                    text(lno)
                        .font(Font::MONOSPACE)
                        .size(12)
                        .color(c.text_muted)
                        .width(Length::Fixed(42.0)),
                    text(r.left_text.clone())
                        .font(Font::MONOSPACE)
                        .size(12)
                        .color(fg)
                        .width(Length::Fill),
                    text(sign)
                        .font(Font::MONOSPACE)
                        .size(12)
                        .color(fg)
                        .width(Length::Fixed(18.0)),
                    text(rno)
                        .font(Font::MONOSPACE)
                        .size(12)
                        .color(c.text_muted)
                        .width(Length::Fixed(42.0)),
                    text(r.right_text.clone())
                        .font(Font::MONOSPACE)
                        .size(12)
                        .color(fg)
                        .width(Length::Fill),
                ]
                .spacing(6),
            )
            .width(Length::Fill)
            .padding(Padding::from([2, 6]))
            .style(move |_| container::Style {
                background: Some(Background::Color(bg)),
                ..Default::default()
            });

            rows_col = rows_col.push(diff_row);
        }
    }

    column![
        container(bar)
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(c.panel_bg)),
                ..Default::default()
            }),
        rule::horizontal(1),
        scrollable(rows_col).height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn csv_grid_pane<'a>(doc: &'a DocumentTab, c: ChromeColors) -> Element<'a, Message> {
    let delim_label = doc
        .csv_table
        .as_ref()
        .map(|t| t.delimiter_label())
        .unwrap_or("Unknown");
    let row_count = doc.csv_table.as_ref().map(|t| t.rows.len()).unwrap_or(0);

    let toolbar = container(
        row![
            text(format!("CSV / TSV Grid — {delim_label} · {row_count} rows"))
                .size(12)
                .color(c.accent),
            space::horizontal().width(12),
            text_input("Filter rows...", &doc.csv_filter)
                .on_input(Message::CsvFilterChanged)
                .size(12)
                .width(Length::Fixed(240.0)),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .padding(Padding::from([6, 12])),
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(c.panel_bg)),
        ..Default::default()
    });

    if let Some(ref err) = doc.csv_error {
        return column![
            toolbar,
            rule::horizontal(1),
            container(text(err).color(c.danger)).padding(16),
        ]
        .into();
    }

    let Some(ref table) = doc.csv_table else {
        return column![
            toolbar,
            rule::horizontal(1),
            container(text("No CSV data").color(c.text_muted)).padding(16),
        ]
        .into();
    };

    let mut header_row = row![
        text("#")
            .font(Font::MONOSPACE)
            .size(12)
            .color(c.text_muted)
            .width(Length::Fixed(46.0))
    ]
    .spacing(6)
    .padding(Padding::from([6, 10]));

    for (col_idx, h) in table.headers.iter().enumerate() {
        let arrow = if doc.csv_sort_col == Some(col_idx) {
            if doc.csv_sort_asc { " ▲" } else { " ▼" }
        } else {
            ""
        };
        let col_px =
            (table.col_widths.get(col_idx).copied().unwrap_or(14) as f32 * 8.5).clamp(90.0, 260.0);
        header_row = header_row.push(
            button(
                text(format!("{h}{arrow}"))
                    .font(Font::MONOSPACE)
                    .size(12)
                    .color(c.accent),
            )
            .width(Length::Fixed(col_px))
            .padding(Padding::from([3, 6]))
            .style(move |_, _| button::Style {
                background: Some(Background::Color(c.tab_bg)),
                text_color: c.accent,
                border: Border {
                    color: c.border,
                    width: 1.0,
                    radius: 3.0.into(),
                },
                ..Default::default()
            })
            .on_press(Message::CsvSortColumn(col_idx)),
        );
    }

    let visible = table.filtered_rows(&doc.csv_filter, doc.csv_sort_col, doc.csv_sort_asc, 400);
    let mut rows_col = column![header_row, rule::horizontal(1)].spacing(2);

    for (r_idx, r_cells) in visible.into_iter().enumerate() {
        let row_bg = if r_idx % 2 == 0 {
            Color::TRANSPARENT
        } else {
            c.panel_bg
        };
        let mut line = row![
            text(format!("{:>4}", r_idx + 1))
                .font(Font::MONOSPACE)
                .size(12)
                .color(c.text_muted)
                .width(Length::Fixed(46.0))
        ]
        .spacing(6);

        for (col_idx, cell) in r_cells.iter().enumerate() {
            let col_px = (table.col_widths.get(col_idx).copied().unwrap_or(14) as f32 * 8.5)
                .clamp(90.0, 260.0);
            line = line.push(
                text(cell.clone())
                    .font(Font::MONOSPACE)
                    .size(12)
                    .color(c.text)
                    .width(Length::Fixed(col_px)),
            );
        }

        rows_col = rows_col.push(container(line).padding(Padding::from([3, 10])).style(
            move |_| container::Style {
                background: Some(Background::Color(row_bg)),
                ..Default::default()
            },
        ));
    }

    column![
        toolbar,
        rule::horizontal(1),
        scrollable(rows_col)
            .direction(scrollable::Direction::Both {
                vertical: scrollable::Scrollbar::default(),
                horizontal: scrollable::Scrollbar::default(),
            })
            .height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn hex_view_pane<'a>(
    app: &'a AetherisApp,
    doc: &'a DocumentTab,
    c: ChromeColors,
) -> Element<'a, Message> {
    let toolbar = container(
        row![
            text(format!(
                "Hex / Binary Inspector — Offset 0x{:08X}",
                doc.hex_offset
            ))
            .size(12)
            .color(c.accent),
            space::horizontal().width(10),
            subtle_button("◀ Prev 2 KiB", c, Message::HexPrevPage),
            subtle_button("Next 2 KiB ▶", c, Message::HexNextPage),
            text_input("Offset (e.g. 0x1000 or 4096)...", &app.hex_jump_input)
                .on_input(Message::HexOffsetInputChanged)
                .on_submit(Message::HexJumpOffset)
                .size(12)
                .width(Length::Fixed(200.0)),
            subtle_button("Jump", c, Message::HexJumpOffset),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .padding(Padding::from([6, 12])),
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(c.panel_bg)),
        ..Default::default()
    });

    let mut dump = String::with_capacity(doc.hex_rows.len() * 82 + 80);
    dump.push_str("OFFSET     00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F  ASCII\n");
    dump.push_str(
        "------------------------------------------------------------------------------\n",
    );
    for row in &doc.hex_rows {
        dump.push_str(&format!(
            "{:08X}   {:<48}  {}\n",
            row.offset, row.hex_columns, row.ascii_column
        ));
    }

    column![
        toolbar,
        rule::horizontal(1),
        scrollable(
            container(
                text(dump)
                    .font(Font::MONOSPACE)
                    .size(f32::from(app.font_size))
                    .color(c.text)
            )
            .padding(12)
        )
        .height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn large_file_pane<'a>(
    app: &'a AetherisApp,
    doc: &'a DocumentTab,
    c: ChromeColors,
) -> Element<'a, Message> {
    let total = doc.large_file.as_ref().map(|r| r.len()).unwrap_or(0);
    let total_mb = total as f64 / (1024.0 * 1024.0);
    let (w_start, w_end) = doc.large_file_window;

    let toolbar = container(
        row![
            text(format!(
                "Fast View (memmap2) — {total_mb:.2} MiB · Bytes {w_start}..{w_end}"
            ))
            .size(12)
            .color(c.accent),
            space::horizontal().width(8),
            subtle_button("◀ Prev", c, Message::LargeFilePrevPage),
            subtle_button("Next ▶", c, Message::LargeFileNextPage),
            subtle_button("0%", c, Message::LargeFileJumpPercent(0)),
            subtle_button("25%", c, Message::LargeFileJumpPercent(25)),
            subtle_button("50%", c, Message::LargeFileJumpPercent(50)),
            subtle_button("75%", c, Message::LargeFileJumpPercent(75)),
            subtle_button("End", c, Message::LargeFileJumpPercent(100)),
            space::horizontal().width(8),
            text_input("Find in mapped file...", &app.large_file_search_input)
                .on_input(Message::LargeFileSearchInputChanged)
                .on_submit(Message::LargeFileFindNext)
                .size(12)
                .width(Length::Fixed(180.0)),
            subtle_button("Find Next", c, Message::LargeFileFindNext),
        ]
        .spacing(5)
        .align_y(Alignment::Center)
        .padding(Padding::from([6, 12])),
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(c.panel_bg)),
        ..Default::default()
    });

    column![
        toolbar,
        rule::horizontal(1),
        scrollable(
            container(
                text(doc.large_file_chunk.clone())
                    .font(Font::MONOSPACE)
                    .size(f32::from(app.font_size))
                    .color(c.text)
            )
            .padding(12)
        )
        .height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn status_bar(app: &AetherisApp, c: ChromeColors) -> Element<'_, Message> {
    let (line, col, lines_total, lang, enc, eol) = if let Some(doc) = app.active_doc() {
        let (ln, cl) = doc.cursor_line_col();
        (
            ln,
            cl,
            doc.line_count(),
            doc.language.label(),
            doc.encoding.label(),
            doc.line_ending.short_label(),
        )
    } else {
        (1, 1, 0, "Plain Text", "UTF-8", "LF")
    };

    let rec_indicator = if app.macro_recorder.is_recording() {
        text("● REC ").size(11).color(c.danger).font(Font {
            weight: iced::font::Weight::Bold,
            ..Font::DEFAULT
        })
    } else {
        text("").size(11)
    };

    let wrap_label = if app.word_wrap {
        "Wrap: ON"
    } else {
        "Wrap: OFF"
    };

    let bar = row![
        rec_indicator,
        text(&app.status_message)
            .size(11)
            .color(c.text_muted)
            .width(Length::Fill),
        status_pill(
            format!("Ln {line}, Col {col} ({lines_total}L)"),
            c,
            Message::ToggleGotoLine
        ),
        status_pill(lang.to_owned(), c, Message::CycleLanguage),
        status_pill(enc.to_owned(), c, Message::CycleEncoding),
        status_pill(eol.to_owned(), c, Message::CycleLineEnding),
        status_pill(wrap_label.to_owned(), c, Message::ToggleWordWrap),
        status_pill(format!("{}px", app.font_size), c, Message::ZoomReset),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding(Padding::from([4, 10]));

    container(bar)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(c.status_bg)),
            ..Default::default()
        })
        .into()
}

fn subtle_button(label: &'static str, c: ChromeColors, msg: Message) -> Element<'static, Message> {
    button(text(label).size(12).color(c.text))
        .padding(Padding::from([4, 9]))
        .style(move |_, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => c.tab_active_bg,
                _ => c.tab_bg,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: c.text,
                border: Border {
                    color: c.border,
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            }
        })
        .on_press(msg)
        .into()
}

fn chip_button(
    label: &'static str,
    active: bool,
    c: ChromeColors,
    msg: Message,
) -> Element<'static, Message> {
    let bg = if active { c.accent } else { c.tab_bg };
    let fg = if active {
        Color::from_rgb8(15, 17, 26)
    } else {
        c.text
    };
    let border_col = if active { c.accent } else { c.border };

    button(text(label).size(12).color(fg))
        .padding(Padding::from([4, 9]))
        .style(move |_, status| {
            let effective_bg = match status {
                button::Status::Hovered if !active => c.tab_active_bg,
                _ => bg,
            };
            button::Style {
                background: Some(Background::Color(effective_bg)),
                text_color: fg,
                border: Border {
                    color: border_col,
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            }
        })
        .on_press(msg)
        .into()
}

fn status_pill(label: String, c: ChromeColors, msg: Message) -> Element<'static, Message> {
    button(text(label).size(11).color(c.text))
        .padding(Padding::from([2, 7]))
        .style(move |_, status| {
            let bg = match status {
                button::Status::Hovered => c.tab_active_bg,
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: c.text,
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .on_press(msg)
        .into()
}
