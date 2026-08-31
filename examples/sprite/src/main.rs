//! A character walked around the screen with the d-pad.
//!
//! The art is Eris Esra's Character Template
//! (<https://erisesra.itch.io/character-templates-pack>),
//! converted with:
//!
//! ```text
//! gb-image-fx eris_walk.png --obj --dmg --metasprite 16x16 --tiles-only -o eris_walk
//! ```
//!
//! The sheet is five rows of four: a row per facing, a column per step. An
//! object has a flip bit, so the three leftward facings are the rightward
//! ones flipped rather than tiles of their own.

#![no_std]
#![no_main]

use gb::joypad::{DPadX, DPadY, Pad};
use gb::mmio::{OamAttr, OamEntry, Palette, Shade, Tile};
use gb::ppu::{self, Vblank, obj, palette, tile};

/// Four tiles a cell, twenty cells: five facings by four steps.
const TILES: [Tile; 80] = chop(include_bytes!("../res/eris_walk_tiles.bin"));

/// Split the packed sheet into tiles. `include_bytes!` gives one flat array and
/// [`tile::write_all`] takes them separated.
const fn chop(bytes: &[u8; 80 * 16]) -> [Tile; 80] {
    let mut out = [[0u8; 16]; 80];
    let mut i = 0;
    while i < 80 {
        let mut b = 0;
        while b < 16 {
            out[i][b] = bytes[i * 16 + b];
            b += 1;
        }
        i += 1;
    }
    out
}

/// Which row of the sheet a facing is drawn on.
#[derive(Clone, Copy)]
enum Facing {
    Down = 0,
    DownSide = 1,
    Side = 2,
    UpSide = 3,
    Up = 4,
}

/// The first of a cell's four tiles.
const fn cell(facing: Facing, step: u8) -> u8 {
    (facing as u8 * 4 + step) * 4
}

/// Objects are 8 by 16, so the character is two of them side by side.
const OBJECTS: u8 = 2;

/// Coordinates and speeds are in 32nds of a pixel.
const SUBPIXEL: u8 = 32;

/// The fastest the character travels: half a pixel a frame.
const TOP_SPEED: i8 = SUBPIXEL as i8 / 2;

/// What the d-pad adds to a speed each frame, reaching the top in eight.
const ACCEL: i8 = TOP_SPEED / 8;

/// What a released d-pad takes off a speed each frame, stopping in sixteen.
const FRICTION: i8 = TOP_SPEED / 16;

/// How far the character's top left corner goes.
const LIMIT_X: u16 = (160 - 16) * SUBPIXEL as u16;
const LIMIT_Y: u16 = (144 - 16) * SUBPIXEL as u16;

/// Travel between steps of the walk.
const STEP_EVERY: u8 = 6 * SUBPIXEL;

/// Carry one axis a frame forward, held between 0 and `limit`.
fn advance(pos: &mut u16, speed: &mut i8, dir: i8, limit: u16) {
    if dir != 0 {
        // `dir` is only ever -1, 0 or 1, which a sign test handles without a
        // multiply.
        let push = if dir < 0 { -ACCEL } else { ACCEL };
        // Turning round drops the old speed instead of subtracting through it.
        if (*speed < 0) != (push < 0) {
            *speed = 0;
        }
        *speed = (*speed + push).clamp(-TOP_SPEED, TOP_SPEED);
    } else if *speed > 0 {
        *speed = (*speed - FRICTION).max(0);
    } else {
        *speed = (*speed + FRICTION).min(0);
    }

    let next = pos.wrapping_add_signed(*speed as i16);
    // A negative speed that runs past zero wraps round to a large number.
    if next > limit || (*speed < 0 && next > *pos) {
        *pos = if *speed < 0 { 0 } else { limit };
        *speed = 0;
    } else {
        *pos = next;
    }
}

fn main_loop(vblank: &Vblank) -> ! {
    let mut pad = Pad::new();
    // Halfway along both limits is the middle of the screen.
    let (mut x, mut y) = (LIMIT_X / 2, LIMIT_Y / 2);
    let (mut vx, mut vy) = (0i8, 0i8);
    let (mut facing, mut mirrored) = (Facing::Down, false);
    let (mut step, mut walked) = (0u8, 0u8);

    loop {
        pad.poll();

        let was_still = vx == 0 && vy == 0;

        // A facing changes only where the d-pad points somewhere.
        let row = match (pad.pressed.x(), pad.pressed.y()) {
            (DPadX::Neutral, DPadY::Up) => Some(Facing::Up),
            (DPadX::Neutral, DPadY::Down) => Some(Facing::Down),
            (_, DPadY::Up) => Some(Facing::UpSide),
            (_, DPadY::Down) => Some(Facing::DownSide),
            (DPadX::Neutral, DPadY::Neutral) => None,
            _ => Some(Facing::Side),
        };
        if let Some(row) = row {
            facing = row;
            mirrored = pad.pressed.x() == DPadX::Left;
        }

        advance(&mut x, &mut vx, pad.pressed.x() as i8, LIMIT_X);
        advance(&mut y, &mut vy, pad.pressed.y() as i8, LIMIT_Y);

        // The `walked` counter grows with the speed, keeping the walk in step
        // with the character.
        let speed = vx.unsigned_abs().max(vy.unsigned_abs());
        walked += speed;
        while walked >= STEP_EVERY {
            walked -= STEP_EVERY;
            step = (step + 1) % 4;
        }

        let pressed = pad.just_pressed.left()
            || pad.just_pressed.right()
            || pad.just_pressed.up()
            || pad.just_pressed.down();
        // Start the walk the frame a button goes down, end it the frame the
        // character stops. Steps 1 and 3 are the strides, 0 and 2 standing.
        if was_still && pressed {
            (step, walked) = (1, 0);
        } else if speed == 0 {
            (step, walked) = (0, 0);
        }

        let first = cell(facing, step);
        let attr = OamAttr::new().with_x_flip(mirrored);

        // `Vblank::with` blocks for the next frame itself.
        vblank.with(|d| {
            for n in 0..OBJECTS {
                // The flip bit turns each object over in place, so the two
                // also trade columns.
                let column = if mirrored { OBJECTS - 1 - n } else { n };
                obj::set(
                    d,
                    n,
                    OamEntry {
                        y: (y / SUBPIXEL as u16) as u8 + 16,
                        x: (x / SUBPIXEL as u16) as u8 + 8 + column * 8,
                        tile: first + n * 2,
                        attr,
                    },
                );
            }
        });
    }
}

#[gb::rt::entry]
fn main() -> ! {
    // Writing the tile data takes far longer than a single VBlank, so the
    // screen goes off for VRAM access.
    ppu::with_lcd_off(|d| {
        tile::write_all(d, 0, &TILES);
        for i in OBJECTS..obj::ENTRY_COUNT {
            obj::hide(d, i);
        }
    });

    // An object palette ignores index 0: it is the transparent one.
    palette::set_object(
        palette::ObjSlot::Zero,
        Palette::new()
            .with_id1(Shade::LightGray)
            .with_id2(Shade::DarkGray)
            .with_id3(Shade::Black),
    );

    obj::set_size(obj::Size::Tall);
    obj::set_enabled(true);

    // SAFETY: outside any critical section
    let vblank = unsafe { Vblank::listen() };
    main_loop(&vblank)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
