use crate::cards::Card;
use crate::game::{PlayerStatus, Street};
use crate::tui::app::AppState;
use ratatui::prelude::*;
use ratatui::widgets::*;

use super::layout::{centered_rect, inner};

pub(super) fn draw_table(f: &mut Frame, app: &AppState) {
    let size = f.area();
    let header_lines_count: u16 = 2;
    // Add borders (2 rows) to get total block height
    let header_height = header_lines_count + 2;
    let status_lines: u16 = 2;
    let status_height: u16 = status_lines + 2; // content + borders

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height), // header
            Constraint::Length(5),             // board
            Constraint::Min(3),                // seats
            Constraint::Length(status_height), // status bar
        ])
        .split(size);

    // Header (multi-line for readability)
    let mut header_lines: Vec<Line> = Vec::new();
    header_lines.push(Line::from(format!(
        "SB: {}  BB: {}  BTN P{}  {}",
        app.game.small_blind(),
        app.game.big_blind(),
        app.game.dealer() + 1,
        pot_line(&app.game).unwrap_or_default(),
    )));
    header_lines.push(Line::from(format!(
        "Bet: {}   MinRaise: {}   ToCall: {}",
        app.game.current_bet(),
        app.game.min_raise(),
        app.game.to_call(app.focus)
    )));
    let header = Paragraph::new(header_lines)
        .block(Block::default().title("poker-rs").borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Board (5 slots)
    let board_block =
        Block::default().title(format!("Board — {:?}", app.game.street())).borders(Borders::ALL);
    let board_area = chunks[1];
    let board_inner = inner(board_area);
    let board_cards = app.game.board().as_slice();
    let card_width = board_inner.width.saturating_sub(2) / 5;
    let board_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(card_width),
            Constraint::Length(card_width),
            Constraint::Length(card_width),
            Constraint::Length(card_width),
            Constraint::Length(card_width),
        ])
        .split(board_inner);
    f.render_widget(board_block, board_area);
    for i in 0..5 {
        let highlight = (matches!(app.game.street(), Street::Flop) && i < 3)
            || (matches!(app.game.street(), Street::Turn) && i == 3)
            || (matches!(app.game.street(), Street::River) && i == 4);
        render_card_widget(
            f,
            board_chunks[i],
            board_cards.get(i).copied(),
            if highlight { Some(Color::Yellow) } else { None },
        );
    }

    // Seats ring layout approximation (top row and bottom row mimic circle)
    let seats_area = chunks[2];
    let rows = 2u16;
    let total = app.game.players().len();
    let top_cols: u16 = ((total + 1) / 2) as u16; // ceil
    let bottom_cols: u16 = (total as u16).saturating_sub(top_cols); // floor
    let row_height = seats_area.height.saturating_sub(2) / rows;
    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints((0..rows).map(|_| Constraint::Length(row_height)).collect::<Vec<_>>())
        .split(inner(seats_area));
    let sb_pos = app.game.sb_pos();
    let bb_pos = app.game.bb_pos();
    for r in 0..rows as usize {
        let cols_this: u16 = if r == 0 { top_cols } else { bottom_cols };
        if cols_this == 0 {
            continue;
        }
        let col_width = seats_area.width.saturating_sub(2) / cols_this.max(1);
        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints((0..cols_this).map(|_| Constraint::Length(col_width)).collect::<Vec<_>>())
            .split(row_chunks[r]);
        for c in 0..cols_this as usize {
            // Map index to approximate ring:
            // Top row left-to-right: players 0..top_cols-1; bottom row right-to-left: remaining
            let idx = if r == 0 { c } else { total.saturating_sub(1) - c };
            if let Some(p) = app.game.players().get(idx) {
                let seat_area = col_chunks[c];
                render_player_card(f, seat_area, app, idx, p, sb_pos, bb_pos);
            }
        }
    }

    // Status bar: split horizontally for info vs keys, render two lines of content
    let status_area = chunks[3];
    f.render_widget(Block::default().borders(Borders::ALL).title("Status"), status_area);
    let status_inner = inner(status_area);
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(status_inner);

    let mut left_info = if !app.hand_started {
        vec![
            Line::from("Hand not started — press Space to deal."),
            Line::from("Actions disabled until deal."),
        ]
    } else if matches!(app.game.street(), Street::Showdown) {
        vec![
            Line::from("Hand over — press Space for new hand."),
            Line::from("Actions disabled at showdown."),
        ]
    } else {
        vec![Line::from(format!("Acting: P{}   Focus: P{}", app.game.current() + 1, app.focus + 1))]
    };

    if let Some(err) = app.action_error() {
        left_info.push(Line::from(Span::styled(
            format!("Error: {err}"),
            Style::default().fg(Color::Red),
        )));
    }

    let (can_act, to_call, stack, current_bet) = if app.game.players().is_empty() {
        (false, 0, 0, app.game.current_bet())
    } else {
        let idx = app.focus.min(app.game.players().len().saturating_sub(1));
        let p = &app.game.players()[idx];
        let can_act = app.hand_started
            && app.focus == app.game.current()
            && !matches!(app.game.street(), Street::Showdown)
            && matches!(p.status(), PlayerStatus::Active);
        (can_act, app.game.to_call(app.game.current()), p.stack(), app.game.current_bet())
    };
    let fold_enabled = can_act && to_call > 0;
    let call_enabled = can_act && stack > 0;
    let bet_enabled = can_act && current_bet == 0 && stack > 0;
    let raise_enabled = can_act && current_bet > 0 && stack > to_call;
    let action_style = |enabled: bool| {
        if enabled {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::DIM)
        }
    };
    if app.hand_started && !matches!(app.game.street(), Street::Showdown) {
        let action_line = Line::from(vec![
            Span::raw("Actions: "),
            Span::styled("F fold", action_style(fold_enabled)),
            Span::raw(" • "),
            Span::styled("C call/check", action_style(call_enabled)),
            Span::raw(" • "),
            Span::styled("B bet", action_style(bet_enabled)),
            Span::raw(" • "),
            Span::styled("R raise", action_style(raise_enabled)),
        ]);
        left_info.push(action_line);
    }

    let right_keys = vec![Line::from(""), Line::from("? help • H history • M menu")];
    let left_para = Paragraph::new(left_info).wrap(Wrap { trim: true });
    let right_para =
        Paragraph::new(right_keys).wrap(Wrap { trim: true }).alignment(Alignment::Right);
    f.render_widget(left_para, cols[0]);
    f.render_widget(right_para, cols[1]);

    if app.help_open() {
        draw_help(f);
    } else if app.history_open() {
        draw_history(f, app);
    } else if app.amount_entry_active() {
        draw_amount_entry(f, app);
    }
}

