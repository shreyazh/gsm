use crate::app::{App, ConfirmAction, Mode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

const BRAND: Color = Color::Rgb(255, 135, 0);
const ADDED: Color = Color::Green;
const REMOVED: Color = Color::Red;
const DIM: Color = Color::DarkGray;
const HIGHLIGHT_BG: Color = Color::Rgb(45, 45, 60);

pub fn render(f: &mut Frame, app: &App) {
    match &app.mode {
        Mode::Diff => render_diff_view(f, app, false),
        Mode::Files => render_diff_view(f, app, true),
        Mode::Confirm(action) => {
            render_main(f, app);
            render_confirm_popup(f, action);
        }
        Mode::NewStash => {
            render_main(f, app);
            render_new_stash_popup(f, app);
        }
        Mode::Message(msg) => {
            render_main(f, app);
            render_message_popup(f, msg);
        }
        Mode::Normal => render_main(f, app),
    }
}

fn render_main(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    render_header(f, chunks[0], app);
    render_stash_list(f, chunks[1], app);
    render_footer(f, chunks[2], app);
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let search_indicator = if app.searching {
        format!("  🔍 /{}", app.search_query)
    } else if !app.search_query.is_empty() {
        format!("  filter: /{}", app.search_query)
    } else {
        String::new()
    };

    let title = Line::from(vec![
        Span::styled(
            " gsm ",
            Style::default()
                .fg(Color::Black)
                .bg(BRAND)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "  branch: {}  stashes: {}{}",
                app.current_branch,
                app.stashes.len(),
                search_indicator
            ),
            Style::default().fg(Color::Gray),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BRAND))
        .title(title);

    f.render_widget(block, area);
}

fn render_stash_list(f: &mut Frame, area: Rect, app: &App) {
    let filtered = app.filtered_stashes();

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, stash)| {
            let is_selected = i == app.selected;
            let index_style = Style::default().fg(BRAND);
            let branch_style = Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::ITALIC);
            let date_style = Style::default().fg(DIM);
            let msg_style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            let line = Line::from(vec![
                Span::styled(format!("{:<3}", stash.index), index_style),
                Span::raw(" "),
                Span::styled(
                    format!("{:<20}", truncate(&stash.branch, 20)),
                    branch_style,
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{:<35}", truncate(&stash.short_msg, 35)),
                    msg_style,
                ),
                Span::raw(" "),
                Span::styled(stash.date.clone(), date_style),
            ]);

            ListItem::new(line)
        })
        .collect();

    if items.is_empty() {
        let empty_msg = if app.stashes.is_empty() {
            "No stashes found. Press 'n' to create one."
        } else {
            "No stashes match your search."
        };
        let p = Paragraph::new(empty_msg)
            .style(Style::default().fg(DIM))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(DIM))
                    .title(" Stashes "),
            );
        f.render_widget(p, area);
        return;
    }

    let mut state = ListState::default();
    state.select(Some(app.selected));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(80, 80, 100)))
                .title(Line::from(vec![
                    Span::raw(" Stashes "),
                    Span::styled(
                        format!("({}/{})", app.selected + 1, filtered.len()),
                        Style::default().fg(DIM),
                    ),
                ])),
        )
        .highlight_style(Style::default().bg(HIGHLIGHT_BG))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut state);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let keys: Vec<Vec<Span>> = if app.searching {
        vec![
            key_span("Enter", "confirm"),
            key_span("Esc", "cancel search"),
        ]
    } else {
        vec![
            key_span("↑↓/jk", "navigate"),
            key_span("Enter/d", "diff"),
            key_span("f", "files"),
            key_span("a", "apply"),
            key_span("p", "pop"),
            key_span("x", "drop"),
            key_span("n", "new"),
            key_span("/", "search"),
            key_span("q", "quit"),
        ]
    };

    let mut spans: Vec<Span> = Vec::new();
    for (i, s) in keys.into_iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.extend(s);
    }

    let line = Line::from(spans);
    let p = Paragraph::new(line)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(60, 60, 80))),
        )
        .alignment(Alignment::Center);

    f.render_widget(p, area);
}

fn key_span(key: &str, desc: &str) -> Vec<Span<'static>> {
    vec![
        Span::styled(
            format!("[{key}]"),
            Style::default().fg(BRAND).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {desc}"), Style::default().fg(Color::Gray)),
    ]
}

