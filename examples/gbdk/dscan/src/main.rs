//! GBDK Color Deep Scan — A direct port of GBDK's dscan example (by Mr. N.U. of TeamKNOx).
//!
//! Ported from the original C code in gbdk-2020/examples/gb/dscan/dscan.c.

#![no_std]
#![no_main]
#![allow(unsafe_op_in_unsafe_fn)]

mod bkg;
mod bkg_c;
mod bkg_m;
mod fore;

use gbdk_sys::gb::cgb::*;
use gbdk_sys::gb::gb::*;
use gbdk_sys::rand::*;

// Palette arrays
#[rustfmt::skip]
static BKG_P: [PaletteColor; 32] = [
    bkg::CGB_PAL0C0, bkg::CGB_PAL0C1, bkg::CGB_PAL0C2, bkg::CGB_PAL0C3,
    bkg::CGB_PAL1C0, bkg::CGB_PAL1C1, bkg::CGB_PAL1C2, bkg::CGB_PAL1C3,
    bkg::CGB_PAL2C0, bkg::CGB_PAL2C1, bkg::CGB_PAL2C2, bkg::CGB_PAL2C3,
    bkg::CGB_PAL3C0, bkg::CGB_PAL3C1, bkg::CGB_PAL3C2, bkg::CGB_PAL3C3,
    bkg::CGB_PAL4C0, bkg::CGB_PAL4C1, bkg::CGB_PAL4C2, bkg::CGB_PAL4C3,
    bkg::CGB_PAL5C0, bkg::CGB_PAL5C1, bkg::CGB_PAL5C2, bkg::CGB_PAL5C3,
    bkg::CGB_PAL6C0, bkg::CGB_PAL6C1, bkg::CGB_PAL6C2, bkg::CGB_PAL6C3,
    bkg::CGB_PAL7C0, bkg::CGB_PAL7C1, bkg::CGB_PAL7C2, bkg::CGB_PAL7C3,
];

#[rustfmt::skip]
static OBJ_P: [PaletteColor; 32] = [
    fore::CGB_PAL0C0, fore::CGB_PAL0C1, fore::CGB_PAL0C2, fore::CGB_PAL0C3,
    fore::CGB_PAL1C0, fore::CGB_PAL1C1, fore::CGB_PAL1C2, fore::CGB_PAL1C3,
    fore::CGB_PAL2C0, fore::CGB_PAL2C1, fore::CGB_PAL2C2, fore::CGB_PAL2C3,
    fore::CGB_PAL3C0, fore::CGB_PAL3C1, fore::CGB_PAL3C2, fore::CGB_PAL3C3,
    fore::CGB_PAL4C0, fore::CGB_PAL4C1, fore::CGB_PAL4C2, fore::CGB_PAL4C3,
    fore::CGB_PAL5C0, fore::CGB_PAL5C1, fore::CGB_PAL5C2, fore::CGB_PAL5C3,
    fore::CGB_PAL6C0, fore::CGB_PAL6C1, fore::CGB_PAL6C2, fore::CGB_PAL6C3,
    fore::CGB_PAL7C0, fore::CGB_PAL7C1, fore::CGB_PAL7C2, fore::CGB_PAL7C3,
];

// Screen size
const MIN_SX: u8 = 5;
const MAX_SX: u8 = 20 - MIN_SX;
const MIN_SY: u8 = 5;
const MAX_SY: u8 = MIN_SY + 13;

const DEF_SP: u8 = 30; // sprite null char code

// Player
const MIN_PX: u8 = MIN_SX * 8 + 8;
const MAX_PX: u8 = MAX_SX * 8 - 8;
const DEF_PX: u8 = 80;
const DEF_PY: u8 = MIN_SY * 8;
const DEF_PC0: u8 = 14;
const DEF_PC1: u8 = 15;
const DEF_PF: u8 = 8;

