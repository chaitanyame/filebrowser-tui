use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::state::{App, MessageLevel};

/// Render the tab bar showing all tabs
pub fn render_tab_bar(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let mut spans = Vec::new();

    // Calculate available width for tabs
    let available_width = area.width.saturating_sub(2) as usize;

    // Render each tab
    let mut current_width = 0;
    for (idx, tab) in app.tabs.iter().enumerate() {
        let is_current = idx == app.current_tab;

        // Tab format: [1: folder_name] or [folder_name]
        let tab_text = if app.tabs.len() <= 9 {
            format!("{}:{}", idx + 1, tab.display_name)
        } else {
            tab.display_name.clone()
        };

        let tab_width = tab_text.len() + 3; // +3 for brackets and space

        // Check if we have space for this tab
        if current_width + tab_width > available_width && idx > 0 {
            // No more space, add ellipsis and break
            spans.push(Span::styled("...", Style::default().fg(Color::Gray)));
            break;
        }

        // Style for current vs inactive tabs
        let style = if is_current {
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Gray)
                .bg(Color::DarkGray)
        };

        // Add tab with brackets
        spans.push(Span::styled("[", Style::default().fg(Color::DarkGray)));
        spans.push(Span::styled(&tab_text, style));
        spans.push(Span::styled("]", Style::default().fg(Color::DarkGray)));

        // Add space between tabs
        if idx < app.tabs.len() - 1 {
            spans.push(Span::raw(" "));
        }

        current_width += tab_width;
    }

    let text = Text::from(Line::from(spans));
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL | Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Blue))
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Render the header showing current path and drives
pub fn render_header(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let path_str = app.current_path().display().to_string();

    // Truncate path if too long
    let display_path = if path_str.len() > area.width as usize {
        let start = path_str.len() - area.width as usize + 3;
        format!("...{}", &path_str[start..])
    } else {
        path_str
    };

    // Build drive indicators
    let mut spans = vec![
        Span::styled("Path: ", Style::default().fg(Color::Cyan)),
        Span::styled(&display_path, Style::default().fg(Color::White)),
    ];

    let header = Line::from(spans);
    let text = Text::from(vec![header]);

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(" File Browser "),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);

    // Render drive indicators on a second line below the header
    let drive_line_area = area.inner(ratatui::layout::Margin::new(1, 1));
    let drive_y = drive_line_area.y + drive_line_area.height - 1;
    if drive_y < area.y + area.height {
        let drive_area = ratatui::layout::Rect {
            x: area.x + 1,
            y: drive_y,
            width: area.width.saturating_sub(2),
            height: 1,
        };
        render_drives(f, drive_area, app);
    }
}

fn render_drives(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let mut spans = vec![Span::styled("Drives: ", Style::default().fg(Color::Cyan))];

    for drive in &app.drives.drives {
        let is_current = app.current_path()
            .to_string_lossy()
            .starts_with(&format!("{}:", drive.letter));

        let style = if is_current {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        spans.push(Span::styled(
            format!("{}:", drive.letter),
            style,
        ));
        spans.push(Span::raw(" "));
    }

    let text = Text::from(Line::from(spans));
    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, area);
}

