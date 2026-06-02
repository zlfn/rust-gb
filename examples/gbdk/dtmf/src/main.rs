//! GB-DTMF Ver1.0 — A direct port of GBDK's gb-dtmf example.
//!
//! Ported from the original C code in gbdk-2020/examples/gb/gb-dtmf/gb-dtmf.c.
//! by Osamu Ohashi

#![no_std]
#![no_main]
#![allow(unsafe_op_in_unsafe_fn)]

mod frm_lcd;
mod brk_btn;
mod prs_btn;
mod dtmf_lcd;
mod key_num;
mod cursor;

use core::ffi::c_char;
use gbdk_sys::gb::gb::*;
use gbdk_sys::stdio::sprintf;
use gbdk_sys::string::strcpy;

// Display
const TITLE: &[u8] = b" GB-DTMF BY 05AMU \0";

const OFFSET: u8 = 27;
const KEY_STEP: u8 = 24;
const START_CURSOR_X: u8 = 24;
const START_CURSOR_Y: u8 = 72;

const LCD_X: u8 = 1;
const LCD_Y: u8 = 2;
const LCD_WIDTH: u8 = 18;
const LCD_HIGHT: u8 = 2;

const ON: u8 = 1;
const OFF: u8 = 0;

// DTMF
const DTMF_ON: u16 = 100;
const DTMF_OFF: u16 = 100;
const MAX_DTMF: usize = 30;

// Frequency setting
const C1: u8 = 0x94; // 1209Hz
const C2: u8 = 0x9E; // 1336Hz
const C3: u8 = 0xA7; // 1477Hz
const C4: u8 = 0xB0; // 1633Hz

const R1: u8 = 0x44; //  697Hz
const R2: u8 = 0x56; //  770Hz
const R3: u8 = 0x66; //  852Hz
const R4: u8 = 0x75; //  941Hz

static ROW: [u8; 4] = [R1, R2, R3, R4];
static COL: [u8; 4] = [C1, C2, C3, C4];

#[rustfmt::skip]
static DTMF_TILE: [u8; 360] = [
     4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
     0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2,
     3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5,
     3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5,
     6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 8,
     4, 9,10,11, 9,10,11, 9,10,11, 9,10,11, 4, 9,10,11, 9,10,11,
     4,12,13,14,12,13,14,12,13,14,12,13,14, 4,12,13,14,12,13,14,
     4,15,16,17,15,16,17,15,16,17,15,16,17, 4,15,16,17,15,16,17,
     4, 9,10,11, 9,10,11, 9,10,11, 9,10,11, 4, 9,10,11, 9,10,11,
     4,12,13,14,12,13,14,12,13,14,12,13,14, 4,12,13,14,12,13,14,
     4,15,16,17,15,16,17,15,16,17,15,16,17, 4,15,16,17,15,16,17,
     4, 9,10,11, 9,10,11, 9,10,11, 9,10,11, 4, 9,10,11, 9,10,11,
     4,12,13,14,12,13,14,12,13,14,12,13,14, 4,12,13,14,12,13,14,
     4,15,16,17,15,16,17,15,16,17,15,16,17, 4,15,16,17,15,16,17,
     4, 9,10,11, 9,10,11, 9,10,11, 9,10,11, 4, 9,10,10,10,10,11,
     4,12,13,14,12,13,14,12,13,14,12,13,14, 4,12,13,13,13,13,14,
     4,15,16,17,15,16,17,15,16,17,15,16,17, 4,15,16,16,16,16,17,
     4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
];

static BREAK_TILE: [u8; 9] = [9,10,11, 12,13,14, 15,16,17];
static DIALING_BREAK: [u8; 18] = [9,10,10,10,10,11, 12,13,13,13,13,14, 15,16,16,16,16,17];
static PRESS_TILE: [u8; 9] = [18,19,20, 21,22,23, 24,25,26];
static DIALING_PRESS: [u8; 18] = [18,19,19,19,19,20, 21,22,22,22,22,23, 24,25,25,25,25,26];

