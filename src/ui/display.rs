// Copyright 2022, 2024, 2025 Kunshan Wang
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

use std::borrow::Cow;

use crate::{
    app::{AppState, UiMode},
    backend::DecodeStatus,
    logging::LogRecord,
    player::{ModuleInfo, MomentState},
    util::{center_region, LayoutSplitN},
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub fn render_ui<'a, 'f, 't, B>(frame: &'f mut Frame<'t, B>, area: Rect, app_state: &'a AppState)
where
    B: Backend + 't,
    't: 'f,
{
    let mut ui_renderer = UIRenderer::new(app_state, frame, ColorScheme::default());
    ui_renderer.render_ui(area);
}

struct ColorScheme {
    normal: Style,
    key: Style,
    block_title: Style,
    list_highlight: Style,
    log_error: Style,
    log_warn: Style,
    log_info: Style,
    log_debug: Style,
    log_trace: Style,
    log_target: Style,
    log_message: Style,
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
            log_error: Style::default()
                .fg(Color::Red)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
            log_warn: Style::default()
                .fg(Color::Magenta)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
            log_info: Style::default()
                .fg(Color::Green)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
            log_debug: Style::default()
                .fg(Color::Blue)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
            log_trace: Style::default()
                .fg(Color::Yellow)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
            log_target: Style::default()
                .fg(Color::Gray)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
            log_message: Style::default().fg(Color::White).bg(Color::Black),
        }
    }
}

trait ThemedUIBuilder {
    fn color_scheme(&self) -> &ColorScheme;

    fn new_block<'t, S: Into<Cow<'t, str>>>(&self, title: S) -> Block<'t> {
        Block::default()
            .style(self.color_scheme().normal)
            .borders(Borders::ALL)
            .title(Span::styled(title, self.color_scheme().block_title))
    }

    fn build_state_line<'t, F: FnOnce(&mut LineBuilder<Self>)>(&self, f: F) -> Spans<'t> {
        let mut builder = LineBuilder::new(self);
        f(&mut builder);
        builder.into_spans()
    }

    fn new_span<'t, S: Into<Cow<'t, str>>>(&self, text: S, style: Style) -> Span<'t> {
        Span::styled(text, style)
    }

    fn new_span_normal<'t, S: Into<Cow<'t, str>>>(&self, text: S) -> Span<'t> {
        self.new_span(text, self.color_scheme().normal)
    }

    fn new_span_key<'t, S: Into<Cow<'t, str>>>(&self, text: S) -> Span<'t> {
        self.new_span(text, self.color_scheme().key)
    }

    fn new_span_value<'t, S: Into<Cow<'t, str>>>(&self, text: S) -> Span<'t> {
        self.new_span(text, self.color_scheme().normal)
    }

    fn new_paragraph_from_raw_lines<'t, S: Into<Cow<'t, str>>>(
        &self,
        lines: Vec<S>,
    ) -> Paragraph<'t> {
        let spanses: Vec<Spans> = lines
            .into_iter()
            .map(|line| Spans::from(Span::raw(line)))
            .collect();
        let text = Text::from(spanses);
        Paragraph::new(text).style(self.color_scheme().normal)
    }

    fn style_for_log_level(&self, level: log::Level) -> Style {
        match level {
            log::Level::Error => self.color_scheme().log_error,
            log::Level::Warn => self.color_scheme().log_warn,
            log::Level::Info => self.color_scheme().log_info,
            log::Level::Debug => self.color_scheme().log_debug,
            log::Level::Trace => self.color_scheme().log_trace,
        }
    }
}

struct LineBuilder<'t, 'b, B: ThemedUIBuilder + ?Sized> {
    spans: Vec<Span<'t>>,
    ui_builder: &'b B,
}

impl<'t, 'b, B: ThemedUIBuilder + ?Sized> LineBuilder<'t, 'b, B> {
    pub fn new(ui_builder: &'b B) -> LineBuilder<'t, 'b, B> {
        Self {
            spans: vec![],
            ui_builder,
        }
    }

    pub fn into_spans(self) -> Spans<'t> {
        let spans = self.spans;
        Spans(spans)
    }

    fn key(&mut self, s: impl Into<Cow<'t, str>>) {
        self.spans.push(self.ui_builder.new_span_key(s));
    }

    fn value(&mut self, s: impl Into<Cow<'t, str>>) {
        self.spans.push(self.ui_builder.new_span_value(s));
    }

    fn space(&mut self, s: impl Into<Cow<'t, str>>) {
        self.spans.push(self.ui_builder.new_span_normal(s));
    }

    pub fn kv(&mut self, k: impl Into<Cow<'t, str>>, v: impl Into<Cow<'t, str>>) {
        self.key(k);
        self.space(" ");
        self.value(v);
        self.space("  ");
    }
}

