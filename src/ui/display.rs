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

use std::borrow::Cow;

use crate::{
    app::AppState,
    backend::DecodeStatus,
    player::{ModuleInfo, MomentState},
    util::LayoutSplitN,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
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

trait ThemedUIBuilder {
    fn color_scheme(&self) -> &ColorScheme;

    fn new_block<'t, S: Into<Cow<'t, str>>>(&self, title: S) -> Block<'t> {
        Block::default()
            .style(self.color_scheme().normal)
            .borders(Borders::ALL)
            .title(Span::styled(title, self.color_scheme().block_title))
    }

    fn build_state_line<'b, 't, F: FnOnce(&mut LineBuilder<Self>)>(&'b self, f: F) -> Spans<'t> {
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

    pub fn render_ui(&mut self, area: Rect) {
        let [left, message] = Layout::default()
            .direction(Direction::Horizontal)
            .split_n(area, [Constraint::Min(10), Constraint::Length(27)]);

        let [state, left_bottom] = Layout::default()
            .direction(Direction::Vertical)
            .split_n(left, [Constraint::Length(7), Constraint::Min(1)]);

        let [playlist, log] = Layout::default().direction(Direction::Horizontal).split_n(
            left_bottom,
            [Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)],
        );

        self.render_state(state);
        self.render_playlist(playlist);
        self.render_message(message);
        self.render_log(log);
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
                buffer_size,
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
                ListItem::new(span).style(color_scheme.normal)
            })
            .collect();

        let now_playing_text = now_playing
            .map(|n| n.to_string())
            .unwrap_or_else(|| "-".to_string());
        let n_items = items.len();

        let block = self.new_block(format!("Playlist {}/{}", now_playing_text, n_items));

        let items = List::new(items)
            .block(block)
            .style(color_scheme.normal)
            .highlight_style(color_scheme.list_highlight)
            .highlight_symbol(">> ");

        let mut state = ListState::default();
        state.select(now_playing);

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
        let text = crate::logging::last_n_records(area.height as usize);

        let block = self.new_block("Log");
        let paragraph = self
            .new_paragraph_from_raw_lines(text)
            .wrap(Wrap { trim: true })
            .block(block);
        self.frame.render_widget(paragraph, area);
    }
}