#[rustfmt::skip]
static INIT_DISP: [u8; 36] = [
    59,59,59,59,59,59,59,59,59,59,59,59,59,59,59,59,59,59,
    60,60,60,60,60,60,60,60,60,60,60,60,60,60,60,60,60,60,
];

#[rustfmt::skip]
static PAD: [[u8; 6]; 4] = [
    [b'1',b'2',b'3',b'A',b'%',b'P'],
    [b'4',b'5',b'6',b'B',b'-',b'F'],
    [b'7',b'8',b'9',b'C',b',',b'?'],
    [b'*',b'0',b'#',b'D',b's',b's'],
];

static mut DISP_TILE: [u8; MAX_DTMF * 2] = [0; MAX_DTMF * 2];

unsafe fn init_dial() {
    NR52_REG.write(0x83);
    NR51_REG.write(0x00);
    NR50_REG.write(0x77);

    NR24_REG.write(0x87);
    NR22_REG.write(0xFF);
    NR21_REG.write(0xBF);

    NR14_REG.write(0x87);
    NR12_REG.write(0xFF);
    NR11_REG.write(0xBF);
    NR10_REG.write(0x04);
}

unsafe fn dialtone(dtmf_on: u16, dtmf_off: u16, str: &[u8]) {
    let mut i: usize = 0;
    while i < str.len() && str[i] != 0 {
        let mut do_tone = true;
        match str[i] {
            b'1' => { NR13_REG.write(R1); NR23_REG.write(C1); }
            b'2' => { NR13_REG.write(R1); NR23_REG.write(C2); }
            b'3' => { NR13_REG.write(R1); NR23_REG.write(C3); }
            b'A' | b'a' => { NR13_REG.write(R1); NR23_REG.write(C4); }
            b'4' => { NR13_REG.write(R2); NR23_REG.write(C1); }
            b'5' => { NR13_REG.write(R2); NR23_REG.write(C2); }
            b'6' => { NR13_REG.write(R2); NR23_REG.write(C3); }
            b'B' | b'b' => { NR13_REG.write(R2); NR23_REG.write(C4); }
            b'7' => { NR13_REG.write(R3); NR23_REG.write(C1); }
            b'8' => { NR13_REG.write(R3); NR23_REG.write(C2); }
            b'9' => { NR13_REG.write(R3); NR23_REG.write(C3); }
            b'C' | b'c' => { NR13_REG.write(R3); NR23_REG.write(C4); }
            b'*' => { NR13_REG.write(R4); NR23_REG.write(C1); }
            b'0' => { NR13_REG.write(R4); NR23_REG.write(C2); }
            b'#' => { NR13_REG.write(R4); NR23_REG.write(C3); }
            b'D' | b'd' => { NR13_REG.write(R4); NR23_REG.write(C4); }
            b',' => {
                delay(dtmf_on);
                delay(dtmf_off);
                NR51_REG.write(0x00);
                do_tone = false;
            }
            _ => {
                NR51_REG.write(0x00);
                do_tone = false;
            }
        }
        if do_tone {
            NR24_REG.write(0x87);
            NR51_REG.write(0x33);
            delay(dtmf_on);
            NR51_REG.write(0x00);
            delay(dtmf_off);
        }
        i += 1;
    }
}