// Bomb
const MAX_TT: usize = 6;
const DEF_TS: u8 = 2;
const DEF_TC: u8 = 2;
const DEF_TX: u8 = 80 - 6;
const DEF_TY: u8 = DEF_PY - 14;
const MAX_TY: u8 = MAX_SY * 8;

// Enemy
const MAX_ET: usize = 10;
const DEF_ES0: u8 = DEF_TS + MAX_TT as u8;
const DEF_ES1: u8 = DEF_ES0 + 1;
const DEF_1EC0: u8 = 32;
const DEF_1EC1: u8 = 48;
const DEF_2EC0: u8 = 64;
const DEF_2EC1: u8 = 80;
const DEF_XEC0: u8 = 96;
const DEF_XEC1: u8 = 112;
const DEF_EY: u8 = DEF_PY + 12;
const DEF_EH: u8 = 10;
const SUB_EX0: u8 = 20;
const SUB_EX1: u8 = SUB_EX0 - 8;
const MIN_EX: u8 = SUB_EX0 - 16;
const MAX_EX: u8 = SUB_EX0 + 180;
const SPEED_EY: u8 = DEF_EY + DEF_EH * 3;
const DEF_BC1: u8 = 4;
const DEF_BC2: u8 = 5;

// Kirai
const MAX_KT: usize = 12;
const DEF_KS: u8 = DEF_ES0 + MAX_ET as u8 * 2;
const DEF_KC: u8 = 4;
const DEF_KX: u8 = 0;
const DEF_KY: u8 = 0;
const MIN_KY: u8 = DEF_PY + 1;

static mut MSG_TILE: [u8; 64] = [0; 64];

static MSG_1UP: [u8; 4] = *b"1UP\0";
static MSG_LV: [u8; 3] = *b"LV\0";
static MSG_GOVER: [u8; 9] = *b"GAMEOVER\0";
static MSG_PAUSE: [u8; 9] = *b" PAUSE! \0";
static MSG_START: [u8; 9] = *b"        \0";

static mut PF: u8 = 0;
static mut PX: u8 = 0;
static mut PP: u8 = 0;
static mut PL: u8 = 0;
static mut PW: u16 = 0;
static mut PS: u16 = 0;
static mut TF: [u8; MAX_TT] = [0; MAX_TT];
static mut TX: [u8; MAX_TT] = [0; MAX_TT];
static mut TY: [u8; MAX_TT] = [0; MAX_TT];
static mut EF: [u8; MAX_ET] = [0; MAX_ET];
static mut EX: [u8; MAX_ET] = [0; MAX_ET];
static mut EY: [u8; MAX_ET] = [0; MAX_ET];
static mut KF: [u8; MAX_KT] = [0; MAX_KT];
static mut KX: [u8; MAX_KT] = [0; MAX_KT];
static mut KY: [u8; MAX_KT] = [0; MAX_KT];
static mut RND_ENEMY: u8 = 0;
static mut RND_KIRAI: u8 = 0;
static mut K_RIGHT: u8 = 0;
static mut K_LEFT: u8 = 0;

unsafe fn set_sprite_attrb(nb: u8, tile: u8) {
    if device_supports_color() {
        set_sprite_prop(nb, tile);
    }
}

unsafe fn set_bkg_attr(x: u8, y: u8, sx: u8, sy: u8, d: *const u8) {
    let mut d = d;
    VBK_REG.write(VBK_ATTRIBUTES);
    for yy in 0..sy {
        for xx in 0..sx {
            (*core::ptr::addr_of_mut!(MSG_TILE))[xx as usize] = bkg::CGB[*d as usize];
            d = d.add(1);
        }
        set_bkg_tiles(x, y + yy, sx, 1, core::ptr::addr_of!(MSG_TILE) as *const u8);
    }
    VBK_REG.write(VBK_TILES);
}

unsafe fn make_rnd(i: u8) -> u8 {
    arand() % (i + 1)
}

