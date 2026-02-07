use ratatui::Frame;

use crate::state::{App, Mode};
use crate::ui::layout::{calculate_layout, calculate_split_layout};
use crate::ui::widgets::{render_bulk_rename_preview, render_confirm_dialog, render_content_search_results, render_file_list, render_footer, render_footer_bulk_rename, render_header, render_preview, render_tab_bar, render_split_panes};

pub fn render_app(f: &mut Frame, app: &App) {
    // Handle BulkRename mode specially
    if app.mode == Mode::BulkRename {
        render_bulk_rename_mode(f, app);
        return;
    }

    // Choose layout based on split_view mode
    let layout = if app.split_view {
        calculate_split_layout(f.area(), app.active_pane)
    } else {
        calculate_layout(
            f.area(),
            app.config.show_preview,
            app.config.preview_width_percent,
        )
    };

    // Render tab bar
    render_tab_bar(f, layout.tab_bar_area, app);

    // Render header
    render_header(f, layout.header_area, app);

    // Render content based on view mode
    if app.split_view {
        // Render split panes
        render_split_panes(f, &layout, app);
    } else {
        // Render single file list
        render_file_list(f, layout.file_list_area, app);

        // Render preview if enabled
        if let Some(preview_area) = layout.preview_area {
            render_preview(f, preview_area, app);
        }
    }

    // Render footer
    render_footer(f, layout.footer_area, app);

    // Render confirmation dialog if active
    if app.confirm_dialog.is_some() {
        render_confirm_dialog(f, f.area(), app);
    }
}

/// Render bulk rename mode overlay
fn render_bulk_rename_mode(f: &mut Frame, app: &App) {
    // Create a centered popup area for the bulk rename interface
    let popup_width = 80.min(f.area().width.saturating_sub(4));
    let popup_height = 20.min(f.area().height.saturating_sub(4));

    let popup_area = ratatui::layout::Rect {
        x: (f.area().width - popup_width) / 2,
        y: (f.area().height - popup_height) / 2,
        width: popup_width,
        height: popup_height,
    };

    // Clear the background area with a dim effect
    let clear_area = ratatui::layout::Rect {
        x: popup_area.x.saturating_sub(1),
        y: popup_area.y.saturating_sub(1),
        width: popup_width + 2,
        height: popup_height + 2,
    };

    // Render a background block for dimming effect
    let background = ratatui::widgets::Block::default()
        .style(ratatui::style::Style::default().bg(ratatui::style::Color::DarkGray));
    f.render_widget(background, clear_area);

    // Render the bulk rename preview in the popup
    render_bulk_rename_preview(f, popup_area, app);

    // Render custom footer for bulk rename mode
    let footer_area = ratatui::layout::Rect {
        x: f.area().x,
        y: f.area().y + f.area().height.saturating_sub(3),
        width: f.area().width,
        height: 3,
    };
    render_footer_bulk_rename(f, footer_area, app);
}
