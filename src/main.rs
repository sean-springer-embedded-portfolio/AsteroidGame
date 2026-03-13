#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_graphics::{
    Drawable,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Triangle},
};
use embedded_hal::delay::DelayNs;
use embedded_hal_bus::spi::ExclusiveDevice;
use microbit::hal::{
    Spim,
    gpio::{
        Level, Output, PushPull,
        p0::{P0_03, P0_04, P0_13, P0_17},
        p1::P1_02,
    },
    spim::{self, Frequency},
    timer::Timer,
};
use mipidsi::{
    Builder,
    models::GC9A01,
    options::{ColorInversion, Orientation, Rotation},
};
use panic_rtt_target as _;
use rtt_target::rprintln;
use rtt_target::rtt_init_print;

use libm::{cosf, roundf, sinf}; // for f32 (single precision)

type SclPin = P0_17<Output<PushPull>>; //e13
type SdaPin = P0_13<Output<PushPull>>; //e15
type DcPin = P0_03<Output<PushPull>>; //e1 - data / command - tell if command is brightnes of color
type CsPin = P1_02<Output<PushPull>>; //e16 - chip selet, wakes up the display
type RstPin = P0_04<Output<PushPull>>; // e2

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = microbit::Board::take().unwrap();

    let mut timer0 = Timer::new(board.TIMER0);

    // Setup SPI
    let sck = board.pins.p0_17.into_push_pull_output(Level::Low).degrade();
    let coti = board.pins.p0_13.into_push_pull_output(Level::Low).degrade();

    let dc = board.edge.e01.into_push_pull_output(Level::Low);
    let cs = board.edge.e16.into_push_pull_output(Level::Low);
    let rst = board.edge.e02.into_push_pull_output(Level::High);

    let spi_bus = Spim::new(
        board.SPIM3,
        microbit::hal::spim::Pins {
            sck: Some(sck),
            mosi: Some(coti),
            miso: None,
        },
        Frequency::M32,
        spim::MODE_0,
        0xFF, // ORC overflow character
    );
    let spi = display_interface_spi::SPIInterface::new(
        ExclusiveDevice::new_no_delay(spi_bus, cs).unwrap(),
        dc,
    );

    // Setup GC9A01 display using mipidsi
    let mut display = Builder::new(GC9A01, spi)
        .orientation(Orientation::new().rotate(Rotation::Deg180))
        .invert_colors(ColorInversion::Inverted)
        .reset_pin(rst)
        .init(&mut timer0)
        .unwrap();

    // Call `embedded_graphics` `clear()` trait method
    <_ as embedded_graphics::draw_target::DrawTarget>::clear(&mut display, Rgb565::WHITE).unwrap();

    let triangle = |color| {
        // make upward-pointing triangle
        let triangle_style = PrimitiveStyleBuilder::new().fill_color(color).build();
        Triangle::new(
            Point { x: 120, y: 70 },  // top vertex (apex)
            Point { x: 70, y: 170 },  // bottom-left vertex
            Point { x: 170, y: 170 }, // bottom-right vertex
        )
        .into_styled(triangle_style)
    };

    let mut triangles = [triangle(Rgb565::BLUE), triangle(Rgb565::RED)];

    let mut i: usize = 0;
    let theta = 6.28319 / 12.0;
    loop {
        // Draw
        let vertices = &mut triangles[i].primitive.vertices;
        let center = Point::new(
            (vertices[0].x + vertices[1].x + vertices[2].x) / 3,
            (vertices[0].y + vertices[1].y + vertices[2].y) / 3,
        );
        for i in 0..3 {
            vertices[i].x = roundf(
                center.x as f32 + (vertices[i].x as f32 - center.x as f32) * cosf(theta)
                    - (vertices[i].y as f32 - center.y as f32) * sinf(theta),
            ) as i32;
            vertices[i].y = roundf(
                center.y as f32
                    + (vertices[i].x as f32 - center.x as f32) * sinf(theta)
                    + (vertices[i].y as f32 - center.y as f32) * cosf(theta),
            ) as i32;
            rprintln!("{} {}", vertices[i].x, vertices[i].y);
        }

        <_ as embedded_graphics::draw_target::DrawTarget>::clear(&mut display, Rgb565::WHITE)
            .unwrap();
        triangles[i].draw(&mut display).unwrap();
        i ^= 1;

        // Hold
        timer0.delay_ms(1000);
    }
}