unsafe fn disp_lcd(len: u8, str: &[u8]) {
    let j = len as usize;
    let mut i: usize = 0;
    let disp = core::ptr::addr_of_mut!(DISP_TILE) as *mut u8;
    while i < str.len() && str[i] != 0 {
        if str[i] >= b'0' && str[i] <= b'9' {
            *disp.add(i) = OFFSET + (str[i] - b'0') * 2;
            *disp.add(i + j) = OFFSET + (str[i] - b'0') * 2 + 1;
        }
        match str[i] {
            b'A' => { *disp.add(i) = OFFSET + 10*2; *disp.add(i+j) = OFFSET + 10*2+1; }
            b'B' => { *disp.add(i) = OFFSET + 11*2; *disp.add(i+j) = OFFSET + 11*2+1; }
            b'C' => { *disp.add(i) = OFFSET + 12*2; *disp.add(i+j) = OFFSET + 12*2+1; }
            b'D' => { *disp.add(i) = OFFSET + 13*2; *disp.add(i+j) = OFFSET + 13*2+1; }
            b'#' => { *disp.add(i) = OFFSET + 14*2; *disp.add(i+j) = OFFSET + 14*2+1; }
            b'*' => { *disp.add(i) = OFFSET + 15*2; *disp.add(i+j) = OFFSET + 15*2+1; }
            b' ' => { *disp.add(i) = OFFSET + 16*2; *disp.add(i+j) = OFFSET + 16*2+1; }
            b'Y' => { *disp.add(i) = OFFSET + 17*2; *disp.add(i+j) = OFFSET + 17*2+1; }
            b'M' => { *disp.add(i) = OFFSET + 18*2; *disp.add(i+j) = OFFSET + 18*2+1; }
            b'U' => { *disp.add(i) = OFFSET + 19*2; *disp.add(i+j) = OFFSET + 19*2+1; }
            b'G' => { *disp.add(i) = OFFSET + 20*2; *disp.add(i+j) = OFFSET + 20*2+1; }
            b'-' => { *disp.add(i) = OFFSET + 21*2; *disp.add(i+j) = OFFSET + 21*2+1; }
            b'T' => { *disp.add(i) = OFFSET + 22*2; *disp.add(i+j) = OFFSET + 22*2+1; }
            b',' => { *disp.add(i) = OFFSET + 23*2; *disp.add(i+j) = OFFSET + 23*2+1; }
            b'F' => { *disp.add(i) = OFFSET + 24*2; *disp.add(i+j) = OFFSET + 24*2+1; }
            b'S' => { *disp.add(i) = OFFSET + (b'5'-b'0')*2; *disp.add(i+j) = OFFSET + (b'5'-b'0')*2+1; }
            _ => {}
        }
        i += 1;
    }
}

unsafe fn clr_disp() {
    set_bkg_data(OFFSET, 50, dtmf_lcd::DATA.as_ptr());
    set_bkg_tiles(LCD_X, LCD_Y, LCD_WIDTH, LCD_HIGHT, INIT_DISP.as_ptr());
}

unsafe fn disp(str: &[u8]) {
    let mut no: u8 = 0;
    let mut i: u8;
    let mut work: [u8; MAX_DTMF] = [0; MAX_DTMF];

    clr_disp();

    // Count string length
    while (no as usize) < str.len() && str[no as usize] != 0 {
        no += 1;
    }

    let start_ch: u8;
    let end_ch: u8;
    if no >= LCD_WIDTH {
        start_ch = no - LCD_WIDTH;
        end_ch = LCD_WIDTH;
    } else {
        start_ch = 0;
        end_ch = no;
    }
    i = 0;
    while i < end_ch {
        work[i as usize] = str[(i + start_ch) as usize];
        i += 1;
    }
    work[end_ch as usize] = 0x00;

    disp_lcd(end_ch, &work);

    let left_pos = 19 - end_ch;
    set_bkg_tiles(left_pos, 2, end_ch, LCD_HIGHT, core::ptr::addr_of!(DISP_TILE) as *const u8);
}

unsafe fn press_button(x: u8, y: u8) {
    if x <= 3 && y <= 3 {
        set_bkg_tiles(x * 3 + 1, y * 3 + 5, 3, 3, PRESS_TILE.as_ptr());
    }
    if (x == 4 || x == 5) && y <= 2 {
        set_bkg_tiles(x * 3 + 2, y * 3 + 5, 3, 3, PRESS_TILE.as_ptr());
    }
    if (x == 4 || x == 5) && y == 3 {
        set_bkg_tiles(14, 14, 6, 3, DIALING_PRESS.as_ptr());
    }
}

