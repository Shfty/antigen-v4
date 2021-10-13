use tui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    text::Spans,
    widgets::{Block, Widget},
};

#[derive(Debug, Clone)]
pub struct TabContainer<'a, 'b, TF, NF>
where
    TF: FnOnce(Vec<Spans<'a>>, bool) -> tui::widgets::Tabs<'a>,
    NF: FnOnce(Rect, &mut Buffer, usize),
{
    titles: Vec<Spans<'a>>,
    block: Option<Block<'b>>,
    highlighted: bool,
    selected: usize,
    tabs: TF,
    next: Option<NF>,
}

impl<'a, 'b, TF, NF> TabContainer<'a, 'b, TF, NF>
where
    TF: FnOnce(Vec<Spans<'a>>, bool) -> tui::widgets::Tabs<'a>,
    NF: FnOnce(Rect, &mut Buffer, usize),
{
    pub fn new(tabs: TF) -> Self {
        TabContainer {
            titles: Default::default(),
            block: None,
            highlighted: false,
            selected: 0,
            tabs,
            next: None,
        }
    }
    pub fn titles(self, titles: Vec<Spans<'a>>) -> Self {
        Self { titles, ..self }
    }

    pub fn block(self, block: Block<'b>) -> Self {
        Self {
            block: Some(block),
            ..self
        }
    }

    pub fn highlight(self, highlighted: bool) -> Self {
        Self {
            highlighted,
            ..self
        }
    }

    pub fn select(self, selected: usize) -> Self {
        Self { selected, ..self }
    }

    pub fn next(self, next: NF) -> Self {
        Self {
            next: Some(next),
            ..self
        }
    }
}

impl<'a, 'b, TF, NF> Widget for TabContainer<'a, 'b, TF, NF>
where
    TF: FnOnce(Vec<Spans<'a>>, bool) -> tui::widgets::Tabs<'a>,
    NF: FnOnce(Rect, &mut Buffer, usize),
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = tui::layout::Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let tabs = (self.tabs)(self.titles, self.highlighted);
        if let Some(block) = self.block {
            tabs.block(block)
        } else {
            tabs
        }
        .select(self.selected)
        .render(chunks[0], buf);

        if let Some(next) = self.next {
            next(chunks[1], buf, self.selected)
        }
    }
}
