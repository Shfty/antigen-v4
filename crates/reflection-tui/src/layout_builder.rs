use tui::layout::Rect;

pub type LayoutIterator = std::vec::IntoIter<Option<Rect>>;

pub trait LayoutStrategy {
    fn allocate(&mut self, desired: Rect);
    fn build(&mut self, source: Rect) -> LayoutIterator;
}

#[derive(Debug, Default)]
pub struct LayoutHorizontal {
    sizes: Vec<(u16, u16)>,
}

impl LayoutStrategy for LayoutHorizontal {
    fn allocate(&mut self, desired: Rect) {
        self.sizes.push((desired.width, desired.height))
    }

    fn build(&mut self, desired: Rect) -> LayoutIterator {
        let mut layout = vec![];

        let mut x = desired.x;
        let y = desired.y;

        for (i, (width, height)) in self.sizes.iter().copied().enumerate() {
            if self.sizes[i..]
                .iter()
                .copied()
                .fold(0, |acc, (next, _)| acc + next)
                > desired.width
            {
                layout.push(None);
            } else {
                layout.push(Some(Rect {
                    x,
                    y,
                    width,
                    height,
                }));

                x += width
            }
        }

        layout.into_iter()
    }
}

#[derive(Debug, Default)]
pub struct LayoutVertical {
    sizes: Vec<(u16, u16)>,
}

impl LayoutStrategy for LayoutVertical {
    fn allocate(&mut self, desired: Rect) {
        self.sizes.push((desired.width, desired.height))
    }

    fn build(&mut self, desired: Rect) -> LayoutIterator {
        let mut layout = vec![];

        let x = desired.x;
        let mut y = desired.y;

        for (i, (width, height)) in self.sizes.iter().copied().enumerate() {
            if self.sizes[i..]
                .iter()
                .copied()
                .fold(0, |acc, (next, _)| acc + next)
                > desired.height
            {
                layout.push(None);
            } else {
                layout.push(Some(Rect {
                    x,
                    y,
                    width,
                    height,
                }));

                y += height
            }
        }

        layout.into_iter()
    }
}

pub struct LayoutBuilder {
    area: Rect,
    strategy: Box<dyn LayoutStrategy>,
}

impl LayoutBuilder {
    pub fn new<S: LayoutStrategy + 'static>(area: Rect, strategy: S) -> Self {
        LayoutBuilder {
            area,
            strategy: Box::new(strategy),
        }
    }

    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn allocate(&mut self, area: Rect) {
        self.strategy.allocate(area)
    }

    pub fn allocate_size(&mut self, width: u16, height: u16) {
        self.allocate(Rect {
            width,
            height,
            ..self.area
        })
    }

    pub fn build(&mut self) -> LayoutIterator {
        self.strategy.build(self.area)
    }
}
