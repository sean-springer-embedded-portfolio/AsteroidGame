#![allow(unused)]

use embedded_graphics::pixelcolor::{PixelColor, Rgb565};
use embedded_graphics::prelude::Primitive;
use embedded_graphics::prelude::*;
use embedded_graphics::{
    pixelcolor,
    primitives::{Circle, PrimitiveStyle, PrimitiveStyleBuilder, Styled},
};
use rtt_target::rprintln;

pub struct Missile {
    vx: i32,
    vy: i32,
    loc_x: f32,
    loc_y: f32,
    graphic: Styled<Circle, PrimitiveStyle<Rgb565>>, //top-left corner of bounding square and radius
    center: Point,
    alive: bool,
}

impl Missile {
    const RADIUS: u32 = 5;
    const CENTER_START: i32 = 120;
    const PEN_WIDTH: u32 = 2;
    const MAX_VALUE: i32 = 240;
    const MIN_VALUE: i32 = 0;
    const COLOR: Rgb565 = Rgb565::WHITE;

    pub fn new(vx: i32, vy: i32) -> Self {
        let circle_style = Circle::new(
            Point::new(
                Self::CENTER_START - Self::RADIUS as i32,
                Self::CENTER_START - Self::RADIUS as i32,
            ),
            Self::RADIUS,
        )
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Missile::COLOR)
                .build(),
        );

        Missile {
            vx,
            vy,
            loc_x: Missile::CENTER_START as f32,
            loc_y: Missile::CENTER_START as f32,
            graphic: circle_style,
            center: Point::new(Self::CENTER_START, Self::CENTER_START),
            alive: true,
        }
    }

    pub fn destroy(&mut self) {
        self.alive = false;
    }

    pub fn is_alive(&self) -> bool {
        self.alive
    }

    pub fn get_position(&self) -> Point {
        self.graphic.primitive.center()
    }

    pub fn update_position(&mut self, d_t_ms: u32) {
        let d_t: f32 = d_t_ms as f32 / 1000.0;
        let distance_x = self.vx as f32 * d_t;
        let distance_y = self.vy as f32 * d_t;

        self.loc_x += distance_x;
        self.loc_y += distance_y;

        let cur_pos = self.graphic.primitive.center();

        let translate_x = self.loc_x as i32 - cur_pos.x;
        let translate_y = self.loc_y as i32 - cur_pos.y;

        self.graphic = self.graphic.translate(Point::new(translate_x, translate_y)); // final - initial
    }

    pub fn get_graphic(&self) -> &Styled<Circle, PrimitiveStyle<Rgb565>> {
        &self.graphic
    }
}