unsafe fn show_score(s: u16) {
    let mut s = s;
    let mut m: u16 = 10000;
    let mut f: u8 = 0;
    let mut score: [u8; 6] = [0; 6];

    for i in 0..5 {
        let n = (s / m) as u8;
        s = s % m;
        m = m / 10;
        if n == 0 && f == 0 {
            score[i] = 0x20; // ' '
        } else {
            f = 1;
            score[i] = 0x30 + n; // '0'-'9'
        }
    }
    score[5] = 0x30; // '0'
    set_bkg_tiles(4, 0, 6, 1, score.as_ptr());
}

unsafe fn set_level(i: u8) {
    if i < 9 {
        RND_ENEMY = 100 - i * 12;
        RND_KIRAI = 50 - i * 6;
    } else {
        RND_ENEMY = 0;
        RND_KIRAI = 0;
    }
}

unsafe fn show_level(i: u8) {
    let mut level: [u8; 2] = [0; 2];
    if i < 9 {
        level[0] = 0x31 + i;
    } else {
        level[0] = 0x41 + i - 9;
    }
    set_bkg_tiles(19, 0, 1, 1, level.as_ptr());
    set_level(i);
}

unsafe fn show_gover() {
    set_bkg_tiles(6, 9, 8, 1, MSG_GOVER.as_ptr());
    PF = DEF_PF;
}

unsafe fn show_pause() {
    set_bkg_tiles(6, 9, 8, 1, MSG_PAUSE.as_ptr());
}

unsafe fn hide_msg() {
    set_bkg_tiles(6, 9, 8, 1, MSG_START.as_ptr());
}

unsafe fn init_score() {
    PS = 0;
    show_score(PS);
    PP = 0;
    PL = 0;
    show_level(PL);
}

unsafe fn init_screen() {
    if device_supports_color() {
        // Transfer color palettes
        set_bkg_palette(0, 1, BKG_P[0..].as_ptr());
        set_bkg_palette(1, 1, BKG_P[4..].as_ptr());
        set_bkg_palette(2, 1, BKG_P[8..].as_ptr());
        set_bkg_palette(3, 1, BKG_P[12..].as_ptr());
        set_bkg_palette(4, 1, BKG_P[16..].as_ptr());
        set_bkg_palette(5, 1, BKG_P[20..].as_ptr());
        set_bkg_palette(6, 1, BKG_P[24..].as_ptr());
        set_bkg_palette(7, 1, BKG_P[28..].as_ptr());
        set_sprite_palette(0, 1, OBJ_P[0..].as_ptr());
        set_sprite_palette(1, 1, OBJ_P[4..].as_ptr());
        set_sprite_palette(2, 1, OBJ_P[8..].as_ptr());
        set_sprite_palette(3, 1, OBJ_P[12..].as_ptr());
        set_sprite_palette(4, 1, OBJ_P[16..].as_ptr());
        set_sprite_palette(5, 1, OBJ_P[20..].as_ptr());
        set_sprite_palette(6, 1, OBJ_P[24..].as_ptr());
        set_sprite_palette(7, 1, OBJ_P[28..].as_ptr());

        // Set attributes
        set_bkg_attr(0, 0, 20, 18, bkg_c::TILE_MAP.as_ptr());
        set_bkg_tiles(0, 0, 20, 18, bkg_c::TILE_MAP.as_ptr());
    } else {
        set_bkg_tiles(0, 0, 20, 18, bkg_m::MAP.as_ptr());
    }

    PW = 50;
    set_bkg_data(0, 96, bkg::TILES.as_ptr());
    set_bkg_tiles(0, 0, 3, 1, MSG_1UP.as_ptr());
    set_bkg_tiles(16, 0, 2, 1, MSG_LV.as_ptr());
    show_bkg();
    sprites_8x8();
    set_sprite_data(0, 128, fore::TILES.as_ptr());
    show_sprites();
    for n in 0..40u8 {
        set_sprite_tile(n, DEF_SP);
        move_sprite(n, 0, 0);
    }
}

