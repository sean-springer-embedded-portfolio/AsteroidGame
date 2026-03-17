//! main.rs AsteroidGame
//! Copyright © 2026 Sean Springer
//! [This program is licensed under the "MIT License"]
//! Please see the file LICENSE in the source distribution of this software for license terms.
//!
//! Main driver code for the AsteroidGame 
//! 
//! Sets up the MB2 peripherals, the sturcts herein, the NVIC interrupts, and then 
//! runs the main event loop which grabs the state of the rotary encoder (via the QDEC peripheral) 
//! and updates the screen UI.
//! 
//! rendered on a TFT LCD screen utilizing the GC9A01 driver
//! 
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_graphics::{
    Drawable,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder},
};
use embedded_graphics_framebuf::FrameBuf;

use embedded_hal_bus::spi::ExclusiveDevice;
use microbit::hal::{
    Rng, Spim,
    gpio::{
        Floating, Input, Level, Output, PushPull,
        p0::{P0_03, P0_04, P0_09, P0_10, P0_12, P0_13, P0_17},
        p1::P1_02,
    },
    spim::{self, Frequency},
    timer::Timer,
};
use microbit::pac::{Interrupt, NVIC, TIMER1, TIMER2, interrupt};

use critical_section_lock_mut::LockMut;
use mipidsi::{
    Builder,
    models::GC9A01,
    options::{ColorInversion, Orientation, Rotation},
};
use panic_rtt_target as _;
//use rtt_target::rprintln;
use rtt_target::rtt_init_print;

use core::sync::atomic::{AtomicI32, AtomicU32, Ordering::SeqCst};
use heapless::Vec;
use libm::roundf; // for f32 (single precision)

mod missile;
use missile::Missile;

mod my_qdec;
use my_qdec::{MyQdec, Pins, SamplePeriod};

mod slider;
use slider::Slider;

/// Screen: typedefs: mostly for future lookup for wiring
type SclPin = P0_17<Output<PushPull>>; //e13
type SdaPin = P0_13<Output<PushPull>>; //e15
type DcPin = P0_03<Output<PushPull>>; //e1 - data / command - tell if command is brightnes of color
type CsPin = P1_02<Output<PushPull>>; //e16 - chip selet, wakes up the display
type RstPin = P0_04<Output<PushPull>>; // e2

/// Rotary Encoder: typedefs mostly for future lookup for wiring
type S1 = P0_10<Input<Floating>>; //e08
type S2 = P0_09<Input<Floating>>; //e09
type Key = P0_12<Input<Floating>>; //e12

/// global constants
const SCREEN_PX: usize = 240; // TFT is 240x240
const TICKS_PER_MS: u32 = 1_000_000 / 1_000; // timer peripheral
const FRAME_RATE_MS: u32 = 100; // rate of the update timer
const MIN_COOLDOWN_MS: u32 = 100; // min allowed time between asteroid emission
const ASTEROID_COUNT: usize = 50; // 50 asteroids per game
const ASTEROID_COOLDOWN_RATE: u32 = 100; // ms decrease in spawn rate per asteroid
const CLICKS_PER_DEDENT: i32 = 4; // this rotary encoder clicks 4x per dedent

/// global statics that may be mutated
static MISSLE_V_MIN: AtomicI32 = AtomicI32::new(-30); //px per s (atomic for future builds that may want to mutate)
static MISSLE_V_MAX: AtomicI32 = AtomicI32::new(30); //px per s (atomic for future builds that may want to mutate)
static MISSLE_SPAWN_COOLDOWN_TIMER: AtomicU32 =
    AtomicU32::new(ASTEROID_COUNT as u32 * ASTEROID_COOLDOWN_RATE); //ms; current duration between asteroids: decreases with game evolution