fn draw_history(f: &mut Frame, app: &AppState) {
    let area = centered_rect(70, 80, f.area());
    let block = Block::default().title("History").borders(Borders::ALL);
    let mut lines: Vec<Line> = Vec::new();
    let entries = app.game.history_recent_offset(AppState::HISTORY_PAGE_SIZE, app.history_offset());
    if entries.is_empty() {
        lines.push(Line::from("No history yet."));
    } else {
        for entry in entries {
            let amount = entry.amount.map(|v| format!(" {v}")).unwrap_or_default();
            let line = format!(
                "P{} {}{} [{:?}]",
                entry.seat + 1,
                entry.verb.label(),
                amount,
                entry.street
            );
            lines.push(Line::from(line));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Up/Down scroll • Close: H or Esc",
        Style::default().add_modifier(Modifier::DIM),
    )));
    let para = Paragraph::new(lines).wrap(Wrap { trim: true });
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(para, inner(area));
}

fn render_player_card(
    f: &mut Frame,
    seat_area: Rect,
    app: &AppState,
    idx: usize,
    p: &crate::game::Player,
    sb_pos: Option<usize>,
    bb_pos: Option<usize>,
) {
    let mut title = format!("P{}", idx + 1);
    if idx == app.focus {
        title.push_str(" [Focus]");
    }
    if idx == app.game.dealer() {
        title.push_str(" [BTN]");
    }
    if sb_pos == Some(idx) {
        title.push_str(" [SB]");
    }
    if bb_pos == Some(idx) {
        title.push_str(" [BB]");
    }
    if let Some(label) = app.bot_profile_label(idx) {
        title.push_str(&format!(" [BOT:{label}]"));
    }
    if matches!(p.status(), PlayerStatus::AllIn) {
        title.push_str(" [ALL-IN]");
    }
    if idx == app.game.current() {
        title.push_str(" [Act]");
    }
    let mut block = Block::default().title(title).borders(Borders::ALL);
    let status = match p.status() {
        PlayerStatus::Active => "Active",
        PlayerStatus::Folded => "Folded",
        PlayerStatus::AllIn => "All-in",
    };
    let dim = Style::default().add_modifier(Modifier::DIM);
    let make_line = |label: &str, value: Option<String>| -> Line {
        if let Some(v) = value {
            Line::from(format!("{label}{v}"))
        } else {
            Line::from(vec![Span::raw(label.to_string()), Span::styled("--", dim)])
        }
    };
    let blind_value = if sb_pos == Some(idx) {
        Some(format!("SB {}", app.game.small_blind()))
    } else if bb_pos == Some(idx) {
        Some(format!("BB {}", app.game.big_blind()))
    } else {
        None
    };
    let last_value = p.last_action().map(|s| s.to_string());
    let category_value = if matches!(app.game.street(), Street::Showdown) {
        app.game.showdown_categories().get(idx).and_then(|c| *c).map(|c| format!("{c:?}"))
    } else {
        None
    };
    let mut lines: Vec<Line> = Vec::with_capacity(6);
    lines.push(Line::from(format!("Stack: ${}", p.stack())));
    lines.push(Line::from(format!("Bet: {}", p.bet())));
    lines.push(Line::from(format!("Status: {status}")));
    lines.push(make_line("Last: ", last_value));
    lines.push(make_line("Blind: ", blind_value));
    lines.push(make_line("Category: ", category_value));
    let show_hole_cards = matches!(app.game.street(), Street::Showdown) || idx == app.focus;
    if matches!(p.status(), PlayerStatus::Folded) {
        block = block.border_style(Style::default().fg(Color::DarkGray));
    } else if matches!(app.game.street(), Street::Showdown) && app.game.winners().contains(&idx) {
        block = block.border_style(Style::default().fg(Color::Green));
    } else if matches!(p.status(), PlayerStatus::AllIn) {
        block = block.border_style(Style::default().fg(Color::LightRed));
    } else if idx == app.game.current() && idx == app.focus {
        block = block.border_style(Style::default().fg(Color::Magenta));
    } else if idx == app.game.current() {
        block = block.border_style(Style::default().fg(Color::Yellow));
    } else if idx == app.focus {
        block = block.border_style(Style::default().fg(Color::Cyan));
    }
    f.render_widget(block, seat_area);
    let seat_inner = inner(seat_area);
    let mut text_area = seat_inner;
    let mut cards_area: Option<Rect> = None;
    if show_hole_cards && p.hole().is_some() && seat_inner.height > 3 {
        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(seat_inner);
        text_area = split[0];
        cards_area = Some(split[1]);
    }
    let para = Paragraph::new(lines).wrap(Wrap { trim: true });
    f.render_widget(para, text_area);
    if let (Some(h), Some(area)) = (p.hole(), cards_area) {
        let cw = area.width.saturating_sub(2) / 2;
        let card_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(cw), Constraint::Length(cw)])
            .split(area);
        render_card_widget(f, card_chunks[0], Some(h.first()), Some(Color::Cyan));
        render_card_widget(f, card_chunks[1], Some(h.second()), Some(Color::Cyan));
    }
}

