#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::rprintln;
use rtt_target::rtt_init_print;

use cortex_m_rt::entry;
use microbit::{
    board::Board,
    display::nonblocking::Display,
    hal::{
        Timer,
        gpio::{
            Floating, Input, Level, Output, PushPull,
            p0::{P0_03, P0_04, P0_13, P0_17},
            p1::P1_02,
        },
        gpiote::Gpiote,
        pac, saadc,
        saadc::{Saadc, SaadcConfig},
        spim::{Frequency as SpimFrequency, MODE_0, MODE_3, Pins as SpimPins, Spim},
    },
    pac::{Interrupt, NVIC, TIMER0, TIMER1, TIMER2, TIMER3, interrupt},
};

use display_interface_spi::SPIInterface;
use embedded_hal_bus::spi::ExclusiveDevice;
use gc9a01::{Gc9a01, SPIDisplayInterface, prelude::*};

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Triangle},
};

type SclPin = P0_17<Output<PushPull>>; //e13
type SdaPin = P0_13<Output<PushPull>>; //e15
type DcPin = P0_03<Output<PushPull>>; //e1 - data / command - tell if command is brightnes of color
type CsPin = P1_02<Output<PushPull>>; //e16 - chip selet, wakes up the display
type RstPin = P0_04<Output<PushPull>>; // e2

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = Board::take().unwrap();

    let peripherals = unsafe { pac::Peripherals::steal() };

    let scl_pin = board.pins.p0_17.into_push_pull_output(Level::Low);
    let sda_pin = board.pins.p0_13.into_push_pull_output(Level::Low); //p0-14 is the a button
    let dc = board.edge.e01.into_push_pull_output(Level::Low).degrade();
    let cs = board.edge.e16.into_push_pull_output(Level::Low).degrade(); // P16 as CS, keep High (low V) while writing data and can set to Low (high V) when finished to save power
    let mut rst = board.edge.e02.into_push_pull_output(Level::Low).degrade();
    let mut delay = Timer::new(board.TIMER0);
    let mut delay1 = Timer::new(board.TIMER1);

    let spim_pins = SpimPins {
        sck: Some(scl_pin.degrade()),
        mosi: Some(sda_pin.degrade()),
        miso: None,
    };
    let spim = Spim::new(
        peripherals.SPIM0,
        spim_pins,
        SpimFrequency::M8,
        MODE_0,
        0xFF,
    );
    let spi_device = ExclusiveDevice::new_no_delay(spim, cs).unwrap();
    let interface = SPIDisplayInterface::new(spi_device, dc);
    let mut display = Gc9a01::new(
        interface,
        DisplayResolution240x240,
        DisplayRotation::Rotate0,
    );

    display.reset(&mut rst, &mut delay1).unwrap();
    display.init(&mut delay1).unwrap();

    display.set_draw_area((0, 0), (239, 239)).unwrap();
    let mut iter = [0x00F8u16].into_iter();
    let row: [u8; 240 * 240] = [0x77; 240 * 240];

    display.set_pixels((0, 0), (239, 239), &mut iter).unwrap();
    loop {}
}