fn render_diff_view(f: &mut Frame, app: &App, is_files: bool) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    let stash_info = app
        .selected_stash()
        .map(|s| format!("{} — {}", s.name, s.short_msg))
        .unwrap_or_default();

    let title = Line::from(vec![
        Span::styled(
            if is_files { " Files " } else { " Diff " },
            Style::default()
                .fg(Color::Black)
                .bg(BRAND)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {stash_info}"),
            Style::default().fg(Color::Gray),
        ),
    ]);

    let header = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BRAND))
        .title(title);

    f.render_widget(header, chunks[0]);

    let visible_height = chunks[1].height.saturating_sub(2) as usize;
    let lines: Vec<Line> = app
        .diff_content
        .iter()
        .skip(app.diff_scroll)
        .take(visible_height)
        .map(|line| colorize_diff_line(line))
        .collect();

    let diff = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(80, 80, 100))),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(diff, chunks[1]);

    let scroll_info = format!(
        "line {}/{}",
        app.diff_scroll + 1,
        app.diff_content.len().max(1)
    );

    let mut footer_spans: Vec<Span> = Vec::new();
    footer_spans.extend(key_span("↑↓/jk", "scroll"));
    footer_spans.push(Span::raw("   "));
    footer_spans.extend(key_span("PgUp/PgDn", "fast scroll"));
    footer_spans.push(Span::raw("   "));
    footer_spans.extend(key_span("Esc/q", "back"));
    footer_spans.push(Span::raw(format!("   {scroll_info}")));

    let footer = Paragraph::new(Line::from(footer_spans))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(60, 60, 80))),
        )
        .alignment(Alignment::Center);

    f.render_widget(footer, chunks[2]);
}

fn colorize_diff_line(line: &str) -> Line<'static> {
    let (style, content) = if line.starts_with('+') && !line.starts_with("+++") {
        (Style::default().fg(ADDED), line.to_string())
    } else if line.starts_with('-') && !line.starts_with("---") {
        (Style::default().fg(REMOVED), line.to_string())
    } else if line.starts_with("@@") {
        (Style::default().fg(Color::Cyan), line.to_string())
    } else if line.starts_with("diff ")
        || line.starts_with("index ")
        || line.starts_with("---")
        || line.starts_with("+++")
    {
        (
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            line.to_string(),
        )
    } else {
        (Style::default().fg(Color::Gray), line.to_string())
    };

    Line::from(Span::styled(content, style))
}

fn render_confirm_popup(f: &mut Frame, action: &ConfirmAction) {
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);

    let (title, body, color) = match action {
        ConfirmAction::Apply => (
            "Apply Stash",
            "Apply this stash? (it stays in the stash list)",
            Color::Green,
        ),
        ConfirmAction::Pop => (
            "Pop Stash",
            "Apply and remove this stash from the list?",
            Color::Yellow,
        ),
        ConfirmAction::Drop => (
            "Drop Stash",
            "Permanently delete this stash? This cannot be undone.",
            Color::Red,
        ),
    };

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(body, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "[y] Yes",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled("[n] No", Style::default().fg(Color::Red)),
        ]),
    ];

    let popup = Paragraph::new(content)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(format!(" {title} "))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color)),
        );

    f.render_widget(popup, area);
}

fn render_new_stash_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 25, f.area());
    f.render_widget(Clear, area);

    let untracked_label = if app.new_stash_untracked {
        Span::styled(
            "[Tab] Include untracked: ON ",
            Style::default().fg(Color::Green),
        )
    } else {
        Span::styled(
            "[Tab] Include untracked: off",
            Style::default().fg(DIM),
        )
    };

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Stash message:",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            format!("{}_", app.new_stash_input),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(untracked_label),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(BRAND)),
            Span::raw(" save   "),
            Span::styled("[Esc]", Style::default().fg(Color::Red)),
            Span::raw(" cancel"),
        ]),
    ];

    let popup = Paragraph::new(content)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(" New Stash ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BRAND)),
        );

    f.render_widget(popup, area);
}

fn render_message_popup(f: &mut Frame, msg: &str) {
    let area = centered_rect(55, 18, f.area());
    f.render_widget(Clear, area);

    let is_error = msg.starts_with("Error");
    let color = if is_error { Color::Red } else { Color::Green };

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            msg.to_string(),
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to continue",
            Style::default().fg(DIM),
        )),
    ];

    let popup = Paragraph::new(content)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(if is_error { " Error " } else { " Done " })
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color)),
        );

    f.render_widget(popup, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}