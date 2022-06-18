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

use std::{borrow::Cow, io::stdout, panic::PanicInfo, time::Duration};

use crate::{
    app::AppState,
    player::{ModuleInfo, MomentState},
    util::LayoutSplitN,
};

use crossterm::{
    event::{self, KeyModifiers},
    execute, terminal,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use anyhow::Result;

static mut OLD_HOOK: Option<Box<dyn Fn(&PanicInfo) + Sync + Send>> = None;
static REGISTER_PANIC_HOOK: std::sync::Once = std::sync::Once::new();

fn ui_panic_hook(panic_info: &PanicInfo<'_>) {
    execute!(stdout(), terminal::LeaveAlternateScreen).unwrap_or_else(|e| {
        // Cannot handle error while handling panic.  Printing is the best effort.
        eprintln!("Failed to leave alternative screen: {}", e);
    });
    crate::logging::set_stderr_enabled(true);
    terminal::disable_raw_mode().unwrap_or_else(|e| {
        // Cannot handle error while handling panic.  Printing is the best effort.
        eprintln!("Failed to disable raw mode: {}", e);
    });
    let old_hook = unsafe { OLD_HOOK.as_ref().unwrap() };
    old_hook(panic_info);
}

pub fn run_ui(app_state: &mut AppState) -> Result<()> {
    REGISTER_PANIC_HOOK.call_once(|| {
        unsafe {
            OLD_HOOK = Some(std::panic::take_hook());
        }
        std::panic::set_hook(Box::new(ui_panic_hook));
    });

    terminal::enable_raw_mode()?;

    crate::logging::set_stderr_enabled(false);
    execute!(stdout(), terminal::EnterAlternateScreen)?;

    let backend = tui::backend::CrosstermBackend::new(stdout());
    let mut term = tui::Terminal::new(backend)?;

    loop {
        let mut redraw = false;

        if event::poll(Duration::from_millis(100))? {
            let ev = event::read()?;
            use event::{Event, KeyCode, KeyEvent};
            #[allow(clippy::single_match)] // Will add more event handling in the future
            #[allow(clippy::collapsible_match)]
            match ev {
                Event::Key(KeyEvent { code, modifiers }) => match code {
                    KeyCode::Char('l') if modifiers.contains(KeyModifiers::CONTROL) => {
                        redraw = true;
                    }
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('m') => {
                        app_state.next();
                    }
                    KeyCode::Char('n') => {
                        app_state.prev();
                    }
                    KeyCode::Char('M') => {
                        app_state.next10();
                    }
                    KeyCode::Char('N') => {
                        app_state.prev10();
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
                    KeyCode::Char('3') => {
                        app_state.gain_down();
                    }
                    KeyCode::Char('4') => {
                        app_state.gain_up();
                    }
                    KeyCode::Char('5') => {
                        app_state.stereo_separation_down();
                    }
                    KeyCode::Char('6') => {
                        app_state.stereo_separation_up();
                    }
                    KeyCode::Char('7') => {
                        app_state.filter_taps_down();
                    }
                    KeyCode::Char('8') => {
                        app_state.filter_taps_up();
                    }
                    KeyCode::Char('9') => {
                        app_state.volume_ramping_down();
                    }
                    KeyCode::Char('0') => {
                        app_state.volume_ramping_up();
                    }
                    KeyCode::Char('r') => {
                        app_state.toggle_repeat();
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

        if std::mem::take(&mut redraw) {
            term.clear()?;
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

struct ColorScheme {
    normal: Style,
    key: Style,
    block_title: Style,
    list_highlight: Style,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            normal: Style::default().fg(Color::White).bg(Color::Black),
            key: Style::default()
                .fg(Color::White)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
            block_title: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            list_highlight: Style::default()
                .fg(Color::Black)
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        }
    }
}

struct LineBuilder<'a> {
    spans: Vec<Span<'a>>,
    color_scheme: &'a ColorScheme,
}

impl<'a> LineBuilder<'a> {
    pub fn new(color_scheme: &'a ColorScheme) -> LineBuilder<'a> {
        Self {
            spans: vec![],
            color_scheme,
        }
    }

    pub fn into_spans(self) -> Spans<'a> {
        Spans(self.spans)
    }

    pub fn build<F: FnOnce(&mut Self)>(color_scheme: &'a ColorScheme, f: F) -> Spans {
        let mut builder = Self::new(color_scheme);
        f(&mut builder);
        builder.into_spans()
    }

    pub fn span(&mut self, s: impl Into<Cow<'a, str>>, style: Style) {
        self.spans.push(Span::styled(s, style));
    }

    pub fn key(&mut self, s: impl Into<Cow<'a, str>>) {
        self.span(s, self.color_scheme.key);
    }

    pub fn value(&mut self, s: impl Into<Cow<'a, str>>) {
        self.span(s, self.color_scheme.normal);
    }

    pub fn space(&mut self, s: impl Into<Cow<'a, str>>) {
        self.span(s, self.color_scheme.normal);
    }

    pub fn kv(&mut self, k: impl Into<Cow<'a, str>>, v: impl Into<Cow<'a, str>>) {
        self.key(k);
        self.space(" ");
        self.value(v);
        self.space("  ");
    }
}

fn render_ui(f: &mut Frame<impl Backend>, area: Rect, app_state: &AppState) {
    let [left, message] = Layout::default()
        .direction(Direction::Horizontal)
        .split_n(area, [Constraint::Min(10), Constraint::Length(24)]);

    let [state, left_bottom] = Layout::default()
        .direction(Direction::Vertical)
        .split_n(left, [Constraint::Length(7), Constraint::Min(1)]);

    let [playlist, log] = Layout::default().direction(Direction::Horizontal).split_n(
        left_bottom,
        [Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)],
    );

    let color_scheme = ColorScheme::default();

    render_state(f, state, app_state, &color_scheme);
    render_playlist(f, playlist, app_state, &color_scheme);
    render_message(f, message, app_state, &color_scheme);
    render_log(f, log, app_state, &color_scheme);
}

fn render_state(
    f: &mut Frame<impl Backend>,
    area: Rect,
    app_state: &AppState,
    color_scheme: &ColorScheme,
) {
    let block = Block::default().title("State").borders(Borders::ALL);

    if let Some(ref play_state) = app_state.play_state {
        let ModuleInfo {
            title,
            n_orders,
            n_patterns,
            message: _,
        } = play_state.module_info.clone();

        let MomentState {
            order,
            pattern,
            row,
            speed,
            tempo,
        } = play_state.moment_state.read();

        let sample_rate = app_state.options.sample_rate;

        let tempo_factor = app_state.control.tempo.value();
        let pitch_factor = app_state.control.pitch.value();
        let gain = app_state.control.gain.output();
        let stereo_separation = app_state.control.stereo_separation.output();
        let filter_taps = app_state.control.filter_taps.output();
        let volume_ramping = app_state.control.volume_ramping.output();
        let repeat = app_state.control.repeat;

        let title_line = LineBuilder::build(color_scheme, |b| {
            b.key("Title");
            b.space("   ");
            b.value(title);
        });

        let player_line = LineBuilder::build(color_scheme, |b| {
            b.kv("Order", format!("{:02}/{:02}", order, n_orders));
            b.kv("Pattern", format!("{:02}/{:02}", pattern, n_patterns));
            b.kv("Row", format!("{:02}", row));
            b.space(" ");
            b.kv("Repeat", if repeat { "on" } else { "off" });
        });

        let control_line = LineBuilder::build(color_scheme, |b| {
            b.kv("Gain", format!("{} dB", gain / 100));
            b.kv("Stereo", format!("{}%", stereo_separation));
            b.kv("Filter", format!("{} taps", filter_taps));
            b.kv("Ramping", format!("{}", volume_ramping));
        });

        let speed_line = LineBuilder::build(color_scheme, |b| {
            b.kv("Speed", format!("{}", speed));
            b.kv("Tempo", format!("{}", tempo));
            b.kv("Tempo±", format!("{}/24", tempo_factor));
            b.kv("Pitch±", format!("{}/24", pitch_factor));
        });

        let sample_rate_line = LineBuilder::build(color_scheme, |b| {
            b.kv("Sample rate", format!("{}", sample_rate));
        });

        let text = Text {
            lines: vec![
                title_line,
                player_line,
                speed_line,
                control_line,
                sample_rate_line,
            ],
        };

        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("No module").block(block);
        f.render_widget(paragraph, area);
    };
}

fn render_playlist(
    f: &mut Frame<impl Backend>,
    area: Rect,
    app_state: &AppState,
    color_scheme: &ColorScheme,
) {
    let (titles, now_playing) = {
        let playlist = app_state.playlist.lock().unwrap();
        let titles = playlist
            .items
            .iter()
            .map(|item| item.mod_path.display_name())
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

    let now_playing_text = now_playing
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string());
    let n_items = items.len();

    let block = Block::default()
        .title(format!("Playlist {}/{}", now_playing_text, n_items))
        .borders(Borders::ALL);

    let items = List::new(items)
        .block(block)
        .style(color_scheme.normal)
        .highlight_style(color_scheme.list_highlight)
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    state.select(now_playing);

    f.render_stateful_widget(items, area, &mut state);
}

fn render_message(
    f: &mut Frame<impl Backend>,
    area: Rect,
    app_state: &AppState,
    _color_scheme: &ColorScheme,
) {
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

fn render_log(
    f: &mut Frame<impl Backend>,
    area: Rect,
    _app_state: &AppState,
    _color_scheme: &ColorScheme,
) {
    let text = crate::logging::last_n_records(area.height as usize)
        .iter()
        .map(|line| Spans::from(line.clone()))
        .collect::<Vec<_>>();

    let block = Block::default().title("Log").borders(Borders::ALL);
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(block);
    f.render_widget(paragraph, area);
}
