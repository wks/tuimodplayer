use ratatui::style::{Color, Modifier, Style};

pub struct ColorScheme {
    pub normal: Style,
    pub key: Style,
    pub block_title: Style,
    pub list_highlight: Style,
    pub log_error: Style,
    pub log_warn: Style,
    pub log_info: Style,
    pub log_debug: Style,
    pub log_trace: Style,
    pub log_target: Style,
    pub log_message: Style,
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