unsafe fn break_button(x: u8, y: u8) {
    if x <= 3 && y <= 3 {
        set_bkg_tiles(x * 3 + 1, y * 3 + 5, 3, 3, BREAK_TILE.as_ptr());
    }
    if (x == 4 || x == 5) && y <= 2 {
        set_bkg_tiles(x * 3 + 2, y * 3 + 5, 3, 3, BREAK_TILE.as_ptr());
    }
    if (x == 4 || x == 5) && y == 3 {
        set_bkg_tiles(14, 14, 6, 3, DIALING_BREAK.as_ptr());
    }
}

unsafe fn init_key() {
    set_sprite_data(0, 24, key_num::DATA.as_ptr());

    // key pad 1 - 3
    let mut key_y = KEY_STEP + 40;
    for i in 1..=3u8 {
        let key_x = i * KEY_STEP;
        set_sprite_tile(i, i);
        move_sprite(i, key_x, key_y);
    }

    // key pad 4 - 6
    key_y = KEY_STEP * 2 + 40;
    for i in 4..=6u8 {
        let key_x = (i - 3) * KEY_STEP;
        set_sprite_tile(i, i);
        move_sprite(i, key_x, key_y);
    }

    // key pad 7 - 9
    key_y = KEY_STEP * 3 + 40;
    for i in 7..=9u8 {
        let key_x = (i - 6) * KEY_STEP;
        set_sprite_tile(i, i);
        move_sprite(i, key_x, key_y);
    }

    // key pad A - D
    let key_x = KEY_STEP * 4;
    for i in 0..=3u8 {
        key_y = (i + 1) * KEY_STEP + 40;
        set_sprite_tile(i + 10, i + 10);
        move_sprite(i + 10, key_x, key_y);
    }

    // key pad *, 0, #
    set_sprite_tile(15, 15);
    move_sprite(15, KEY_STEP * 1, KEY_STEP * 4 + 40);
    set_sprite_tile(0, 0);
    move_sprite(0, KEY_STEP * 2, KEY_STEP * 4 + 40);
    set_sprite_tile(14, 14);
    move_sprite(14, KEY_STEP * 3, KEY_STEP * 4 + 40);

    // func left
    let key_x = KEY_STEP * 5 + 8;
    for i in 0..=2u8 {
        key_y = (i + 1) * KEY_STEP + 40;
        set_sprite_tile(i + 16, i + 16);
        move_sprite(i + 16, key_x, key_y);
    }

    // func right
    let key_x = KEY_STEP * 6 + 8;
    for i in 0..=2u8 {
        key_y = (i + 1) * KEY_STEP + 40;
        set_sprite_tile(i + 19, i + 19);
        move_sprite(i + 19, key_x, key_y);
    }

    // dialing button
    let key_x = KEY_STEP * 5 + 20;
    let key_y = KEY_STEP * 4 + 40;
    set_sprite_tile(22, 22);
    move_sprite(22, key_x, key_y);
}

unsafe fn init_background() {
    set_bkg_data(0, 9, frm_lcd::DATA.as_ptr());
    set_bkg_data(9, 9, brk_btn::DATA.as_ptr());
    set_bkg_data(18, 9, prs_btn::DATA.as_ptr());
    set_bkg_tiles(0, 0, 20, 18, DTMF_TILE.as_ptr());
}

unsafe fn init_cursor() {
    set_sprite_data(23, 8, cursor::DATA.as_ptr());
    for i in 23..=30u8 {
        set_sprite_tile(i, i);
    }
}