static RND_GEN: LockMut<Rng> = LockMut::new(); // random number gen from peripheral
static MISSILE_LIST: LockMut<Vec<Missile, ASTEROID_COUNT>> = LockMut::new(); // heap vec of the 50 asteroids (missisles)
static SPAWN_TIMER: LockMut<Timer<TIMER2>> = LockMut::new(); // timer peripheral dicctating asteroid spawn
static MOVE_TIMER: LockMut<Timer<TIMER1>> = LockMut::new(); // timer peripheral dictating UI update

/// UI update NVIC interrupt
/// 
/// This interrupt updates the state of non-playable UI (the asteroids / missiles). 
/// It first grabs the missile list LockMut and updates all of their positions based upon the 
/// time interval and then resets the timer peripheral
#[interrupt]
fn TIMER1() {
    MISSILE_LIST.with_lock(|missile_list| {
        for missile in missile_list {
            missile.update_position(FRAME_RATE_MS);
        }
    });

    MOVE_TIMER.with_lock(|move_timer| {
        move_timer.reset_event();
        move_timer.start(FRAME_RATE_MS * TICKS_PER_MS);
    });
}

/// Asteroid spawn NVIC interrupt
/// 
/// Initiates a new asteroid (missile) to be deployed onto the playing field
/// 
/// The duration between asteroids decreases after each deployment. Uses the 
/// Rng peripheral to generate a random X and Y-component of velocity (bound by 
/// the values stored in MISSLE_V_MIN and MISSILE_V_MAX atomics), builds a new 
/// asteroid object (missile), adds it to the MISSILE_LIST, and updates timers
#[interrupt]
fn TIMER2() {
    let mut vx: f32 = 1.0;
    let mut vy: f32 = 1.0;
    let min_v = MISSLE_V_MIN.load(SeqCst);
    let max_v = MISSLE_V_MAX.load(SeqCst);

    RND_GEN.with_lock(|rand_gen| {
        vx = rand_gen.random_u8() as f32 / 255.0; // percentage [0,1]
        vy = rand_gen.random_u8() as f32 / 255.0;
    });

    let off_vx = roundf(vx * (max_v - min_v) as f32 + min_v as f32) as i32; // bound value [min_v, max_v]
    let off_vy = roundf(vy * (max_v - min_v) as f32 + min_v as f32) as i32;

    if off_vx != 0 || off_vy != 0 {
        let missle = Missile::new(off_vx, off_vy);

        MISSILE_LIST.with_lock(|missile_list| {
            let _ = missile_list.push(missle); //ignore capacity full
        });
    }

    // decrease time between next asteroid spawn
    let cur_cooldown = MISSLE_SPAWN_COOLDOWN_TIMER.fetch_sub(ASTEROID_COOLDOWN_RATE, SeqCst);

    if cur_cooldown < MIN_COOLDOWN_MS {
        MISSLE_SPAWN_COOLDOWN_TIMER.store(MIN_COOLDOWN_MS, SeqCst);
    }

    SPAWN_TIMER.with_lock(|span_timer| {
        span_timer.reset_event();
        span_timer.start(MISSLE_SPAWN_COOLDOWN_TIMER.load(SeqCst) * TICKS_PER_MS);
    });
}

/// initializes the 2 timer peripherals
fn init() {
    SPAWN_TIMER.with_lock(|span_timer| {
        span_timer.start(MISSLE_SPAWN_COOLDOWN_TIMER.load(SeqCst) * TICKS_PER_MS);
    });

    MOVE_TIMER.with_lock(|move_timer| {
        move_timer.start(FRAME_RATE_MS * TICKS_PER_MS);
    });
}