unsafe fn init_player() {
    PF = 0;
    PX = DEF_PX;
    set_sprite_tile(0, 0);
    set_sprite_attrb(0, fore::CGB[0]);
    move_sprite(0, PX, DEF_PY);
    set_sprite_tile(1, 1);
    set_sprite_attrb(1, fore::CGB[1]);
    move_sprite(1, PX + 8, DEF_PY);
}

unsafe fn init_tama() {
    for i in 0..MAX_TT {
        TF[i] = 0;
        TX[i] = i as u8 * 4 + DEF_TX;
        TY[i] = DEF_TY;
        set_sprite_tile(i as u8 + DEF_TS, TF[i] + DEF_TC);
        set_sprite_attrb(i as u8 + DEF_TS, fore::CGB[(TF[i] + DEF_TC) as usize]);
        move_sprite(i as u8 + DEF_TS, TX[i], TY[i]);
    }
}

unsafe fn init_enemy() {
    for i in 0..MAX_ET {
        EF[i] = 0;
        EX[i] = 0;
        EY[i] = 0;
        set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_SP);
        set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_SP);
        move_sprite(i as u8 * 2 + DEF_ES0, EX[i], EY[i]);
        move_sprite(i as u8 * 2 + DEF_ES1, EX[i], EY[i]);
    }
}

unsafe fn init_kirai() {
    for i in 0..MAX_KT {
        KF[i] = 0;
        KX[i] = DEF_KX;
        KY[i] = DEF_KY;
        set_sprite_tile(i as u8 + DEF_KS, DEF_SP);
        move_sprite(i as u8 + DEF_KS, KX[i], KY[i]);
    }
}

unsafe fn player() {
    let key = joypad();

    // Pause / start
    if key & J_START != 0 {
        if PF == DEF_PF {
            let mut seed = DIV_REG.read() as u16;
            waitpadup();
            seed |= (DIV_REG.read() as u16) << 8;
            initarand(seed);
            hide_msg();
            init_score();
            init_player();
            init_tama();
            init_enemy();
            init_kirai();
            delay(500);
        } else {
            show_pause();
            waitpadup();
            let mut key = joypad();
            while key & J_START == 0 {
                key = joypad();
                if key & J_DOWN != 0 {
                    if PL > 0 {
                        PL -= 1;
                    }
                    show_level(PL);
                    waitpadup();
                } else if key & J_UP != 0 {
                    if PL < 8 {
                        PL += 1;
                    }
                    show_level(PL);
                    waitpadup();
                } else if key & J_LEFT != 0 {
                    while joypad() & J_LEFT != 0 {
                        if PW > 0 {
                            PW -= 1;
                        }
                        show_score(PW);
                        delay(250);
                    }
                    show_score(PS);
                } else if key & J_RIGHT != 0 {
                    while joypad() & J_RIGHT != 0 {
                        if PW < 99 {
                            PW += 1;
                        }
                        show_score(PW);
                        delay(250);
                    }
                    show_score(PS);
                } else if key & J_SELECT != 0 {
                    let i = K_RIGHT;
                    K_RIGHT = K_LEFT;
                    K_LEFT = i;
                    waitpadup();
                }
            }
            waitpadup();
            hide_msg();
            delay(500);
        }
        return;
    }

    // Dead
    if PF > 1 {
        if PF < DEF_PF {
            set_sprite_tile(0, PF * 2 + DEF_PC0);
            set_sprite_attrb(0, fore::CGB[(PF * 2 + DEF_PC0) as usize]);
            set_sprite_tile(1, PF * 2 + DEF_PC1);
            set_sprite_attrb(1, fore::CGB[(PF * 2 + DEF_PC1) as usize]);
            PF += 1;
        } else {
            set_sprite_tile(0, DEF_SP);
            set_sprite_tile(1, DEF_SP);
            show_gover();
        }
        return;
    }

    // Move
    if (key & J_LEFT != 0) && PX > MIN_PX {
        PX -= 1;
        move_sprite(0, PX, DEF_PY);
        move_sprite(1, PX + 8, DEF_PY);
    } else if (key & J_RIGHT != 0) && PX < MAX_PX {
        PX += 1;
        move_sprite(0, PX, DEF_PY);
        move_sprite(1, PX + 8, DEF_PY);
    }

    // Shot
    if key & K_LEFT != 0 {
        if PF == 0 {
            PF = 1;
            for i in 0..MAX_TT {
                if TF[i] == 0 {
                    TF[i] = 1;
                    TX[i] = PX - 4;
                    TY[i] = DEF_PY;
                    break;
                }
            }
        }
    } else if key & K_RIGHT != 0 {
        if PF == 0 {
            PF = 1;
            for i in 0..MAX_TT {
                if TF[i] == 0 {
                    TF[i] = 1;
                    TX[i] = PX + 12;
                    TY[i] = DEF_PY;
                    break;
                }
            }
        }
    } else if PF == 1 {
        PF = 0;
    }
}

