use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub tab_bar_height: u16,
    pub header_height: u16,
    pub footer_height: u16,
    pub preview_width_percent: u16,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        LayoutConfig {
            tab_bar_height: 3,
            header_height: 3,
            footer_height: 3,
            preview_width_percent: 40,
        }
    }
}

/// Calculate layout for normal view (single pane or with preview)
pub fn calculate_layout(area: Rect, show_preview: bool, preview_width: u16) -> LayoutParts {
    let tab_bar_height = LayoutConfig::default().tab_bar_height;
    let header_height = LayoutConfig::default().header_height;
    let footer_height = LayoutConfig::default().footer_height;

    // Vertical split: tab bar | header | main | footer
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(tab_bar_height),
            Constraint::Length(header_height),
            Constraint::Min(0),
            Constraint::Length(footer_height),
        ])
        .split(area);

    let tab_bar_area = vertical_chunks[0];
    let header_area = vertical_chunks[1];
    let main_area = vertical_chunks[2];
    let footer_area = vertical_chunks[3];

    // Split main area into file list and (optionally) preview
    let (file_list_area, preview_area) = if show_preview && main_area.width > 40 {
        let preview_width = (main_area.width as u16 * preview_width / 100).min(main_area.width - 20);
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(main_area.width - preview_width),
                Constraint::Length(preview_width),
            ])
            .split(main_area);
        (horizontal_chunks[0], Some(horizontal_chunks[1]))
    } else {
        (main_area, None)
    };

    LayoutParts {
        tab_bar_area,
        header_area,
        file_list_area,
        preview_area,
        footer_area,
        left_pane_area: None,
        right_pane_area: None,
    }
}

/// Calculate layout for split view (dual-pane side by side)
pub fn calculate_split_layout(area: Rect, active_pane: crate::state::ActivePane) -> LayoutParts {
    let tab_bar_height = LayoutConfig::default().tab_bar_height;
    let header_height = LayoutConfig::default().header_height;
    let footer_height = LayoutConfig::default().footer_height;

    // Vertical split: tab bar | header | main | footer
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(tab_bar_height),
            Constraint::Length(header_height),
            Constraint::Min(0),
            Constraint::Length(footer_height),
        ])
        .split(area);

    let tab_bar_area = vertical_chunks[0];
    let header_area = vertical_chunks[1];
    let main_area = vertical_chunks[2];
    let footer_area = vertical_chunks[3];

    // Split main area into two panes
    let pane_width = main_area.width / 2;
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(pane_width),
            Constraint::Min(0),
        ])
        .split(main_area);

    let (left_pane_area, right_pane_area) = match active_pane {
        crate::state::ActivePane::Left => (horizontal_chunks[0], horizontal_chunks[1]),
        crate::state::ActivePane::Right => (horizontal_chunks[1], horizontal_chunks[0]),
    };

    LayoutParts {
        tab_bar_area,
        header_area,
        file_list_area: left_pane_area, // For backwards compatibility
        preview_area: None,
        footer_area,
        left_pane_area: Some(left_pane_area),
        right_pane_area: Some(right_pane_area),
    }
}

pub struct LayoutParts {
    pub tab_bar_area: Rect,
    pub header_area: Rect,
    pub file_list_area: Rect,
    pub preview_area: Option<Rect>,
    pub footer_area: Rect,
    /// Left pane area (only when split_view is enabled)
    pub left_pane_area: Option<Rect>,
    /// Right pane area (only when split_view is enabled)
    pub right_pane_area: Option<Rect>,
}
