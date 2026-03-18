# AsteroidGame

Copyright (c) 2026 Sean Springer

Microbit asteriods are inbound! It's up to you to save the universe from complete destruction!

https://github.com/user-attachments/assets/93128a30-d306-49e2-b6a6-325a8fcbd5e8

## What It Is

AsteroidGame is a very simple game running on the MB2 using a TFT GC9A01 LCD screen for display  
and a rotary encoder knob for user controls. The goal is to block all asteroids from leaving the  
screen and save the universe from catastrophe!

## Game Play

Asteroids will emit out of the red center circle object and head outwards towards the screen boarder.  
Using the rotary encoder knob, you are granted control of the blue slider bar which will progress  
around the outer edge of the screen. Your goal: block as many emitted asteroids before they reap havoc  
in the rest of the universe!

A total of 50 asteroids will be emitted over a gameplay with orthogonal veloctiy components randomly  
chosen (using the MB2 hardware RNG) between -30 and 30 pixels per second. The asteroid spawn rate will  
increase as the game progress, starting at 5 second intervals and decreasing by 100ms after each spawn.  
After gameplay has concluded, feel free to hit the reset button on the MB2 to begin a new game!

## How It Works

2 of the MB2 timers are dedicate to spawning new asteroids and updating the positions of currently  
spawned asteroids. The RNG MB2 peripheral is used to randomly generate the newly spawned asteroids  
velocity components (bounded to +- 30 px / sec). The rotary encoder knob state is captured via the QDEC  
peripheral capturing movemenet at 512 us.

Graphic rendering is currently streamed to the display (not DMA) but a frame buffer is first built to  
speed-up rendering. There is no RTOS currnetly implemented - #bare metal games!

The following MB2 peripherals are used by this game:

1. QDEC
2. TIMER0
3. TIMER1
4. TIMER2
5. RNG

The rust embedded graphics crate is used to build each primitive shape shown on the display  
and mipidsi crate is used as the driver to communicate to the LCD display over I2C.

## Physical Setup

For the TFT Display:

1. SclPin = P0_17 / e13
2. SdaPin = P0_13 /e15
3. DcPin = P0_03 / e1
4. CsPin = P1_02 / e16
5. RstPin = P0_04 / e2

for the Rotary Encoder:

1. S1 = P0_10 / e08
2. S2 = P0_09 / e09
3. Key = P0_12 / e12

Image of the MB2 + Edge Connector and the wiring described above:

<img src="imgs/wiring-schematic.png" alt="Wiring Schematic" width="300" height="500">

## To Do

1. Startup Screen
2. Score Keeper
3. End Game Display
4. Game Restart Options

## Build and Run

Assuming you have an attached MB2 with necessary permissions (see [Rust MB2 Discovery Book](https://docs.rust-embedded.org/discovery-mb2/))  
then this program can be `flashed` onto the MB2 nRF52820 using

```bash
cargo embed --release
```

## Sources

1. [Rust MB2 Discovery Book](https://docs.rust-embedded.org/discovery-mb2/)
2. [Rustdoc](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html)
3. Claude Sonnet 4.5 (free version)
4. nRF52833 Product Specification v1.6
5. MicroBit v2.21 Schematic
6. [Microbit Hal Docs](https://docs.rs/microbit/latest/microbit/hal/index.html)
7. [Microbit V2 Crate](https://docs.rs/microbit-v2/latest/microbit/)
8. [HSV](https://en.wikipedia.org/wiki/HSL_and_HSV)

## License

This program is licensed under the "MIT License". Please  
see the file `LICENSE` in the source distribution of this  
software for license terms.