/// fn main
/// 
/// initialize the peripherals to be used, sets up some embedded graphics ui plus 
/// the other modules defined herein, and updates the UI plus the slider graphic 
/// based upon the accumulated QDEC value of the rotary encoder
#[entry]
fn main() -> ! {
    rtt_init_print!();

    let missile_vec: Vec<Missile, 50> = Vec::new();
    MISSILE_LIST.init(missile_vec);

    let board = microbit::Board::take().unwrap();
    let mut display_timer = Timer::new(board.TIMER0);
    let mut missle_timer = Timer::new(board.TIMER1);
    missle_timer.enable_interrupt();
    missle_timer.reset_event();
    MOVE_TIMER.init(missle_timer);
    let mut missle_spawn_timer = Timer::new(board.TIMER2);
    missle_spawn_timer.enable_interrupt();
    missle_spawn_timer.reset_event();
    SPAWN_TIMER.init(missle_spawn_timer);

    let random_gen = Rng::new(board.RNG);
    RND_GEN.init(random_gen);

    // Setup Rotary Encoder QDEc
    let s1: S1 = board.edge.e08.into_floating_input();
    let s2: S2 = board.edge.e09.into_floating_input();
    let key: Key = board.edge.e12.into_floating_input();
    let pins = Pins {
        a: s2.degrade(),
        b: s1.degrade(),
        led: Some(key.degrade()),
    };

    //custom QDEC that allows for floating pins
    let q_dec = MyQdec::new(board.QDEC, pins, SamplePeriod::_512us);
    q_dec.enable();
    q_dec.debounce(true);

    // Setup SPI
    let sck: SclPin = board.pins.p0_17.into_push_pull_output(Level::Low);
    let coti: SdaPin = board.pins.p0_13.into_push_pull_output(Level::Low);

    let dc: DcPin = board.edge.e01.into_push_pull_output(Level::Low);
    let cs: CsPin = board.edge.e16.into_push_pull_output(Level::Low);
    let rst: RstPin = board.edge.e02.into_push_pull_output(Level::High);

    let spi_bus = Spim::new(
        board.SPIM3,
        microbit::hal::spim::Pins {
            sck: Some(sck.degrade()),
            mosi: Some(coti.degrade()),
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
        .orientation(Orientation::new().rotate(Rotation::Deg0))
        .invert_colors(ColorInversion::Inverted)
        .reset_pin(rst)
        .init(&mut display_timer)
        .unwrap();

    // frame buffer for rendering speedup
    let mut frame_data = [Rgb565::BLACK; SCREEN_PX * SCREEN_PX];
    let mut frame_buffer = FrameBuf::new(&mut frame_data, SCREEN_PX, SCREEN_PX);
    frame_buffer.clear(Rgb565::BLACK);

    // center source is just a static circle in the middle of the screen
    let center_source = Circle::new(Point::new(120 - 10, 120 - 10), 20).into_styled(
        PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::BLUE)
            .build(),
    );

    // NVIC timer interrupts
    unsafe {
        NVIC::unmask(Interrupt::TIMER1); // non-blockign display timer
        NVIC::unmask(Interrupt::TIMER2); // color change timer
    }; // allow NVIC to handle GPIOTE signals
    //clear any currently pending GPIOTE state
    NVIC::unpend(Interrupt::TIMER1);
    NVIC::unpend(Interrupt::TIMER2);

    let mut slider = Slider::new();
    let mut accumulation: i32 = 0;

    init();

    loop {
        let value = q_dec.read(); //each click is 4 counts, 20 total counts per revolution
        accumulation += value as i32;

        slider.update(accumulation / CLICKS_PER_DEDENT);

        frame_buffer.clear(Rgb565::BLACK);

        MISSILE_LIST.with_lock(|missile_list| {
            for missile in missile_list {
                let obj = missile.get_graphic();
                let pos = missile.get_position();

                // determine if render or not
                if missile.is_alive() {
                    if slider.check_for_collision(&pos) {
                        missile.destroy();
                        continue; //dont render
                    }

                    // else render
                    obj.draw(&mut frame_buffer);
                }
            }
        });

        // render 
        slider.get_graphic().draw(&mut frame_buffer).unwrap();
        center_source.draw(&mut frame_buffer).unwrap();
        display.draw_iter(&frame_buffer).unwrap();
    }
}
