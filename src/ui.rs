// Copyright 2022 Kunshan Wang
//
// This file is part of TUIModPlayer.  TUIModPlayer is free software: you can redistribute it
// and/or modify it under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// TUIModPlayer is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See
// the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with TUIModPlayer. If
// not, see <https://www.gnu.org/licenses/>.

use std::{io::stdout, time::Duration};

use crate::{
    app::AppState,
    player::{ModuleInfo, MomentStateCopy},
};

use crossterm::{event, execute, terminal};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table},
};

use anyhow::Result;

pub fn run_ui(app_state: &mut AppState) -> Result<()> {
    terminal::enable_raw_mode()?;

    crate::logging::set_stderr_enabled(false);
    execute!(stdout(), terminal::EnterAlternateScreen)?;

    let backend = tui::backend::CrosstermBackend::new(stdout());
    let mut term = tui::Terminal::new(backend)?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            let ev = event::read()?;
            use event::{Event, KeyCode, KeyEvent};
            #[allow(clippy::single_match)] // Will add more event handling in the future
            #[allow(clippy::collapsible_match)]
            match ev {
                Event::Key(KeyEvent { code, modifiers: _ }) => match code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('m') => {
                        app_state.next();
                    }
                    KeyCode::Char('n') => {
                        app_state.prev();
                    }
                    KeyCode::Char('u') => {
                        app_state.tempo_down();
                    }
                    KeyCode::Char('i') => {
                        app_state.tempo_up();
                    }
                    KeyCode::Char('o') => {
                        app_state.pitch_down();
                    }
                    KeyCode::Char('p') => {
                        app_state.pitch_up();
                    }
                    KeyCode::Char(' ') => {
                        app_state.pause_resume();
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        app_state.handle_backend_events();

        term.draw(|f| {
            render_ui(f, f.size(), app_state);
        })?;
    }

    execute!(stdout(), terminal::LeaveAlternateScreen)?;
    crate::logging::set_stderr_enabled(true);

    terminal::disable_raw_mode()?;

    Ok(())
}

fn render_ui(f: &mut Frame<impl Backend>, area: Rect, app_state: &AppState) {
    let split1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
        .split(area);

    let split2 = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(15), Constraint::Min(1)].as_ref())
        .split(split1[0]);

    let split3 = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(10)].as_ref())
        .split(split1[1]);

    render_state(f, split2[0], app_state);
    render_playlist(f, split2[1], app_state);
    render_message(f, split3[0], app_state);
    render_log(f, split3[1], app_state);
}

fn render_state(f: &mut Frame<impl Backend>, area: Rect, app_state: &AppState) {
    let block = Block::default().title("State").borders(Borders::ALL);

    if let Some(ref play_state) = app_state.play_state {
        let ModuleInfo {
            title,
            n_orders,
            n_patterns,
            message: _,
        } = play_state.module_info.clone();

        let MomentStateCopy {
            order,
            pattern,
            row,
            speed,
            tempo,
        } = play_state.moment_state.load_atomic();

        let sample_rate = app_state.options.sample_rate;

        let tempo_factor = app_state.control.tempo.output();
        let pitch_factor = app_state.control.pitch.output();

        let mut max_key_len = 0;
        let mut rows = vec![];

        let mut add_row = |k: &str, v: String| {
            max_key_len = Ord::max(max_key_len, k.len());
            let row = Row::new([
                Cell::from(Span::raw(k.to_string())),
                Cell::from(Span::raw(v)),
            ]);
            rows.push(row);
        };

        add_row("Title", title);
        add_row("Order", format!("{}/{}", order, n_orders));
        add_row("Pattern", format!("{}/{}", pattern, n_patterns));
        //        add_row("Row", format!("{}/{}", row, n_rows));
        add_row("Row", format!("{}", row));
        add_row("Speed", format!("{}", speed));
        add_row("Tempo", format!("{}", tempo));
        add_row("Sample rate", format!("{}", sample_rate));
        add_row("Tempo factor", format!("{}", tempo_factor));
        add_row("Pitch factor", format!("{}", pitch_factor));

        let table_layout = [
            Constraint::Length(max_key_len as u16),
            Constraint::Percentage(100),
        ];
        let table = Table::new(rows).widths(&table_layout).block(block);

        f.render_widget(table, area);
    } else {
        let paragraph = Paragraph::new("No module").block(block);
        f.render_widget(paragraph, area);
    };
}

fn render_playlist(f: &mut Frame<impl Backend>, area: Rect, app_state: &AppState) {
    let (titles, now_playing) = {
        let playlist = app_state.playlist.lock().unwrap();
        let titles = playlist
            .items
            .iter()
            .map(|item| item.mod_path.root_path.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        let now_playing = playlist.now_playing;
        (titles, now_playing)
    };

    let items: Vec<ListItem> = titles
        .iter()
        .cloned()
        .map(|line| {
            let span = Spans::from(line);
            ListItem::new(span).style(Style::default().fg(Color::White).bg(Color::Black))
        })
        .collect();

    let block = Block::default().title("Playlist").borders(Borders::ALL);

    let items = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    state.select(now_playing);

    f.render_stateful_widget(items, area, &mut state);
}

fn render_message(f: &mut Frame<impl Backend>, area: Rect, app_state: &AppState) {
    let text = if let Some(ref play_state) = app_state.play_state {
        play_state
            .module_info
            .message
            .iter()
            .map(|line| Spans::from(line.clone()))
            .collect::<Vec<_>>()
    } else {
        vec![Spans::from("(No module)")]
    };

    let block = Block::default().title("Message").borders(Borders::ALL);
    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn render_log(f: &mut Frame<impl Backend>, area: Rect, _app_state: &AppState) {
    let text = crate::logging::last_n_records(10)
        .iter()
        .map(|line| Spans::from(line.clone()))
        .collect::<Vec<_>>();

    let block = Block::default().title("Log").borders(Borders::ALL);
    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}