/// Object with the contents for rendering the UI.
///
/// Notes on the lifetimes:
/// -   `'a`: app_state
/// -   `'f`: frame
/// -   `'t`: the underlying terminal of the frame object. `'t` must outlive `'f'`.
struct UIRenderer<'a, 'f, 't, B>
where
    't: 'f,
    B: Backend,
{
    app_state: &'a AppState,
    frame: &'f mut Frame<'t, B>,
    color_scheme: ColorScheme,
}

impl<B: Backend> ThemedUIBuilder for UIRenderer<'_, '_, '_, B> {
    fn color_scheme(&self) -> &ColorScheme {
        &self.color_scheme
    }
}

impl<'a, 'f, 't, B> UIRenderer<'a, 'f, 't, B>
where
    't: 'f,
    B: Backend,
{
    pub fn new(
        app_state: &'a AppState,
        frame: &'f mut Frame<'t, B>,
        color_scheme: ColorScheme,
    ) -> Self {
        Self {
            app_state,
            frame,
            color_scheme,
        }
    }

    const MAX_MOD_SAMPLE_NAME_LEN: usize = 22;

    pub fn render_ui(&mut self, area: Rect) {
        let maybe_message_width = self
            .app_state
            .play_state
            .as_ref()
            .map(|ps| ps.module_info.message_width);

        let message_window_width = maybe_message_width
            .iter()
            .cloned()
            .fold(Self::MAX_MOD_SAMPLE_NAME_LEN, usize::max)
            + 2;

        let [left, message] = Layout::default().direction(Direction::Horizontal).split_n(
            area,
            [
                Constraint::Min(10),
                Constraint::Length(message_window_width as u16),
            ],
        );

        let [state, left_bottom] = Layout::default()
            .direction(Direction::Vertical)
            .split_n(left, [Constraint::Length(7), Constraint::Min(1)]);

        let [playlist_filter, log] = Layout::default().direction(Direction::Horizontal).split_n(
            left_bottom,
            [Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)],
        );

        let maybe_filter_string = {
            let playlist = self.app_state.playlist.lock().unwrap();
            playlist.get_filter_string()
        };

        let (show_filter, edit_filter) = match self.app_state.ui_mode {
            UiMode::Normal => (maybe_filter_string.is_some(), false),
            UiMode::Filter => (true, true),
        };

        let (playlist, maybe_filter) = if show_filter {
            let [filter, playlist] = Layout::default().direction(Direction::Vertical).split_n(
                playlist_filter,
                [Constraint::Length(3), Constraint::Percentage(100)],
            );
            (playlist, Some(filter))
        } else {
            (playlist_filter, None)
        };

        self.render_state(state);
        self.render_playlist(playlist);
        self.render_message(message);
        self.render_log(log);
        if let Some(filter) = maybe_filter {
            self.render_filter(filter, maybe_filter_string, edit_filter);
        }
    }

    fn render_state(&mut self, area: Rect) {
        let block = self.new_block("State");

        let app_state = self.app_state;

        if let Some(ref play_state) = app_state.play_state {
            let ModuleInfo {
                title,
                n_orders,
                n_patterns,
                message: _,
                ..
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

            let DecodeStatus {
                buffer_samples: buffer_size,
                cpu_util,
                ..
            } = app_state.backend.read_decode_status();

            let title_line = self.build_state_line(|b| {
                b.key("Title");
                b.space("   ");
                b.value(title);
            });

            let player_line = self.build_state_line(|b| {
                b.kv("Order", format!("{:02}/{:02}", order, n_orders));
                b.kv("Pattern", format!("{:02}/{:02}", pattern, n_patterns));
                b.kv("Row", format!("{:02}", row));
                b.space(" ");
                b.kv("Repeat", if repeat { "on" } else { "off" });
            });

            let control_line = self.build_state_line(|b| {
                b.kv("Gain", format!("{} dB", gain / 100));
                b.kv("Stereo", format!("{}%", stereo_separation));
                b.kv("Filter", format!("{} taps", filter_taps));
                b.kv("Ramping", format!("{}", volume_ramping));
            });

            let speed_line = self.build_state_line(|b| {
                b.kv("Speed", format!("{}", speed));
                b.kv("Tempo", format!("{}", tempo));
                b.kv("Tempo±", format!("{}/24", tempo_factor));
                b.kv("Pitch±", format!("{}/24", pitch_factor));
            });

            let decoding_line = self.build_state_line(|b| {
                b.kv("Sample Rate", format!("{}", sample_rate));
                b.kv("Buffer Size", format!("{}", buffer_size));
                b.kv("CPU", format!("{:.2}%", cpu_util * 100.0));
            });

            let text = Text {
                lines: vec![
                    title_line,
                    player_line,
                    speed_line,
                    control_line,
                    decoding_line,
                ],
            };

            let paragraph = Paragraph::new(text).block(block);
            self.frame.render_widget(paragraph, area);
        } else {
            let paragraph = Paragraph::new("No module").block(block);
            self.frame.render_widget(paragraph, area);
        };
    }

    fn render_playlist(&mut self, area: Rect) {
        let app_state = self.app_state;
        let color_scheme = &self.color_scheme;

        let window_height = area.height as usize - 2;

        let (shown_titles, list_len, now_playing, offset) = {
            let playlist = app_state.playlist.lock().unwrap();

            let list_len = playlist.len();
            let now_playing = playlist.now_playing_in_view;
            assert!(now_playing.is_none() || list_len > 0);
            let offset = now_playing
                .map(|s| center_region(list_len, window_height, s))
                .unwrap_or(0);
            let limit = (offset + window_height).min(playlist.len());

            let shown_titles = (offset..limit)
                .map(|i| {
                    let item = playlist.get_item(i).unwrap();
                    item.mod_path.display_name()
                })
                .collect::<Vec<_>>();
            (shown_titles, list_len, now_playing, offset)
        };

        let items: Vec<ListItem> = shown_titles
            .iter()
            .cloned()
            .map(|line| {
                let span = Spans::from(line);
                ListItem::new(span).style(color_scheme.normal)
            })
            .collect();

        let now_playing_text = now_playing
            .map(|n| n.to_string())
            .unwrap_or_else(|| "-".to_string());

        let block = self.new_block(format!("Playlist {}/{}", now_playing_text, list_len));

        let items = List::new(items)
            .block(block)
            .style(color_scheme.normal)
            .highlight_style(color_scheme.list_highlight)
            .highlight_symbol(">> ");

        let mut state = ListState::default();
        state.select(now_playing.map(|s| s - offset));

        self.frame.render_stateful_widget(items, area, &mut state);
    }

    fn render_message(&mut self, area: Rect) {
        let app_state = self.app_state;
        let lines: Vec<Cow<str>> = if let Some(ref play_state) = app_state.play_state {
            play_state
                .module_info
                .message
                .iter()
                .map(|s| Cow::<str>::Borrowed(s))
                .collect::<Vec<_>>()
        } else {
            vec![Cow::Borrowed("(No module)")]
        };

        let block = self.new_block("Message");
        let paragraph = self.new_paragraph_from_raw_lines(lines).block(block);
        self.frame.render_widget(paragraph, area);
    }

    fn render_log(&mut self, area: Rect) {
        let width = (area.width - 2) as usize;
        let height = (area.height - 2) as usize;
        let message_width = width - 6;

        let log_records = crate::logging::last_n_records(height);

        let mut last_texts = vec![];
        let mut last_texts_lines = 0;

        for record in log_records.into_iter().rev() {
            let LogRecord {
                level,
                target,
                message,
            } = record;
            let level_string = level.to_string();
            let level_string_len = level_string.len();
            let level_span = self.new_span(level.to_string(), self.style_for_log_level(level));
            let title_space_span = self.new_span_normal(" ".repeat(6 - level_string_len));
            let target_span = self.new_span(target, self.color_scheme().log_target);
            let title_line = Spans(vec![level_span, title_space_span, target_span]);
            let mut lines: Vec<Spans> = vec![title_line];

            let indent_span = self.new_span_normal(" ".repeat(6));

            let message_spans =
                Spans(vec![self.new_span(message, self.color_scheme().log_message)]);
            let mut wrapped = crate::util::force_wrap_spans(&message_spans, message_width);
            wrapped.iter_mut().for_each(|s| {
                s.0.insert(0, indent_span.clone());
            });
            lines.append(&mut wrapped);

            let num_lines = lines.len();
            let text = Text { lines };

            if last_texts.is_empty() || last_texts_lines + num_lines <= height {
                last_texts.push(text);
                last_texts_lines += num_lines;
            } else {
                break;
            }
        }

        let list_ltems = last_texts
            .into_iter()
            .rev()
            .map(ListItem::new)
            .collect::<Vec<_>>();

        let block = self.new_block("Log");
        let list = List::new(list_ltems).block(block);
        self.frame.render_widget(list, area);
    }

    fn render_filter(&mut self, area: Rect, maybe_filter_string: Option<String>, editing: bool) {
        let title = if editing { "Filter (edit)" } else { "Filter" };
        let filter_string = maybe_filter_string.as_deref().unwrap_or("");
        let block = self.new_block(title);
        let paragraph = Paragraph::new(self.new_span_value(filter_string)).block(block);
        self.frame.render_widget(paragraph, area);
    }
}
