use reflection::data::{Data, DataFields};
use tui::{
    layout::Rect,
    style::{Modifier, Style},
};

use crate::{DataWidget, WidgetState};

#[derive(serde::Serialize)]
pub struct PongState {
    p1_pos: (i16, i16),
    p1_size: (u16, u16),

    p2_pos: (i16, i16),
    p2_size: (u16, u16),

    ball_pos: (i16, i16),
    ball_vel: (i16, i16),

    playfield_size: (u16, u16),
}

impl Default for PongState {
    fn default() -> Self {
        PongState {
            p1_pos: (-8, -4),
            p1_size: (1, 2),

            p2_pos: (8, 4),
            p2_size: (1, 2),

            ball_pos: (0, 0),
            ball_vel: (1, 1),

            playfield_size: (12, 6),
        }
    }
}

impl PongState {
    pub fn tick(&mut self) {
        // Integrate ball position
        self.ball_pos.0 += self.ball_vel.0;
        self.ball_pos.1 += self.ball_vel.1;

        // P1 collision
        let p1_min_x = self.p1_pos.0 - self.p1_size.0 as i16;
        let p1_min_y = self.p1_pos.1 - self.p1_size.1 as i16;

        let p1_max_x = self.p1_pos.0 + self.p1_size.0 as i16;
        let p1_max_y = self.p1_pos.1 + self.p1_size.1 as i16;

        if self.ball_pos.0 > p1_min_x
            && self.ball_pos.0 < p1_max_x
            && self.ball_pos.1 > p1_min_y
            && self.ball_pos.1 < p1_max_y
        {
            self.ball_vel.0 *= -1;
        }

        // P2 collision
        let p2_min_x = self.p2_pos.0 - self.p2_size.0 as i16;
        let p2_min_y = self.p2_pos.1 - self.p2_size.1 as i16;

        let p2_max_x = self.p2_pos.0 + self.p2_size.0 as i16;
        let p2_max_y = self.p2_pos.1 + self.p2_size.1 as i16;

        if self.ball_pos.0 > p2_min_x
            && self.ball_pos.0 < p2_max_x
            && self.ball_pos.1 > p2_min_y
            && self.ball_pos.1 < p2_max_y
        {
            self.ball_vel.0 *= -1;
        }

        // Playfield collision
        let playfield_max_x = self.playfield_size.0 as i16;
        let playfield_max_y = self.playfield_size.1 as i16;

        let playfield_min_x = -playfield_max_x;
        let playfield_min_y = -playfield_max_y;

        if self.ball_pos.0 < playfield_min_x {
            self.ball_pos.0 = 0;
            self.ball_vel.0 *= -1;
        } else if self.ball_pos.0 > playfield_max_x {
            self.ball_pos.0 = 0;
            self.ball_vel.0 *= -1;
        }

        if self.ball_pos.1 < playfield_min_y {
            self.ball_pos.1 = playfield_min_y;
            self.ball_vel.1 *= -1;
        } else if self.ball_pos.1 > playfield_max_y {
            self.ball_pos.1 = playfield_max_y;
            self.ball_vel.1 *= -1;
        }
    }
}

pub struct PongWidget<'a> {
    state: &'a mut Data,
}

impl<'a> PongWidget<'a> {
    pub fn new(state: &'a mut Data) -> Self {
        PongWidget { state }
    }
}

impl WidgetState for PongWidget<'_> {}