unsafe fn move_cursor(x: u8, y: u8) {
    move_sprite(23, x, y);
    move_sprite(24, x, y + 8);
    move_sprite(25, x + 8, y);
    move_sprite(26, x + 8, y + 8);
}

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    unsafe {
        gbdk_sys::init();

        let mut key1: u8;
        let mut key2: u8 = 0;
        let mut i: u8 = 0;
        let mut j: u8 = 0;
        let mut non_flick: u8 = OFF;
        let mut on_time: u16 = DTMF_ON;
        let mut off_time: u16 = DTMF_OFF;
        let mut str: [u8; MAX_DTMF] = [0; MAX_DTMF];
        let mut str_ms: [u8; 10] = [0; 10];
        let mut ch_pos: u8 = 0;

        disable_interrupts();
        sprites_8x8();

        init_dial();
        init_background();
        init_key();
        init_cursor();
        disp(TITLE);

        show_bkg();
        show_sprites();
        display_on();
        enable_interrupts();

        loop {
            vsync();
            key1 = joypad();

            if key1 != key2 {
                let pos_x = i * KEY_STEP + START_CURSOR_X;
                let pos_y = j * KEY_STEP + START_CURSOR_Y;
                move_cursor(pos_x, pos_y);
            }

            if key2 & J_A != 0 {
                if key1 & J_A != 0 {
                    // value set for each sound reg only numeric key pad
                    if i <= 3 && j <= 3 {
                        NR13_REG.write(ROW[i as usize]);
                        NR23_REG.write(COL[j as usize]);
                        NR24_REG.write(0x87);
                        NR51_REG.write(0x33);
                    }

                    // '?' button
                    if i == 5 && j == 0 && non_flick == OFF {
                        disp(TITLE);
                        non_flick = ON;
                    }

                    // incremental/decremental button
                    if i == 5 && (j == 1 || j == 2) && non_flick == OFF {
                        sprintf(str_ms.as_mut_ptr() as *mut c_char,
                                b"%u MS\0".as_ptr() as *const c_char,
                                on_time as u32);
                        disp(&str_ms);
                        non_flick = ON;
                    }
                } else {
                    // sound output off
                    NR51_REG.write(0x00);
                    break_button(i, j);

                    // return to normal display at release the A button
                    if i == 5 && (j == 0 || j == 1 || j == 2) {
                        non_flick = OFF;
                        if ch_pos == 0 {
                            clr_disp();
                        } else {
                            disp(&str);
                        }
                    }
                }
            } else {
                if key1 & J_A != 0 {
                    // button display handle
                    press_button(i, j);

                    // numeric key pad handling
                    if i <= 3 && j <= 3 {
                        if ch_pos < MAX_DTMF as u8 - 1 {
                            str[ch_pos as usize] = PAD[j as usize][i as usize];
                            ch_pos += 1;
                            str[ch_pos as usize] = 0x00;
                            disp(&str);
                        }
                    }

                    // ',' button
                    if i == 4 && j == 2 {
                        if ch_pos < MAX_DTMF as u8 - 1 {
                            str[ch_pos as usize] = PAD[j as usize][i as usize];
                            ch_pos += 1;
                            str[ch_pos as usize] = 0x00;
                            disp(&str);
                        }
                    }

                    // all clear button
                    if i == 4 && j == 0 {
                        ch_pos = 0x00;
                        strcpy(str.as_mut_ptr() as *mut c_char, b"\0".as_ptr() as *const c_char);
                        clr_disp();
                    }

                    // delete button
                    if i == 4 && j == 1 {
                        if ch_pos > 0 {
                            ch_pos -= 1;
                            str[ch_pos as usize] = 0x00;
                            if ch_pos == 0 {
                                clr_disp();
                            } else {
                                disp(&str);
                            }
                        }
                    }

                    // incremental button
                    if i == 5 && j == 1 {
                        if on_time >= DTMF_ON / 2 {
                            on_time -= 10;
                            off_time -= 10;
                        }
                    }

                    // decremental button
                    if i == 5 && j == 2 {
                        if on_time <= DTMF_ON * 2 {
                            on_time += 10;
                            off_time += 10;
                        }
                    }

                    // dialing button
                    if (i == 4 || i == 5) && j == 3 {
                        dialtone(on_time, off_time, &str);
                    }
                }
            }

            if key1 & J_A == 0 {
                if (key1 & J_UP != 0) && (key2 & J_UP == 0) && j > 0 { j -= 1; }
                else if (key1 & J_DOWN != 0) && (key2 & J_DOWN == 0) && j < 3 { j += 1; }

                if (key1 & J_LEFT != 0) && (key2 & J_LEFT == 0) && i > 0 { i -= 1; }
                else if (key1 & J_RIGHT != 0) && (key2 & J_RIGHT == 0) && i < 5 { i += 1; }
            }

            key2 = key1;
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
