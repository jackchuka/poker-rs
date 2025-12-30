use ratatui::layout::Constraint;
use ratatui::prelude::{Layout, Rect};

pub(super) fn inner(area: Rect) -> Rect {
    Rect { x: area.x + 1, y: area.y + 1, width: area.width - 2, height: area.height - 2 }
}

pub(super) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(ratatui::prelude::Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    let area = Layout::default()
        .direction(ratatui::prelude::Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);
    area[1]
}
