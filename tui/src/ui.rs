use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::state::{AppState, Role};

pub fn draw(frame: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(15),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(4),
        ])
        .split(frame.area());

    let mut header_lines = vec![Line::from(vec![
        Span::styled(
            "OnlyNak3d TUI ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw("| Tab switch role | q quit | r refresh"),
    ])];
    header_lines.extend(role_avatar_lines(app.role));

    let title = Paragraph::new(header_lines)
    .block(Block::default().borders(Borders::ALL).title("Header"));
    frame.render_widget(title, chunks[0]);

    let status = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("Model balance: {}", app.model_balance),
            Style::default().fg(Color::Green),
        ),
        Span::raw("  "),
        Span::styled(
            format!("Fan balance: {}", app.fan_balance),
            Style::default().fg(Color::Magenta),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Balances"));
    frame.render_widget(status, chunks[1]);

    match app.role {
        Role::Model => draw_model(frame, app, chunks[2]),
        Role::Fan => draw_fan(frame, app, chunks[2]),
    }

    let footer = Paragraph::new(format!(
        "{}\nKey file: {}",
        app.status, app.key_path
    ))
    .block(Block::default().borders(Borders::ALL).title("Status"))
    .wrap(Wrap { trim: true });
    frame.render_widget(footer, chunks[3]);
}

fn role_avatar_lines(role: Role) -> Vec<Line<'static>> {
    match role {
        Role::Model => vec![
            Line::from(Span::styled(
                "                 .-''''''-.                ",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "               .'  _.._  '.              ",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "              /  .'_  _'.  \\             ",
                Style::default().fg(Color::Magenta),
            )),
            Line::from(Span::styled(
                "             /  / ( \\/ ) \\  \\            ",
                Style::default().fg(Color::Magenta),
            )),
            Line::from(Span::styled(
                "            |  |   /\\    |  |           ",
                Style::default().fg(Color::Magenta),
            )),
            Line::from(Span::styled(
                "            |  |  \\__/   |  |           ",
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "            |  | .--.   |  |   *       ",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "            |  |(____)  |  |           ",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "             \\  \\______/  /            ",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "              '.______.''             ",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "             _/  /||\\  \\_             ",
                Style::default().fg(Color::Magenta),
            )),
            Line::from(Span::styled(
                "           .'/__/ || \\__\\'.            ",
                Style::default().fg(Color::Magenta),
            )),
            Line::from(Span::styled(
                "          /_/___/ || \\___\\_\\           ",
                Style::default().fg(Color::Magenta),
            )),
            Line::from(Span::styled(
                "                __ || __                ",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "               /___||___\\               ",
                Style::default().fg(Color::Yellow),
            )),
        ],
        Role::Fan => vec![
            Line::from(Span::styled(
                "                 .-======-.               ",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                "               .'  .--.  '.             ",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                "              /  /-__-\\  \\             ",
                Style::default().fg(Color::Blue),
            )),
            Line::from(Span::styled(
                "             |  |[o][o]|  |            ",
                Style::default().fg(Color::Blue),
            )),
            Line::from(Span::styled(
                "             |  | -- |  |            ",
                Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "             |  |/__\\|  |            ",
                Style::default().fg(Color::Blue),
            )),
            Line::from(Span::styled(
                "             |  |\\__/|  |            ",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                "             |  '----'  |            ",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                "              \\  .__.  /             ",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                "               '.____.'              ",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                "             ___/|  |\\___             ",
                Style::default().fg(Color::Blue),
            )),
            Line::from(Span::styled(
                "            /__ /|  |\\ __\\            ",
                Style::default().fg(Color::Blue),
            )),
            Line::from(Span::styled(
                "           _/___/ |  | \\___\\_          ",
                Style::default().fg(Color::Blue),
            )),
            Line::from(Span::styled(
                "                _/____\\_               ",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                "               /__====__\\              ",
                Style::default().fg(Color::Cyan),
            )),
        ],
    }
}

fn draw_model(frame: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
    let prompt_h = if app.create_profile_prompt.is_some() || app.add_prompt.is_some() {
        4
    } else {
        3
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(prompt_h), Constraint::Min(8)])
        .split(area);

    let controls_text = if let Some(prompt) = &app.create_profile_prompt {
        format!("{}\n> {}", prompt.title(), prompt.input)
    } else if let Some(prompt) = &app.add_prompt {
        format!(
            "{}\n> {}",
            prompt.title(),
            prompt.input
        )
    } else {
        format!(
            "c create profile (prompt) | a add paid content (prompt)\nprofile created: {}",
            if app.model_profile_created { "yes" } else { "no" }
        )
    };
    let controls = Paragraph::new(controls_text)
        .block(Block::default().borders(Borders::ALL).title("Model Controls"));
    frame.render_widget(controls, chunks[0]);

    let tables = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[1]);

    let paid_rows = app.model_paid.iter().map(|p| {
        Row::new(vec![
            Cell::from(p.content_id.to_string()),
            Cell::from(p.preview.clone()),
            Cell::from(p.price.to_string()),
        ])
    });
    let paid_table = Table::new(
        paid_rows,
        [
            Constraint::Length(12),
            Constraint::Percentage(60),
            Constraint::Length(16),
        ],
    )
    .header(
        Row::new(vec!["Content ID", "Preview", "Price"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).title("Added Paid Content"));
    frame.render_widget(paid_table, tables[0]);

    let history_rows = app.history.iter().map(|h| {
        Row::new(vec![
            Cell::from(h.buyer.to_string()),
            Cell::from(h.content_id.to_string()),
            Cell::from(h.price.to_string()),
        ])
    });

    let history_table = Table::new(
        history_rows,
        [
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(16),
        ],
    )
    .header(
        Row::new(vec!["Buyer", "Content ID", "Price"])
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).title("Fan Purchases History"));

    frame.render_widget(history_table, tables[1]);
}

fn draw_fan(frame: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let mut items: Vec<ListItem> = Vec::new();
    for (idx, row) in app.fan_hidden.iter().enumerate() {
        let marker = if idx == app.selected_hidden { ">" } else { " " };
        items.push(ListItem::new(format!(
            "{marker} [{}] {} | price {}",
            row.content_id, row.preview, row.price
        )));
    }
    if items.is_empty() {
        items.push(ListItem::new("No hidden content available"));
    }

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Fan Content (j/k or arrows, b buy)"),
    );
    frame.render_widget(list, chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(8)])
        .split(chunks[1]);

    let profiles_count = app.profiles.profiles.len();
    let summary = Paragraph::new(format!(
        "Models loaded: {profiles_count}\nHidden rows: {}",
        app.fan_hidden.len()
    ))
    .block(Block::default().borders(Borders::ALL).title("Summary"));
    frame.render_widget(summary, right_chunks[0]);

    let decrypted = Paragraph::new(app.decrypted.as_str())
        .block(Block::default().borders(Borders::ALL).title("Decrypted Last Purchase"))
        .wrap(Wrap { trim: true });
    frame.render_widget(decrypted, right_chunks[1]);
}
