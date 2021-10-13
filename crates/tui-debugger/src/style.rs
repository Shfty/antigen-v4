use std::borrow::Cow;

use tui::{
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Row, Table, Tabs},
};

pub const COLOR_TITLES: Color = Color::White;
pub const COLOR_TRIM: Color = Color::DarkGray;
pub const COLOR_HIGHLIGHT: Color = Color::Magenta;
pub const COLOR_LOWLIGHT: Color = Color::LightBlue;
pub const COLOR_INFO: Color = Color::Cyan;

pub fn style_highlight(focused: bool) -> Style {
    let style = Style::default().fg(COLOR_HIGHLIGHT);
    if focused {
        style.add_modifier(Modifier::REVERSED)
    } else {
        style
    }
}

pub fn tabs<'a>(titles: Vec<Spans<'a>>, focused: bool) -> Tabs<'a> {
    tui::widgets::Tabs::new(titles)
        .style(Style::default().fg(COLOR_LOWLIGHT))
        .divider(DOT)
        .highlight_style(style_highlight(focused))
}

pub fn list<'a, T>(titles: T, focused: bool) -> List<'a>
where
    T: Into<Vec<ListItem<'a>>>,
{
    List::new(titles)
        .style(Style::default().fg(COLOR_LOWLIGHT))
        .highlight_style(super::style::style_highlight(focused))
}

pub fn table<'a, T>(rows: T, focused: bool) -> Table<'a>
where
    T: IntoIterator<Item = Row<'a>>,
{
    Table::new(rows)
        .style(Style::default().fg(COLOR_LOWLIGHT))
        .highlight_style(super::style::style_highlight(focused))
}

pub fn block<'a, T>(title: T) -> Block<'a>
where
    T: Into<Cow<'a, str>>,
{
    Block::default()
        .title(Span::styled(title, Style::default().fg(COLOR_TITLES)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_TRIM))
}
