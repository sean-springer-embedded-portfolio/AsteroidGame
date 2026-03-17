use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Line, PrimitiveStyle, Styled};

use libm::{cosf, roundf, sinf}; // for f32 (single precision)

//use rtt_target::rprintln;

pub struct Slider {
    graphic: Styled<Line, PrimitiveStyle<Rgb565>>,
    cur_angle: f32, //radians
}

impl Slider {
    const HYPOTENUSE: u32 = 120;
    const HYPOTENUSE_F: f32 = Slider::HYPOTENUSE as f32;
    const SCALED_HYPOT_FACTOR: f32 = 12.0; //scale 120 / 12 = 10 = half width
    const P1_START_ANGLE: f32 = core::f32::consts::FRAC_PI_2; //1.5708; //90 deg
    const P2_START_ANGLE: f32 = core::f32::consts::FRAC_PI_2 * 3.1; //4.71239; //270 deg
    const SLIDER_WIDTH: i32 = 20;
    const SLIDER_HALF_WIDTH: i32 = Slider::SLIDER_WIDTH / 2;
    const STEP_SIZE: f32 = 0.0174533 * 4.0; //4 deg * pi / 180 = 90 total steps around the circle
    const COLOR: Rgb565 = Rgb565::RED;
    const CENTER: i32 = 120;

    pub fn new() -> Self {
        Slider {
            graphic: Line::new(
                Point::new(0, -Slider::SLIDER_HALF_WIDTH),
                Point::new(0, Slider::SLIDER_HALF_WIDTH),
            ) //10,110 10,130
            .into_styled(PrimitiveStyle::with_stroke(
                Slider::COLOR,
                Slider::SLIDER_WIDTH as u32,
            )),
            cur_angle: 0.0,
        }
    }

    pub fn update(&mut self, angle: i32) {
        self.cur_angle = Slider::STEP_SIZE * angle as f32;

        // slider rotation of endpoints and scaling
        let new_p1_angle = Slider::P1_START_ANGLE + self.cur_angle;
        let new_p2_angle = Slider::P2_START_ANGLE + self.cur_angle;

        let new_p1 = Point::new(
            roundf(Slider::HYPOTENUSE_F * cosf(new_p1_angle) / Slider::SCALED_HYPOT_FACTOR) as i32,
            roundf(Slider::HYPOTENUSE_F * sinf(new_p1_angle) / Slider::SCALED_HYPOT_FACTOR) as i32,
        );

        let new_p2 = Point::new(
            roundf(Slider::HYPOTENUSE_F * cosf(new_p2_angle) / Slider::SCALED_HYPOT_FACTOR) as i32,
            roundf(Slider::HYPOTENUSE_F * sinf(new_p2_angle) / Slider::SCALED_HYPOT_FACTOR) as i32,
        );

        // center point translation
        let new_x = roundf(Slider::HYPOTENUSE_F * cosf(self.cur_angle)) as i32;
        let new_y = roundf(Slider::HYPOTENUSE_F * sinf(self.cur_angle)) as i32;

        self.graphic = Line::new(new_p1, new_p2)
            .into_styled(PrimitiveStyle::with_stroke(
                Rgb565::RED,
                Slider::SLIDER_WIDTH as u32,
            ))
            .translate(Point::new(Slider::CENTER + new_x, Slider::CENTER + new_y)); //move to center then add new coordiantes
    }

    pub fn get_graphic(&self) -> &Styled<Line, PrimitiveStyle<Rgb565>> {
        &self.graphic
    }

    pub fn check_for_collision(&self, pos: &Point) -> bool {
        let slider_pos = self.graphic.primitive.midpoint();
        let mut min_x = slider_pos.x - Slider::SLIDER_HALF_WIDTH;
        let mut max_x = slider_pos.x + Slider::SLIDER_HALF_WIDTH;
        let mut min_y = slider_pos.y - Slider::SLIDER_HALF_WIDTH;
        let mut max_y = slider_pos.y + Slider::SLIDER_HALF_WIDTH;

        if min_x > max_x {
            core::mem::swap(&mut min_x, &mut max_x);
        }
        if min_y > max_y {
            core::mem::swap(&mut min_y, &mut max_y);
        }

        let mut collided = false;
        if pos.x >= min_x && pos.x <= max_x && pos.y >= min_y && pos.y <= max_y {
            collided = true;
        }

        collided
    }
}
