//! GBDK Super Game Boy Pong, a port of gbdk-2020/examples/gb/sgb_pong.
//!
//! Two-player Pong using the SGB multiplayer joypad support: player 1 on the
//! first controller, player 2 on the second. Must run on a Super Game Boy.

#![no_std]
#![no_main]

use core::ffi::c_char;
use gbdk_sys::gb::gb::*;
use gbdk_sys::gbdk::console::gotoxy;
use gbdk_sys::stdio::printf;

#[rustfmt::skip]
static SPRITE_DATA: [u8; 64] = [
    0x3C,0x3C,0x42,0x7E,0x99,0xFF,0xA9,0xFF,0x89,0xFF,0x89,0xFF,0x42,0x7E,0x3C,0x3C,
    0x3C,0x3C,0x42,0x7E,0xB9,0xFF,0x89,0xFF,0x91,0xFF,0xB9,0xFF,0x42,0x7E,0x3C,0x3C,
    0x3C,0x3C,0x42,0x7E,0x99,0xFF,0x89,0xFF,0x99,0xFF,0x89,0xFF,0x5A,0x7E,0x3C,0x3C,
    0x3C,0x3C,0x42,0x7E,0xA9,0xFF,0xA9,0xFF,0xB9,0xFF,0x89,0xFF,0x42,0x7E,0x3C,0x3C,
];

const YMIN: u8 = 28;
const YMAX: u8 = 100;
const PLAYER1_X: u8 = 16;
const PLAYER2_X: u8 = 20 * 8 - 8;
const INITBALLX: u8 = 80 + 4;
const INITBALLY: u8 = 64 + 8;
const HUD: &[u8] = b" p1: %d   p2: %d\0";

/// Each paddle uses three stacked sprites whose ids are aligned to 4.
unsafe fn init_pad(n: u8) {
    unsafe {
        set_sprite_tile(n << 2, n);
        set_sprite_tile((n << 2) + 1, n);
        set_sprite_tile((n << 2) + 2, n);
    }
}

unsafe fn draw_pad(n: u8, x: u8, y: u8) {
    unsafe {
        move_sprite(n << 2, x, y);
        move_sprite((n << 2) + 1, x, y + 8);
        move_sprite((n << 2) + 2, x, y + 16);
    }
}

#[gb_rt::entry]
fn main() {
    unsafe {
        gbdk_sys::init();

        BGP_REG.write(0xE4);
        OBP0_REG.write(0xE4);
        OBP1_REG.write(0xE4);

        set_sprite_data(0, 4, SPRITE_DATA.as_ptr());
        init_pad(0);
        init_pad(1);
        set_sprite_tile(3, 2); // ball
        show_bkg();
        show_sprites();

        // PAL SGB needs a few frames before the border/multiplayer is ready.
        for _ in 0..4 {
            vsync();
        }

        let mut joypads = JoypadsT { npads: 0, joypads: [0; 4] };
        if joypad_init(2, &raw mut joypads) != 2 {
            printf(b" This program must\n  be executed  on\n   Super GameBoy\0".as_ptr()
                as *const c_char);
            return;
        }

        let mut player1: u8 = 64;
        let mut player2: u8 = 64;
        let mut p1_score: u16 = 0;
        let mut p2_score: u16 = 0;
        printf(HUD.as_ptr() as *const c_char, p1_score as i32, p2_score as i32);

        let mut ball_x: u8 = INITBALLX;
        let mut ball_y: u8 = INITBALLY;
        let mut spd_x: i8 = 1;
        let mut spd_y: i8 = 1;

        loop {
            joypad_ex(&raw mut joypads);
            let joy0 = joypads.joypads[0];
            let joy1 = joypads.joypads[1];

            if joy0 & J_UP != 0 {
                player1 = player1.wrapping_sub(2);
                if player1 < YMIN {
                    player1 = YMIN;
                }
            } else if joy0 & J_DOWN != 0 {
                player1 = player1.wrapping_add(2);
                if player1 > YMAX {
                    player1 = YMAX;
                }
            }
            draw_pad(0, PLAYER1_X, player1);

            if joy1 & J_UP != 0 {
                player2 = player2.wrapping_sub(2);
                if player2 < YMIN {
                    player2 = YMIN;
                }
            } else if joy1 & J_DOWN != 0 {
                player2 = player2.wrapping_add(2);
                if player2 > YMAX {
                    player2 = YMAX;
                }
            }
            draw_pad(1, PLAYER2_X, player2);

            ball_x = ball_x.wrapping_add(spd_x as u8);
            ball_y = ball_y.wrapping_add(spd_y as u8);

            if ball_y < YMIN || ball_y > YMAX + 24 {
                spd_y = -spd_y;
            }
            if ball_x < PLAYER1_X + 8 {
                if ball_y > player1 && ball_y < player1 + 24 && spd_x < 0 {
                    spd_x = -spd_x;
                }
            } else if ball_x > PLAYER2_X - 8
                && ball_y > player2
                && ball_y < player2 + 24
                && spd_x > 0
            {
                spd_x = -spd_x;
            }

            if ball_x < PLAYER1_X {
                ball_x = INITBALLX;
                ball_y = INITBALLY;
                spd_x = -spd_x;
                p2_score += 1;
                gotoxy(0, 0);
                printf(HUD.as_ptr() as *const c_char, p1_score as i32, p2_score as i32);
            } else if ball_x > PLAYER2_X {
                ball_x = INITBALLX;
                ball_y = INITBALLY;
                spd_x = -spd_x;
                p1_score += 1;
                gotoxy(0, 0);
                printf(HUD.as_ptr() as *const c_char, p1_score as i32, p2_score as i32);
            }
            move_sprite(3, ball_x, ball_y);

            vsync();
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
