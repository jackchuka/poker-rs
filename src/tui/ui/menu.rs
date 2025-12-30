use crate::tui::app::AppState;
use ratatui::prelude::*;
use ratatui::widgets::*;

use super::layout::{centered_rect, inner};

pub(super) fn draw_menu(f: &mut Frame, app: &AppState) {
    let size = f.area();
    let area = centered_rect(80, 80, size);
    let block = Block::default().title("poker-rs").borders(Borders::ALL);
    f.render_widget(Clear, area);
    f.render_widget(block.clone(), area);
    let inner_all = inner(area);

    // ASCII logo at the top (render left-aligned to preserve spacing)
    let logo = r#"
 /$$$$$$$                        /$$              
| $$__  $$                      | $$              
| $$  \ $$ /$$   /$$  /$$$$$$$ /$$$$$$   /$$   /$$
| $$$$$$$/| $$  | $$ /$$_____/|_  $$_/  | $$  | $$
| $$__  $$| $$  | $$|  $$$$$$   | $$    | $$  | $$
| $$  \ $$| $$  | $$ \____  $$  | $$ /$$| $$  | $$
| $$  | $$|  $$$$$$/ /$$$$$$$/  |  $$$$/|  $$$$$$$
|__/  |__/ \______/ |_______/    \___/   \____  $$
                                         /$$  | $$
                                        |  $$$$$$/
                                         \______/ 
 /$$$$$$$           /$$                           
| $$__  $$         | $$                           
| $$  \ $$ /$$$$$$ | $$   /$$  /$$$$$$   /$$$$$$  
| $$$$$$$//$$__  $$| $$  /$$/ /$$__  $$ /$$__  $$ 
| $$____/| $$  \ $$| $$$$$$/ | $$$$$$$$| $$  \__/ 
| $$     | $$  | $$| $$_  $$ | $$_____/| $$       
| $$     |  $$$$$$/| $$ \  $$|  $$$$$$$| $$       
|__/      \______/ |__/  \__/ \_______/|__/       
                                                 "#;

    let logo_lines: Vec<Line> = logo
        .lines()
        .map(|l| Line::from(Span::styled(l.to_string(), Style::default().fg(Color::Cyan))))
        .collect();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(logo_lines.len() as u16 + 1), Constraint::Min(3)])
        .split(inner_all);

    let logo_para =
        Paragraph::new(logo_lines).wrap(Wrap { trim: false }).alignment(Alignment::Center);
    f.render_widget(logo_para, rows[0]);

    // Configuration section (centered text)
    let config_items = app.menu_items_display();
    let hints = [String::from("[Enter] Apply  [Q] Quit  [Esc] Cancel  [↑/↓] Move  [+/-] Adjust")];
    let mut cfg_lines: Vec<Line> = Vec::new();
    cfg_lines.push(Line::from(Span::styled(
        "Configuration:",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    for (i, it) in config_items.iter().enumerate() {
        let style = if i == app.menu_index {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        cfg_lines.push(Line::from(Span::styled(it.clone(), style)));
    }
    cfg_lines.push(Line::from(""));
    for hint in hints {
        cfg_lines
            .push(Line::from(Span::styled(hint, Style::default().add_modifier(Modifier::DIM))));
    }
    let cfg_para = Paragraph::new(cfg_lines).wrap(Wrap { trim: true }).alignment(Alignment::Center);
    f.render_widget(cfg_para, rows[1]);
}