unsafe fn bombs() {
    for i in 0..MAX_TT {
        if TF[i] != 0 {
            TY[i] = TY[i].wrapping_add(1);
            if TY[i] > MAX_TY {
                TF[i] = 0;
                TX[i] = i as u8 * 4 + DEF_TX;
                TY[i] = DEF_TY;
            } else {
                TF[i] = 3 - TF[i];
            }
            set_sprite_tile(i as u8 + DEF_TS, TF[i] + DEF_TC);
            set_sprite_attrb(i as u8 + DEF_TS, fore::CGB[(TF[i] + DEF_TC) as usize]);
            move_sprite(i as u8 + DEF_TS, TX[i], TY[i]);
        }
    }
}

unsafe fn enemys() {
    for i in 0..MAX_ET {
        if EF[i] == 1 {
            // Move right to left
            EX[i] = EX[i].wrapping_sub(1);
            if PL > 0 && EY[i] < SPEED_EY {
                EX[i] = EX[i].wrapping_sub(1);
            }
            if EX[i] <= MIN_EX {
                EF[i] = 0;
                set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_SP);
                set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_SP);
            } else {
                // Sprite animation
                if EX[i] < MIN_SX * 8 + 13 {
                    set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_1EC0);
                    set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_1EC1);
                } else if EX[i] < MIN_SX * 8 + 20 {
                    set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_1EC0);
                    set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_1EC1 + (EX[i] - MIN_SX * 8 - 13));
                } else if EX[i] < MIN_SX * 8 + 28 {
                    set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_1EC0 + (EX[i] - MIN_SX * 8 - 20));
                    set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_1EC1 + 8);
                } else if EX[i] < MAX_SX * 8 + 13 {
                    set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_1EC0 + 8);
                    set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_1EC1 + 8);
                } else if EX[i] < MAX_SX * 8 + 20 {
                    set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_1EC0 + 8);
                    set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_1EC1 + (EX[i] - MAX_SX * 8 - 12) + 7);
                } else if EX[i] < MAX_SX * 8 + 28 {
                    set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_1EC0 + (EX[i] - MAX_SX * 8 - 20) + 8);
                    set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_1EC1);
                } else {
                    set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_1EC0);
                    set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_1EC1);
                }
                move_sprite(i as u8 * 2 + DEF_ES0, EX[i] - SUB_EX0, EY[i]);
                move_sprite(i as u8 * 2 + DEF_ES1, EX[i] - SUB_EX1, EY[i]);
                // Check bomb hit
                for j in 0..MAX_TT {
                    if TF[j] != 0 {
                        if TY[j] > EY[i] - 2 && TY[j] < EY[i] + 2 {
                            if TX[j] > (EX[i] - SUB_EX0 - 5) && TX[j] < (EX[i] - SUB_EX1 + 5) {
                                // Hit
                                TF[j] = 0;
                                TX[j] = j as u8 * 4 + DEF_TX;
                                TY[j] = DEF_TY;
                                set_sprite_tile(j as u8 + DEF_TS, TF[j] + DEF_TC);
                                set_sprite_attrb(j as u8 + DEF_TS, fore::CGB[(TF[j] + DEF_TC) as usize]);
                                move_sprite(j as u8 + DEF_TS, TX[j], TY[j]);
                                EF[i] = 3;
                                set_sprite_tile(i as u8 * 2 + DEF_ES0, EF[i] * 2 + DEF_BC1);
                                set_sprite_attrb(i as u8 * 2 + DEF_ES0, fore::CGB[(EF[i] * 2 + DEF_BC1) as usize]);
                                set_sprite_tile(i as u8 * 2 + DEF_ES1, EF[i] * 2 + DEF_BC2);
                                set_sprite_attrb(i as u8 * 2 + DEF_ES1, fore::CGB[(EF[i] * 2 + DEF_BC2) as usize]);
                            }
                        }
                    }
                }
                if make_rnd(RND_KIRAI) == 0 {
                    if (EX[i] - SUB_EX0) > MIN_PX && (EX[i] - SUB_EX0) < MAX_PX {
                        if KF[i] == 0 {
                            KF[i] = 1;
                            KX[i] = EX[i] - SUB_EX0 + 4;
                            KY[i] = EY[i] - 4;
                        } else if KF[i + 1] == 0 {
                            KF[i + 1] = 1;
                            KX[i + 1] = EX[i] - SUB_EX0 + 4;
                            KY[i + 1] = EY[i] - 4;
                        } else if KF[i + 2] == 0 {
                            KF[i + 2] = 1;
                            KX[i + 2] = EX[i] - SUB_EX0 + 4;
                            KY[i + 2] = EY[i] - 4;
                        }
                    }
                }
            }
        } else if EF[i] == 2 {
            // Move left to right
            EX[i] = EX[i].wrapping_add(1);
            if PL > 0 && EY[i] < SPEED_EY {
                EX[i] = EX[i].wrapping_add(1);
            }
            if EX[i] >= MAX_EX {
                EF[i] = 0;
                set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_SP);
                set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_SP);
            } else {
                if i == 9 {
                    if EX[i] < MIN_SX * 8 + 13 {
                        set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_XEC0);
                        set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_XEC1);
                    } else if EX[i] < MIN_SX * 8 + 20 {
                        set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_XEC0);
                        set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_XEC1 + (EX[i] - MIN_SX * 8 - 13));
                    } else if EX[i] < MIN_SX * 8 + 28 {
                        set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_XEC0 + (EX[i] - MIN_SX * 8 - 20));
                        set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_XEC1 + 8);
                    } else if EX[i] < MAX_SX * 8 + 13 {
                        set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_XEC0 + 8);
                        set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_XEC1 + 8);
                    } else if EX[i] < MAX_SX * 8 + 20 {
                        set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_XEC0 + 8);
                        set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_XEC1 + (EX[i] - MAX_SX * 8 - 12) + 7);
                    } else if EX[i] < MAX_SX * 8 + 28 {
                        set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_XEC0 + (EX[i] - MAX_SX * 8 - 20) + 8);
                        set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_XEC1);
                    } else {
                        set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_XEC0);
                        set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_XEC1);
                    }
                } else {
                    if EX[i] < MIN_SX * 8 + 13 {
                        set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_2EC0);
                        set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_2EC1);
                    } else if EX[i] < MIN_SX * 8 + 20 {
                        set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_2EC0);
                        set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_2EC1 + (EX[i] - MIN_SX * 8 - 13));
                    } else if EX[i] < MIN_SX * 8 + 28 {
                        set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_2EC0 + (EX[i] - MIN_SX * 8 - 20));
                        set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_2EC1 + 8);
                    } else if EX[i] < MAX_SX * 8 + 13 {
                        set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_2EC0 + 8);
                        set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_2EC1 + 8);
                    } else if EX[i] < MAX_SX * 8 + 20 {
                        set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_2EC0 + 8);
                        set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_2EC1 + (EX[i] - MAX_SX * 8 - 12) + 7);
                    } else if EX[i] < MAX_SX * 8 + 28 {
                        set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_2EC0 + (EX[i] - MAX_SX * 8 - 20) + 8);
                        set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_2EC1);
                    } else {
                        set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_2EC0);
                        set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_2EC1);
                    }
                }
                move_sprite(i as u8 * 2 + DEF_ES0, EX[i] - SUB_EX0, EY[i]);
                move_sprite(i as u8 * 2 + DEF_ES1, EX[i] - SUB_EX1, EY[i]);
                // Check bomb hit
                for j in 0..MAX_TT {
                    if TF[j] != 0 {
                        if TY[j] > EY[i] - 2 && TY[j] < EY[i] + 2 {
                            if TX[j] > (EX[i] - SUB_EX0 - 5) && TX[j] < (EX[i] - SUB_EX1 + 5) {
                                TF[j] = 0;
                                TX[j] = j as u8 * 4 + DEF_TX;
                                TY[j] = DEF_TY;
                                set_sprite_tile(j as u8 + DEF_TS, TF[j] + DEF_TC);
                                set_sprite_attrb(j as u8 + DEF_TS, fore::CGB[(TF[j] + DEF_TC) as usize]);
                                move_sprite(j as u8 + DEF_TS, TX[j], TY[j]);
                                EF[i] = 3;
                                set_sprite_tile(i as u8 * 2 + DEF_ES0, EF[i] * 2 + DEF_BC1);
                                set_sprite_attrb(i as u8 * 2 + DEF_ES0, fore::CGB[(EF[i] * 2 + DEF_BC1) as usize]);
                                set_sprite_tile(i as u8 * 2 + DEF_ES1, EF[i] * 2 + DEF_BC2);
                                set_sprite_attrb(i as u8 * 2 + DEF_ES1, fore::CGB[(EF[i] * 2 + DEF_BC2) as usize]);
                            }
                        }
                    }
                }
                if make_rnd(RND_KIRAI) == 0 {
                    if (EX[i] - SUB_EX0) > MIN_PX && (EX[i] - SUB_EX0) < MAX_PX {
                        if KF[i] == 0 {
                            KF[i] = 1;
                            KX[i] = EX[i] - SUB_EX0 + 4;
                            KY[i] = EY[i] - 4;
                        } else if KF[i + 1] == 0 {
                            KF[i + 1] = 1;
                            KX[i + 1] = EX[i] - SUB_EX0 + 4;
                            KY[i + 1] = EY[i] - 4;
                        } else if KF[i + 2] == 0 {
                            KF[i + 2] = 1;
                            KX[i + 2] = EX[i] - SUB_EX0 + 4;
                            KY[i + 2] = EY[i] - 4;
                        }
                    }
                }
            }
        } else if EF[i] >= 3 {
            if EF[i] > 4 {
                EF[i] = 0;
                set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_SP);
                set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_SP);
                if i == 9 {
                    PS += 100;
                    show_score(PS);
                    PP += 1;
                    set_level(PL - 1);
                } else {
                    PS += i as u16 + 1;
                    show_score(PS);
                    PP += 1;
                }
            } else {
                set_sprite_tile(i as u8 * 2 + DEF_ES0, EF[i] * 2 + DEF_BC1);
                set_sprite_attrb(i as u8 * 2 + DEF_ES0, fore::CGB[(EF[i] * 2 + DEF_BC1) as usize]);
                set_sprite_tile(i as u8 * 2 + DEF_ES1, EF[i] * 2 + DEF_BC2);
                set_sprite_attrb(i as u8 * 2 + DEF_ES1, fore::CGB[(EF[i] * 2 + DEF_BC2) as usize]);
                EF[i] += 1;
            }
        } else if i == 9 {
            if PP > 20 {
                PP = 0;
                PL += 1;
                show_level(PL);
                // X enemy
                EY[i] = i as u8 * DEF_EH + DEF_EY;
                EF[i] = (i as u8) % 2 + 1;
                EX[i] = MIN_EX;
                set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_XEC0);
                set_sprite_attrb(i as u8 * 2 + DEF_ES0, fore::CGB[DEF_XEC0 as usize]);
                set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_XEC1);
                set_sprite_attrb(i as u8 * 2 + DEF_ES1, fore::CGB[DEF_XEC1 as usize]);
                move_sprite(i as u8 * 2 + DEF_ES0, EX[i] - SUB_EX0, EY[i]);
                move_sprite(i as u8 * 2 + DEF_ES1, EX[i] - SUB_EX1, EY[i]);
            }
        } else if make_rnd(RND_ENEMY) == 0 {
            if !(PL < 4 && i == 0) {
                // Create enemy
                EY[i] = i as u8 * DEF_EH + DEF_EY;
                EF[i] = (i as u8) % 2 + 1;
                if EF[i] == 1 {
                    EX[i] = MAX_EX;
                    set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_1EC0);
                    set_sprite_attrb(i as u8 * 2 + DEF_ES0, fore::CGB[DEF_1EC0 as usize]);
                    set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_1EC1);
                    set_sprite_attrb(i as u8 * 2 + DEF_ES1, fore::CGB[DEF_1EC1 as usize]);
                } else {
                    EX[i] = MIN_EX;
                    set_sprite_tile(i as u8 * 2 + DEF_ES0, DEF_2EC0);
                    set_sprite_attrb(i as u8 * 2 + DEF_ES0, fore::CGB[DEF_2EC0 as usize]);
                    set_sprite_tile(i as u8 * 2 + DEF_ES1, DEF_2EC1);
                    set_sprite_attrb(i as u8 * 2 + DEF_ES1, fore::CGB[DEF_2EC1 as usize]);
                }
                move_sprite(i as u8 * 2 + DEF_ES0, EX[i] - SUB_EX0, EY[i]);
                move_sprite(i as u8 * 2 + DEF_ES1, EX[i] - SUB_EX1, EY[i]);
            }
        }
    }
}