fn draw_help(f: &mut Frame) {
    let area = centered_rect(70, 80, f.area());
    let block = Block::default().title("Help").borders(Borders::ALL);
    let lines = vec![
        Line::from(Span::styled("Table:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("- Space: deal / new hand"),
        Line::from("- A: amount entry"),
        Line::from("- F: fold"),
        Line::from("- C: call / check"),
        Line::from("- B: bet min"),
        Line::from("- R: raise min"),
        Line::from("- D: cycle bot difficulty (focus)"),
        Line::from("- ] / [: focus next / prev"),
        Line::from("- 1-9: focus seat"),
        Line::from("- H: history"),
        Line::from(""),
        Line::from(Span::styled("Amount Entry:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("- 0-9: edit amount"),
        Line::from("- Backspace: delete digit"),
        Line::from("- + / -: adjust by BB"),
        Line::from("- Enter: submit"),
        Line::from("- Esc: cancel"),
        Line::from(""),
        Line::from(Span::styled("Menu:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("- M: open / close menu"),
        Line::from("- Up / Down: move selection"),
        Line::from("- + / -: adjust value"),
        Line::from("- Enter: apply"),
        Line::from("- Esc: cancel"),
        Line::from("- Q: quit (menu)"),
        Line::from(""),
        Line::from("Close help: ? or Esc"),
    ];
    let para = Paragraph::new(lines).wrap(Wrap { trim: true });
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(para, inner(area));
}

fn draw_amount_entry(f: &mut Frame, app: &AppState) {
    let area = centered_rect(50, 30, f.area());
    let title = if app.game.current_bet() == 0 { "Bet Amount" } else { "Raise Amount" };
    let min = if app.game.current_bet() == 0 {
        app.game.big_blind().max(1)
    } else {
        app.game.current_bet() + app.game.min_raise()
    };
    let current = app.amount_entry_text().unwrap_or("");
    let lines = vec![
        Line::from(format!("Current: {current}")),
        Line::from(format!("Min: {min}")),
        Line::from("Digits to edit, Backspace to delete"),
        Line::from("+/- in BB steps, Enter submit, Esc cancel"),
    ];
    let block = Block::default().title(title).borders(Borders::ALL);
    let inner_area = inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner_area);
    let para = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(para, chunks[0]);
    let error = app.amount_entry_error().unwrap_or("");
    let error_line = Line::from(Span::styled(error, Style::default().fg(Color::Red)));
    let error_para = Paragraph::new(error_line).alignment(Alignment::Center);
    f.render_widget(error_para, chunks[1]);
}

fn pot_line(game: &crate::game::Game) -> Option<String> {
    let breakdown = game.pot_breakdown();
    let mut parts: Vec<String> = Vec::with_capacity(1 + breakdown.sides.len());
    parts.push(breakdown.main.to_string());
    for side in breakdown.sides {
        parts.push(side.to_string());
    }
    Some(format!("Pots: ${} = {}", game.pot(), parts.join(" + ")))
}

fn suit_glyph_and_style(s: crate::cards::Suit) -> (char, Style) {
    use crate::cards::Suit::*;
    match s {
        Hearts => ('♥', Style::default().fg(Color::Red)),
        Diamonds => ('♦', Style::default().fg(Color::Red)),
        Spades => ('♠', Style::default().fg(Color::White)),
        Clubs => ('♣', Style::default().fg(Color::White)),
    }
}

fn rank_char(r: crate::cards::Rank) -> &'static str {
    use crate::cards::Rank::*;
    match r {
        Two => "2",
        Three => "3",
        Four => "4",
        Five => "5",
        Six => "6",
        Seven => "7",
        Eight => "8",
        Nine => "9",
        Ten => "10",
        Jack => "J",
        Queen => "Q",
        King => "K",
        Ace => "A",
    }
}

fn render_card_widget(
    f: &mut Frame,
    area: Rect,
    card: Option<crate::cards::Card>,
    border: Option<Color>,
) {
    let mut block = Block::default().borders(Borders::ALL).title_alignment(Alignment::Center);
    if let Some(color) = border {
        block = block.border_style(Style::default().fg(color));
    }
    let inner = inner(area);
    f.render_widget(block, area);
    let content = if let Some(c) = card {
        let (sg, style) = suit_glyph_and_style(c.suit());
        let text = format!("{}{}", rank_char(c.rank()), sg);
        Line::from(Span::styled(text, style))
    } else {
        Line::from("[  ]")
    };
    let para = Paragraph::new(content).alignment(Alignment::Center);
    f.render_widget(para, inner);
}

#[allow(dead_code)]
fn short_card(c: Card) -> String {
    let (sg, _) = suit_glyph_and_style(c.suit());
    format!("{}{}", rank_char(c.rank()), sg)
}
