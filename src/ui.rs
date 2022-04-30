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

use crate::app::AppState;

use atomic::Ordering;
use crossterm::{event, execute, terminal};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
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
                    _ => {}
                },
                _ => {}
            }
        }

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
        .constraints([Constraint::Length(10), Constraint::Min(1)].as_ref())
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

    if let Some(mod_info) = &app_state.mod_info.as_ref() {
        let play_state = &app_state.play_state;

        let mut rows = vec![];

        let mut add_row = |k, v| {
            let row = Row::new([Cell::from(Span::raw(k)), Cell::from(Span::raw(v))]);

            rows.push(row);
        };

        let order = play_state.order.load(Ordering::SeqCst);
        let n_orders = mod_info.n_orders;

        let pattern = play_state.pattern.load(Ordering::SeqCst);
        let n_patterns = mod_info.n_patterns;

        let row = play_state.row.load(Ordering::SeqCst);
        let n_rows = play_state.n_rows.load(Ordering::SeqCst);

        let speed = play_state.speed.load(Ordering::SeqCst);
        let tempo = play_state.tempo.load(Ordering::SeqCst);

        add_row("Title", mod_info.title.clone());
        add_row("Order", format!("{}/{}", order, n_orders));
        add_row("Pattern", format!("{}/{}", pattern, n_patterns));
        add_row("Row", format!("{}/{}", row, n_rows));
        add_row("Speed", format!("{}", speed));
        add_row("Tempo", format!("{}", tempo));

        let table = Table::new(rows)
            .widths(&[Constraint::Length(10), Constraint::Percentage(100)])
            .block(block);

        f.render_widget(table, area);
    } else {
        let paragraph = Paragraph::new("No module").block(block);
        f.render_widget(paragraph, area);
    };
}

fn render_playlist(f: &mut Frame<impl Backend>, area: Rect, app_state: &AppState) {
    let text = app_state
        .playlist
        .iter()
        .map(|item| Spans::from(item.mod_path.root_path.to_string_lossy().to_string()))
        .collect::<Vec<_>>();

    let block = Block::default().title("Playlist").borders(Borders::ALL);
    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn render_message(f: &mut Frame<impl Backend>, area: Rect, app_state: &AppState) {
    let text = if let Some(mod_info) = app_state.mod_info.as_ref() {
        mod_info
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