unsafe fn kirai() {
    for i in 0..MAX_KT {
        if KF[i] != 0 {
            KY[i] = KY[i].wrapping_sub(1);
            if KF[i] >= 3 {
                KF[i] += 1;
                if KX[i] > (PX - 5) && KX[i] < (PX + 12) {
                    if PF < 2 {
                        PF = 2; // out!!
                    }
                }
                if KF[i] >= 6 {
                    KF[i] = 0;
                    KX[i] = DEF_KX;
                    KY[i] = DEF_KY;
                }
            } else if KY[i] <= MIN_KY {
                KF[i] = 3;
            } else {
                KF[i] = 3 - KF[i];
            }
            set_sprite_tile(i as u8 + DEF_KS, KF[i] + DEF_KC);
            set_sprite_attrb(i as u8 + DEF_KS, fore::CGB[(KF[i] + DEF_KC) as usize]);
            move_sprite(i as u8 + DEF_KS, KX[i], KY[i]);
        }
    }
}

#[gb_rt::entry]
fn main() -> ! {
    unsafe {
        gbdk_sys::init();

        disable_interrupts();
        display_off();

        initarand((DIV_REG.read() as u16) << 8);

        init_screen();
        init_score();
        init_player();
        init_tama();
        init_enemy();
        init_kirai();
        show_gover();
        K_RIGHT = J_A;
        K_LEFT = J_B;
        display_on();
        enable_interrupts();

        loop {
            delay(PW);
            player();
            bombs();
            enemys();
            kirai();
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