/// Render the file list
pub fn render_file_list(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let items: Vec<ListItem> = app
        .displayed_indices
        .iter()
        .enumerate()
        .map(|(idx, &file_idx)| {
            let file = &app.all_files[file_idx];
            let is_selected = idx == app.selected_index;
            let is_multiselected = app.selection.is_selected(&file.path);

            let (icon, icon_color) = if file.is_dir {
                ("📁", Color::Blue)
            } else {
                match file.extension() {
                    Some("rs") | Some("go") | Some("py") | Some("js") | Some("ts") => ("📄", Color::Yellow),
                    Some("txt") | Some("md") => ("📝", Color::White),
                    Some("zip") | Some("tar") | Some("gz") => ("📦", Color::Magenta),
                    Some("exe") | Some("dll") => ("⚙️", Color::LightRed),
                    _ => ("📄", Color::Gray),
                }
            };

            let name = file.name.clone();

            // Visual indicators
            let mut prefix = String::new();
            if is_multiselected {
                prefix.push('✓');
            }
            if file.is_symlink {
                prefix.push('@');
            }
            if file.is_hidden {
                prefix.push('h');
            }

            let style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(icon_color)
            };

            // Format: [icon] name [size] [modified]
            let size_str = file.display_size();
            let modified_str = file.display_modified();

            let content = format!(
                "{} {} {:>12} {}",
                icon,
                name,
                size_str,
                if file.is_dir { "" } else { &modified_str }
            );

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(" Files "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, area);
}

/// Render the preview pane
pub fn render_preview(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let title = if let Some(ref file) = app.preview_file {
        file.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Preview")
            .to_string()
    } else {
        "Preview".to_string()
    };

    let content = if let Some(ref text) = app.preview_content {
        Text::from(text.as_str())
    } else {
        Text::from("No preview available")
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(format!(" {} ", title)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Render the footer with status messages
pub fn render_footer(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let mut lines = Vec::new();

    // Handle ContentSearch mode differently
    if app.mode == crate::state::Mode::ContentSearch {
        render_footer_content_search(f, area, app);
        return;
    }

    // Mode indicator
    let mode_span = match app.mode {
        crate::state::Mode::Normal => Span::styled(
            "NORMAL",
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
        crate::state::Mode::Command => Span::styled(
            "COMMAND",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        crate::state::Mode::Search => Span::styled(
            "SEARCH",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        crate::state::Mode::ContentSearch => Span::styled(
            "CONTENT SEARCH",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        ),
        crate::state::Mode::BulkRename => Span::styled(
            "BULK RENAME",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
    };

    // File info
    let info_span = if let Some(file) = app.get_selected_file() {
        let count = app.displayed_indices.len();
        let selected = app.selection.count();
        Span::styled(
            format!(
                " [{}/{}] {} | {} | {}",
                app.selected_index + 1,
                count,
                file.name,
                file.display_size(),
                if file.is_dir { "DIR" } else { "FILE" }
            ),
            Style::default().fg(Color::Gray),
        )
    } else {
        Span::styled(
            format!(" [0/0] No files"),
            Style::default().fg(Color::Gray),
        )
    };

    // Message
    let msg_span = if let Some(ref msg) = app.message {
        let color = match app.message_level {
            MessageLevel::Info => Color::White,
            MessageLevel::Success => Color::Green,
            MessageLevel::Warning => Color::Yellow,
            MessageLevel::Error => Color::Red,
        };
        Span::styled(format!(" | {}", msg), Style::default().fg(color))
    } else {
        Span::raw("")
    };

    lines.push(Line::from(vec![mode_span, info_span, msg_span]));

    // Command/search input
    if app.mode != crate::state::Mode::Normal {
        let prompt = if app.mode == crate::state::Mode::Search {
            "/"
        } else {
            ":"
        };
        lines.push(Line::from(vec![
            Span::styled(prompt, Style::default().fg(Color::Cyan)),
            Span::styled(
                &app.command_input,
                Style::default().fg(Color::White),
            ),
            Span::styled("█", Style::default().fg(Color::White)), // cursor
        ]));
    } else {
        // Keybinding hints with undo/redo status
        let undo_hint = if app.can_undo() {
            format!("^Z:Undo({})", app.history.undo_count())
        } else {
            "^Z:Undo".to_string()
        };

        let redo_hint = if app.can_redo() {
            format!("^Y:Redo({})", app.history.redo_count())
        } else {
            "^Y:Redo".to_string()
        };

        let hints = if app.split_view {
            Span::styled(
                format!(
                    " q:Quit | hjkl/↑↓:Move | Enter:Open | Backspace:Up | Space:Select | /:Search | p:Preview | .:Hidden | F7:Mkdir | F8:Delete | F2:Rename | Tab:SwitchPane | F5:Copy | F6:Move | ^P:SplitOff | {} | {} ",
                    undo_hint, redo_hint
                ),
                Style::default().fg(Color::DarkGray),
            )
        } else {
            Span::styled(
                format!(
                    " q:Quit | hjkl/↑↓:Move | Enter:Open | Backspace:Up | Space:Select | /:Search | p:Preview | .:Hidden | F7:Mkdir | F8:Delete | F2:Rename | Ctrl+T:NewTab | Ctrl+W:CloseTab | ^P:SplitView | {} | {} ",
                    undo_hint, redo_hint
                ),
                Style::default().fg(Color::DarkGray),
            )
        };
        lines.push(Line::from(hints));
    }

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Render a confirmation dialog
pub fn render_confirm_dialog(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    if let Some(ref dialog) = app.confirm_dialog {
        let (title, message) = match dialog {
            crate::state::ConfirmDialog::Delete { files } => (
                "Confirm Delete",
                format!("Delete {} item(s)? (y/N)", files.len()),
            ),
            crate::state::ConfirmDialog::Overwrite { source, target } => (
                "Confirm Overwrite",
                format!(
                    "Overwrite {} with {}? (y/N)",
                    target.file_name().unwrap_or_default().to_string_lossy(),
                    source.file_name().unwrap_or_default().to_string_lossy()
                ),
            ),
        };

        // Center the dialog
        let width = 50.min(area.width);
        let height = 5.min(area.height);
        let x = (area.width - width) / 2 + area.x;
        let y = (area.height - height) / 2 + area.y;

        let dialog_area = ratatui::layout::Rect {
            x,
            y,
            width,
            height,
        };

        let text = Text::from(vec![
            Line::from(""),
            Line::from(message),
        ]);

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .title(title),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, dialog_area);
    }
}

/// Render the content search results panel
pub fn render_content_search_results(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let title = if let Some(ref query) = app.content_search_query {
        format!(" Search Results: '{}' ", query)
    } else {
        " Search Results ".to_string()
    };

    if app.content_search_results.is_empty() {
        // Show empty state or searching message
        let message = if app.content_search_in_progress {
            "Searching..."
        } else {
            "No matches found. Press Ctrl+G to start a new search."
        };

        let paragraph = Paragraph::new(message)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(title),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
        return;
    }

    // Build list items from search results
    let items: Vec<ListItem> = app.content_search_results
        .iter()
        .enumerate()
        .skip(app.content_search_scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .map(|(idx, result)| {
            let is_selected = idx == app.content_search_selected_index;

            // Get relative path for display
            let relative_path = result.relative_path(app.current_path());

            // Format: file_path:line_number: line_content
            let line_number = format!("{}:", result.line_number);
            let line_preview = if result.line_content.len() > 80 {
                format!("{}...", &result.line_content[..77])
            } else {
                result.line_content.clone()
            };

            let content = format!(
                "{}:{} {}",
                relative_path,
                line_number,
                line_preview
            );

            // Highlight the matched text
            let mut spans = Vec::new();
            let mut last_end = 0;

            // Add the path part (before the match)
            spans.push(Span::styled(
                format!("{}:{} ", relative_path, line_number),
                Style::default().fg(Color::Cyan),
            ));

            // Add the line content with highlighted match
            if result.match_start < result.line_content.len() && result.match_end <= result.line_content.len() {
                // Text before the match
                if result.match_start > 0 {
                    spans.push(Span::styled(
                        &result.line_content[0..result.match_start],
                        Style::default().fg(Color::Gray),
                    ));
                }

                // The matched text
                spans.push(Span::styled(
                    &result.line_content[result.match_start..result.match_end],
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));

                // Text after the match
                if result.match_end < result.line_content.len() {
                    spans.push(Span::styled(
                        &result.line_content[result.match_end..],
                        Style::default().fg(Color::Gray),
                    ));
                }
            } else {
                // No valid match range, show entire line
                spans.push(Span::styled(
                    line_preview,
                    Style::default().fg(Color::Gray),
                ));
            }

            let style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(spans)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(title),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, area);
}

/// Update the footer to show content search mode
pub fn render_footer_content_search(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let mut lines = Vec::new();

    // Mode indicator
    let mode_span = Span::styled(
        "CONTENT SEARCH",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    );

    // Search progress/result count
    let info_span = if app.content_search_in_progress {
        Span::styled(
            " | Searching...",
            Style::default().fg(Color::Yellow),
        )
    } else {
        Span::styled(
            format!(" | {} results", app.content_search_results.len()),
            Style::default().fg(Color::Gray),
        )
    };

    // Current selection info
    let selection_span = if !app.content_search_results.is_empty() {
        let result = &app.content_search_results[app.content_search_selected_index];
        Span::styled(
            format!(
                " | [{}/{}] {}:{}",
                app.content_search_selected_index + 1,
                app.content_search_results.len(),
                result.file_path.file_name().unwrap_or_default().to_string_lossy(),
                result.line_number
            ),
            Style::default().fg(Color::White),
        )
    } else {
        Span::raw("")
    };

    // Message
    let msg_span = if let Some(ref msg) = app.message {
        let color = match app.message_level {
            MessageLevel::Info => Color::White,
            MessageLevel::Success => Color::Green,
            MessageLevel::Warning => Color::Yellow,
            MessageLevel::Error => Color::Red,
        };
        Span::styled(format!(" | {}", msg), Style::default().fg(color))
    } else {
        Span::raw("")
    };

    lines.push(Line::from(vec![mode_span, info_span, selection_span, msg_span]));

    // Search input or keybinding hints
    if app.content_search_query.is_some() && app.command_input.is_empty() {
        // Show input prompt
        lines.push(Line::from(vec![
            Span::styled("Search: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                &app.command_input,
                Style::default().fg(Color::White),
            ),
            Span::styled("█", Style::default().fg(Color::White)), // cursor
        ]));
    } else {
        // Keybinding hints
        let hints = Span::styled(
            " Enter:Execute | Esc:Cancel | ↑↓:Navigate | Enter:Open file | Ctrl+G:New search ",
            Style::default().fg(Color::DarkGray),
        );
        lines.push(Line::from(hints));
    }

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Render a single pane in split view
pub fn render_pane(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    pane: &crate::state::Pane,
    is_active: bool,
    title: &str,
) {
    let items: Vec<ListItem> = pane
        .displayed_indices
        .iter()
        .enumerate()
        .map(|(idx, &file_idx)| {
            let file = &pane.files[file_idx];
            let is_selected = idx == pane.selected_index;
            let is_multiselected = pane.selection.is_selected(&file.path);

            let (icon, icon_color) = if file.is_dir {
                ("📁", Color::Blue)
            } else {
                match file.extension() {
                    Some("rs") | Some("go") | Some("py") | Some("js") | Some("ts") => ("📄", Color::Yellow),
                    Some("txt") | Some("md") => ("📝", Color::White),
                    Some("zip") | Some("tar") | Some("gz") => ("📦", Color::Magenta),
                    Some("exe") | Some("dll") => ("⚙️", Color::LightRed),
                    _ => ("📄", Color::Gray),
                }
            };

            let name = file.name.clone();

            // Visual indicators
            let mut prefix = String::new();
            if is_multiselected {
                prefix.push('✓');
            }
            if file.is_symlink {
                prefix.push('@');
            }
            if file.is_hidden {
                prefix.push('h');
            }

            let base_style = if is_active {
                if is_selected {
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(icon_color)
                }
            } else {
                // Inactive pane styling
                if is_selected {
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::White)
                        .add_modifier(Modifier::DIM)
                } else {
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM)
                }
            };

            // Format: [icon] name [size] [modified]
            let size_str = file.display_size();
            let modified_str = file.display_modified();

            let content = format!(
                "{} {} {:>12} {}",
                icon,
                name,
                size_str,
                if file.is_dir { "" } else { &modified_str }
            );

            ListItem::new(content).style(base_style)
        })
        .collect();

    let border_style = if is_active {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(format!(" {} ", title)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, area);
}

/// Render both panes in split view
pub fn render_split_panes(f: &mut Frame, layout: &crate::ui::layout::LayoutParts, app: &App) {
    if let (Some(left_area), Some(right_area)) = (layout.left_pane_area, layout.right_pane_area) {
        let is_left_active = app.active_pane == crate::state::ActivePane::Left;

        // Render left pane
        let left_title = format!(
            " {} {} ",
            app.left_pane.display_name(),
            if is_left_active { "[ACTIVE]" } else { "" }
        );
        render_pane(f, left_area, &app.left_pane, is_left_active, &left_title);

        // Render right pane
        let right_title = format!(
            " {} {} ",
            app.right_pane.display_name(),
            if !is_left_active { "[ACTIVE]" } else { "" }
        );
        render_pane(f, right_area, &app.right_pane, !is_left_active, &right_title);
    }
}

/// Render the bulk rename preview panel
pub fn render_bulk_rename_preview(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let mut lines = Vec::new();

    // Title and pattern input
    lines.push(Line::from(vec![
        Span::styled("Pattern: ", Style::default().fg(Color::Cyan)),
        Span::styled(
            &app.rename_pattern_input,
            Style::default().fg(Color::White),
        ),
        Span::styled("█", Style::default().fg(Color::White)), // cursor
    ]));

    lines.push(Line::from(""));

    // Pattern help
    let help = vec![
        "Pattern types:",
        "  old,new        - Simple replace: 'old' with 'new'",
        "  file_{n},1,3   - Numbered: file_001, file_002...",
        "  s/old/new/     - Regex substitution",
        "  case:upper,all - Case transform (upper/lower/title/toggle)",
        "  ext:add,txt    - Extension action (add/remove/replace)",
    ];
    for line in help {
        lines.push(Line::from(Span::styled(line, Style::default().fg(Color::Gray))));
    }

    lines.push(Line::from(""));

    // Header for the preview table
    lines.push(Line::from(vec![
        Span::styled("  Status  ", Style::default().fg(Color::Cyan)),
        Span::styled("  Old Name", Style::default().fg(Color::Cyan)),
        Span::styled("  ->  ", Style::default().fg(Color::Cyan)),
        Span::styled("  New Name", Style::default().fg(Color::Cyan)),
    ]));

    lines.push(Line::from(Span::styled(
        "─".repeat(60),
        Style::default().fg(Color::DarkGray),
    )));

    // Preview items
    if app.rename_previews.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Enter a pattern to preview changes...",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )));
    } else {
        let available_height = area.height.saturating_sub(lines.len() as u16 + 2) as usize;
        let visible_previews = app.rename_previews.iter()
            .take(available_height)
            .enumerate();

        for (idx, preview) in visible_previews {
            let is_selected = idx == app.rename_selected_index;

            // Status indicator
            let status = if !preview.is_valid {
                "✗ ERROR"
            } else if preview.accepted {
                "✓ APPLY"
            } else {
                "⊘ SKIP "
            };

            let status_style = if !preview.is_valid {
                Style::default().fg(Color::Red)
            } else if preview.accepted {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            };

            // Get file names
            let old_name = preview.old_name();
            let new_name = preview.new_name();

            // Highlight changes in new name
            let new_span = if old_name == new_name {
                Span::styled(&new_name, Style::default().fg(Color::White))
            } else {
                Span::styled(&new_name, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            };

            // Selection indicator
            let prefix = if is_selected { "> " } else { "  " };

            lines.push(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled(status, status_style),
                Span::styled("  ", Style::default()),
                Span::styled(
                    if old_name.len() > 25 {
                        format!("{}...", &old_name[..22])
                    } else {
                        format!("{: <25}", old_name)
                    },
                    Style::default().fg(Color::White),
                ),
                Span::styled("  ->  ", Style::default().fg(Color::Gray)),
                new_span,
            ]));

            // Show error message if invalid
            if !preview.is_valid {
                if let Some(ref error) = preview.error {
                    lines.push(Line::from(vec![
                        Span::styled("         ", Style::default()),
                        Span::styled(error, Style::default().fg(Color::Red)),
                    ]));
                }
            }
        }

        // Show count if more items than visible
        if app.rename_previews.len() > available_height {
            lines.push(Line::from(Span::styled(
                format!("  ... and {} more", app.rename_previews.len() - available_height),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    // Summary
    if !app.rename_previews.is_empty() {
        let accepted_count = app.rename_previews.iter().filter(|p| p.accepted && p.is_valid).count();
        let error_count = app.rename_previews.iter().filter(|p| !p.is_valid).count();

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Summary: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{} total, {} to apply", app.rename_previews.len(), accepted_count),
                Style::default().fg(Color::White),
            ),
            if error_count > 0 {
                Span::styled(
                    format!", {} errors", error_count),
                    Style::default().fg(Color::Red),
                )
            } else {
                Span::raw("")
            },
        ]));
    }

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Bulk Rename Preview "),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Update the footer to show bulk rename mode
pub fn render_footer_bulk_rename(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let mut lines = Vec::new();

    // Mode indicator
    let mode_span = Span::styled(
        "BULK RENAME",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    );

    // File count
    let info_span = Span::styled(
        format!(" | {} file(s)", app.rename_previews.len()),
        Style::default().fg(Color::Gray),
    );

    // Message
    let msg_span = if let Some(ref msg) = app.message {
        let color = match app.message_level {
            MessageLevel::Info => Color::White,
            MessageLevel::Success => Color::Green,
            MessageLevel::Warning => Color::Yellow,
            MessageLevel::Error => Color::Red,
        };
        Span::styled(format!(" | {}", msg), Style::default().fg(color))
    } else {
        Span::raw("")
    };

    lines.push(Line::from(vec![mode_span, info_span, msg_span]));

    // Keybinding hints
    let hints = Span::styled(
        " Enter:Apply | Esc:Cancel | Space:Toggle item | ↑↓:Navigate | Tab:Edit pattern ",
        Style::default().fg(Color::DarkGray),
    );
    lines.push(Line::from(hints));

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}