impl DataWidget for PongWidget<'_> {
    fn size_complex(
        &mut self,
        _area: Rect,
        _predicate: &dyn Fn(&mut Data, std::any::TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) -> (u16, u16) {
        let fields = self
            .state
            .downcast_struct_mut("PongState")
            .expect("State is not a PongState struct");

        let playfield_size = fields
            .get_mut("playfield_size")
            .expect("No playfield_size field")
            .downcast_tuple()
            .expect("playfield_size is not a tuple");

        let width = *playfield_size[0]
            .downcast_u16()
            .expect("playfield_width is not a u16");
        let height = *playfield_size[1]
            .downcast_u16()
            .expect("playfield_height is not a u16");

        (width * 2, height * 2)
    }

    fn render_complex_impl(
        &mut self,
        mut layout: crate::LayoutIterator,
        buf: &mut tui::buffer::Buffer,
        _state: &mut crate::ReflectionWidgetState,
        _predicate: &dyn Fn(&mut Data, std::any::TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) {
        if let Some(area) = layout.next().unwrap() {
            let Rect {
                x: area_x,
                y: area_y,
                ..
            } = area;

            let fields = self.state.downcast_struct("PongState").expect("PongState");

            // Draw walls
            let playfield_size = fields
                .get("playfield_size")
                .expect("No playfield_size field")
                .downcast_tuple()
                .expect("playfield_size");

            let playfield_width =
                *playfield_size[0].downcast_u16().expect("playfield_width") as u16;

            let playfield_height =
                *playfield_size[1].downcast_u16().expect("playfield_height") as u16;

            buf.set_style(
                Rect {
                    x: area_x,
                    y: area_y,
                    width: playfield_width * 2,
                    height: 1,
                },
                Style::default().add_modifier(Modifier::REVERSED),
            );

            buf.set_style(
                Rect {
                    x: area_x,
                    y: area_y + playfield_height * 2,
                    width: playfield_width * 2,
                    height: 1,
                },
                Style::default().add_modifier(Modifier::REVERSED),
            );

            buf.set_style(
                Rect {
                    x: area_x,
                    y: area_y,
                    width: 1,
                    height: playfield_height * 2,
                },
                Style::default().add_modifier(Modifier::REVERSED),
            );

            buf.set_style(
                Rect {
                    x: area_x + playfield_width * 2,
                    y: area_y,
                    width: 1,
                    height: playfield_height * 2,
                },
                Style::default().add_modifier(Modifier::REVERSED),
            );

            // Draw players
            let p1_pos = fields
                .get("p1_pos")
                .expect("No p1_pos field")
                .downcast_tuple()
                .expect("p1_pos");

            let p1_x = *p1_pos[0].downcast_i16().expect("p1_x");
            let p1_y = *p1_pos[1].downcast_i16().expect("p1_y");

            let p1_size = fields
                .get("p1_size")
                .expect("No p1_size field")
                .downcast_tuple()
                .expect("p1_size");

            let p1_width = *p1_size[0].downcast_u16().expect("p1_width");
            let p1_height = *p1_size[1].downcast_u16().expect("p1_height");

            buf.set_style(
                Rect {
                    x: ((area_x as i16) + p1_x + (playfield_width as i16)) as u16 - p1_width,
                    y: ((area_y as i16) + p1_y + (playfield_height as i16)) as u16 - p1_height,
                    width: p1_width * 2,
                    height: p1_height * 2,
                },
                Style::default().add_modifier(Modifier::REVERSED),
            );

            let p2_pos = fields
                .get("p2_pos")
                .expect("No p2_pos field")
                .downcast_tuple()
                .expect("p2_pos");

            let p2_x = *p2_pos[0].downcast_i16().expect("p2_x is not an i16");
            let p2_y = *p2_pos[1].downcast_i16().expect("p2_y is not an i16");

            let p2_size = fields
                .get("p2_size")
                .expect("No p2_size field")
                .downcast_tuple()
                .expect("p2_size");

            let p2_width = *p2_size[0].downcast_u16().expect("p2_width is not a u16");
            let p2_height = *p2_size[1].downcast_u16().expect("p2_height is not a u16");

            buf.set_style(
                Rect {
                    x: ((area_x as i16) + p2_x + (playfield_width as i16)) as u16 - p2_width,
                    y: ((area_y as i16) + p2_y + (playfield_height as i16)) as u16 - p2_height,
                    width: p2_width * 2,
                    height: p2_height * 2,
                },
                Style::default().add_modifier(Modifier::REVERSED),
            );

            // Draw ball
            let ball_pos = fields
                .get("ball_pos")
                .expect("No ball_pos field")
                .downcast_tuple()
                .expect("ball_pos");

            let ball_x = *ball_pos[0].downcast_i16().expect("ball_x");
            let ball_y = *ball_pos[1].downcast_i16().expect("ball_y");

            buf.set_style(
                Rect {
                    x: ((area_x as i16) + ball_x + (playfield_width as i16)) as u16,
                    y: ((area_y as i16) + ball_y + (playfield_height as i16)) as u16,
                    width: 1,
                    height: 1,
                },
                Style::default().add_modifier(Modifier::REVERSED),
            );
        }
    }
}
