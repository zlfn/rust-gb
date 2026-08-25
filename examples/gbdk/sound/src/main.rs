//! GBDK sound, a port of gbdk-2020/examples/gb/sound.
//!
//! An interactive APU register editor.
//!
//! SELECT picks a channel, UP/DOWN move the cursor, LEFT/RIGHT change a field
//! (hold A/B for bigger steps), START plays, SELECT+A dumps the registers.
//! The unpacked field state lives in `SoundReg`; the `nrNN` methods pack it into
//! the hardware registers. MegaDuck's `translate_*` quirks are omitted (this
//! targets the GB).

#![no_std]
#![no_main]

use core::ffi::{c_char, c_int};
use gbdk_sys::gb::gb::*;
use gbdk_sys::gbdk::console::{gotoxy, setchar};
use gbdk_sys::stdio::printf;

const ARROW_X: u8 = 0;
const VAL_X: u8 = 15;
const TITLE_Y: u8 = 0;
const FIRST_X: u8 = ARROW_X + 1;
const FIRST_Y: u8 = TITLE_Y + 2;

const PLAY: u8 = 0x20;
const FREQUENCY: u8 = 0x21;
const NB_MODES: u8 = 5;

const ARROW_CHAR: c_char = b'>' as c_char;
const SPACE_CHAR: c_char = b' ' as c_char;

// ===== Text helpers (printf-based, like the C original) =====

fn print(s: &[u8]) {
    unsafe { printf(b"%s\0".as_ptr() as *const c_char, s.as_ptr() as *const c_char) };
}

fn printn(n: u16, base: u8) {
    unsafe {
        if base == 16 {
            const HEX: &[u8; 16] = b"0123456789ABCDEF";
            printf(b"%c\0".as_ptr() as *const c_char, HEX[((n >> 4) & 0xF) as usize] as c_int);
            printf(b"%c\0".as_ptr() as *const c_char, HEX[(n & 0xF) as usize] as c_int);
        } else {
            printf(b"%d\0".as_ptr() as *const c_char, n as c_int);
        }
    }
}

fn println(n: u16, base: u8) {
    printn(n, base);
    print(b"\n\0");
}

fn clss() {
    for i in 0..18u8 {
        unsafe { gotoxy(0, i) };
        print(b"                    \0");
    }
}

// ===== Menu tables: (name, max). Index 0 is the title. =====

type Param = (&'static [u8], u16);

const PARAMS_0: &[Param] = &[
    (b"Main Controls\0", 0),
    (b"All On/Off\0", 1),
    (b"Vin->SO1\0", 1),
    (b"Vin->SO2\0", 1),
    (b"SO1 Volume\0", 7),
    (b"SO2 Volume\0", 7),
];
const PARAMS_1: &[Param] = &[
    (b"Sound Mode #1\0", 0),
    (b"Swp Time\0", 7),
    (b"Swp Mode\0", 1),
    (b"Swp Shifts\0", 7),
    (b"Pat Duty\0", 3),
    (b"Sound Len\0", 63),
    (b"Env Init\0", 15),
    (b"Env Mode\0", 1),
    (b"Env Nb Swp\0", 7),
    (b"Frequency\0", 2047),
    (b"Cons Sel\0", 1),
    (b"Out to SO1\0", 1),
    (b"Out to SO2\0", 1),
    (b"On/Off\0", 1),
];
const PARAMS_2: &[Param] = &[
    (b"Sound Mode #2\0", 0),
    (b"Pat Duty\0", 3),
    (b"Sound Len\0", 63),
    (b"Env Init\0", 15),
    (b"Env Mode\0", 1),
    (b"Env Nb Step\0", 7),
    (b"Frequency\0", 2047),
    (b"Cons Sel\0", 1),
    (b"Out to SO1\0", 1),
    (b"Out to SO2\0", 1),
    (b"On/Off\0", 1),
];
const PARAMS_3: &[Param] = &[
    (b"Sound Mode #3\0", 0),
    (b"Sound On/Off\0", 1),
    (b"Sound Len\0", 255),
    (b"Sel Out Level\0", 3),
    (b"Frequency\0", 2047),
    (b"Cons Sel\0", 1),
    (b"Out to SO1\0", 1),
    (b"Out to SO2\0", 1),
    (b"On/Off\0", 1),
];
const PARAMS_4: &[Param] = &[
    (b"Sound Mode #4\0", 0),
    (b"Sound Len\0", 63),
    (b"Env Init\0", 15),
    (b"Env Mode\0", 1),
    (b"Env Nb Step\0", 7),
    (b"Poly Cnt Freq\0", 15),
    (b"Poly Cnt Step\0", 1),
    (b"Poly Cnt Div\0", 7),
    (b"Cons Sel\0", 1),
    (b"Out to SO1\0", 1),
    (b"Out to SO2\0", 1),
    (b"On/Off\0", 1),
];

fn params(mode: u8) -> &'static [Param] {
    match mode {
        0 => PARAMS_0,
        1 => PARAMS_1,
        2 => PARAMS_2,
        3 => PARAMS_3,
        _ => PARAMS_4,
    }
}

// A tone: C3..A3 etc.; indices into `FREQUENCIES`. `SILENCE`/`END` are markers.
const SILENCE: u8 = 72;
const END: u8 = 73;
const FREQUENCIES: [u16; 72] = [
    44, 156, 262, 363, 457, 547, 631, 710, 786, 854, 923, 986, 1046, 1102, 1155, 1205, 1253, 1297,
    1339, 1379, 1417, 1452, 1486, 1517, 1546, 1575, 1602, 1627, 1650, 1673, 1694, 1714, 1732, 1750,
    1767, 1783, 1798, 1812, 1825, 1837, 1849, 1860, 1871, 1881, 1890, 1899, 1907, 1915, 1923, 1930,
    1936, 1943, 1949, 1954, 1959, 1964, 1969, 1974, 1978, 1982, 1985, 1988, 1992, 1995, 1998, 2001,
    2004, 2006, 2009, 2011, 2013, 2015,
];
const MUSIC: &[u8] = &[
    36, 36, 43, 43, 45, 45, 43, SILENCE, 41, 41, 40, 40, 38, 38, 36, SILENCE, 43, 43, 41, 41, 40, 40,
    38, 38, 43, 43, 41, 41, 40, 40, 38, 38, 36, 36, 43, 43, 45, 45, 43, SILENCE, 41, 41, 40, 40, 38,
    38, 36, SILENCE, END,
];

// ===== Unpacked register state =====

#[derive(Default)]
struct Mode1 {
    sweep_shifts: u8,
    sweep_mode: u8,
    sweep_time: u8,
    sound_length: u8,
    pattern_duty: u8,
    env_nb_sweep: u8,
    env_mode: u8,
    env_initial_value: u8,
    frequency_low: u8,
    frequency_high: u8,
    counter_cons_sel: u8,
    restart: u8,
}
#[derive(Default)]
struct Mode2 {
    sound_length: u8,
    pattern_duty: u8,
    env_nb_step: u8,
    env_mode: u8,
    env_initial_value: u8,
    frequency_low: u8,
    frequency_high: u8,
    counter_cons_sel: u8,
    restart: u8,
}
#[derive(Default)]
struct Mode3 {
    on_off: u8,
    sound_length: u8,
    sel_output_level: u8,
    frequency_low: u8,
    frequency_high: u8,
    counter_cons_sel: u8,
    restart: u8,
}
#[derive(Default)]
struct Mode4 {
    sound_length: u8,
    env_nb_step: u8,
    env_mode: u8,
    env_initial_value: u8,
    poly_counter_div: u8,
    poly_counter_step: u8,
    poly_counter_freq: u8,
    counter_cons_sel: u8,
    restart: u8,
}
#[derive(Default)]
struct Control {
    so1_level: u8,
    vin_so1: u8,
    so2_level: u8,
    vin_so2: u8,
    to_so1: [u8; 4],
    to_so2: [u8; 4],
    on: [u8; 4],
    global_on: u8,
}

#[derive(Default)]
struct SoundReg {
    m1: Mode1,
    m2: Mode2,
    m3: Mode3,
    m4: Mode4,
    c: Control,
}

static mut SREG: SoundReg = SoundReg {
    m1: Mode1 { sweep_shifts: 0, sweep_mode: 0, sweep_time: 0, sound_length: 1, pattern_duty: 2,
        env_nb_sweep: 3, env_mode: 0, env_initial_value: 4, frequency_low: 0x73, frequency_high: 6,
        counter_cons_sel: 0, restart: 0 },
    m2: Mode2 { sound_length: 1, pattern_duty: 2, env_nb_step: 4, env_mode: 0, env_initial_value: 8,
        frequency_low: 0xD7, frequency_high: 6, counter_cons_sel: 0, restart: 0 },
    m3: Mode3 { on_off: 1, sound_length: 0, sel_output_level: 3, frequency_low: 0xD6,
        frequency_high: 6, counter_cons_sel: 1, restart: 0 },
    m4: Mode4 { sound_length: 58, env_nb_step: 1, env_mode: 0, env_initial_value: 10,
        poly_counter_div: 0, poly_counter_step: 0, poly_counter_freq: 0, counter_cons_sel: 1,
        restart: 0 },
    c: Control { so1_level: 7, vin_so1: 0, so2_level: 7, vin_so2: 0, to_so1: [1, 1, 1, 1],
        to_so2: [1, 1, 1, 1], on: [0, 0, 0, 0], global_on: 1 },
};

/// The global register state as a place expression.
///
/// Goes through `&raw mut`, so no reference to the static is created.
macro_rules! sr {
    () => {
        (*(&raw mut SREG))
    };
}

impl SoundReg {
    fn nr10(&self) -> u8 { self.m1.sweep_shifts | (self.m1.sweep_mode << 3) | (self.m1.sweep_time << 4) }
    fn nr11(&self) -> u8 { self.m1.sound_length | (self.m1.pattern_duty << 6) }
    fn nr12(&self) -> u8 { self.m1.env_nb_sweep | (self.m1.env_mode << 3) | (self.m1.env_initial_value << 4) }
    fn nr13(&self) -> u8 { self.m1.frequency_low }
    fn nr14(&self) -> u8 { self.m1.frequency_high | (self.m1.counter_cons_sel << 6) | (self.m1.restart << 7) }

    fn nr21(&self) -> u8 { self.m2.sound_length | (self.m2.pattern_duty << 6) }
    fn nr22(&self) -> u8 { self.m2.env_nb_step | (self.m2.env_mode << 3) | (self.m2.env_initial_value << 4) }
    fn nr23(&self) -> u8 { self.m2.frequency_low }
    fn nr24(&self) -> u8 { self.m2.frequency_high | (self.m2.counter_cons_sel << 6) | (self.m2.restart << 7) }

    fn nr30(&self) -> u8 { self.m3.on_off << 7 }
    fn nr31(&self) -> u8 { self.m3.sound_length }
    fn nr32(&self) -> u8 { self.m3.sel_output_level << 5 }
    fn nr33(&self) -> u8 { self.m3.frequency_low }
    fn nr34(&self) -> u8 { self.m3.frequency_high | (self.m3.counter_cons_sel << 6) | (self.m3.restart << 7) }

    fn nr41(&self) -> u8 { self.m4.sound_length }
    fn nr42(&self) -> u8 { self.m4.env_nb_step | (self.m4.env_mode << 3) | (self.m4.env_initial_value << 4) }
    fn nr43(&self) -> u8 { self.m4.poly_counter_div | (self.m4.poly_counter_step << 3) | (self.m4.poly_counter_freq << 4) }
    fn nr44(&self) -> u8 { (self.m4.counter_cons_sel << 6) | (self.m4.restart << 7) }

    fn nr50(&self) -> u8 { self.c.so1_level | (self.c.vin_so1 << 3) | (self.c.so2_level << 4) | (self.c.vin_so2 << 7) }
    fn nr51(&self) -> u8 {
        self.c.to_so1[0] | (self.c.to_so1[1] << 1) | (self.c.to_so1[2] << 2) | (self.c.to_so1[3] << 3)
            | (self.c.to_so2[0] << 4) | (self.c.to_so2[1] << 5) | (self.c.to_so2[2] << 6) | (self.c.to_so2[3] << 7)
    }
    fn nr52(&self) -> u8 { self.c.global_on << 7 }

    fn current_value(&self, mode: u8, line: u8) -> u16 {
        match mode {
            0 => match line {
                0 => self.c.global_on as u16,
                1 => self.c.vin_so1 as u16,
                2 => self.c.vin_so2 as u16,
                3 => self.c.so1_level as u16,
                4 => self.c.so2_level as u16,
                _ => 0,
            },
            1 => match line {
                0 => self.m1.sweep_time as u16,
                1 => self.m1.sweep_mode as u16,
                2 => self.m1.sweep_shifts as u16,
                3 => self.m1.pattern_duty as u16,
                4 => self.m1.sound_length as u16,
                5 => self.m1.env_initial_value as u16,
                6 => self.m1.env_mode as u16,
                7 => self.m1.env_nb_sweep as u16,
                8 | FREQUENCY => ((self.m1.frequency_high as u16) << 8) | self.m1.frequency_low as u16,
                9 => self.m1.counter_cons_sel as u16,
                10 => self.c.to_so1[0] as u16,
                11 => self.c.to_so2[0] as u16,
                12 => self.c.on[0] as u16,
                _ => 0,
            },
            2 => match line {
                0 => self.m2.pattern_duty as u16,
                1 => self.m2.sound_length as u16,
                2 => self.m2.env_initial_value as u16,
                3 => self.m2.env_mode as u16,
                4 => self.m2.env_nb_step as u16,
                5 | FREQUENCY => ((self.m2.frequency_high as u16) << 8) | self.m2.frequency_low as u16,
                6 => self.m2.counter_cons_sel as u16,
                7 => self.c.to_so1[1] as u16,
                8 => self.c.to_so2[1] as u16,
                9 => self.c.on[1] as u16,
                _ => 0,
            },
            3 => match line {
                0 => self.m3.on_off as u16,
                1 => self.m3.sound_length as u16,
                2 => self.m3.sel_output_level as u16,
                3 | FREQUENCY => ((self.m3.frequency_high as u16) << 8) | self.m3.frequency_low as u16,
                4 => self.m3.counter_cons_sel as u16,
                5 => self.c.to_so1[2] as u16,
                6 => self.c.to_so2[2] as u16,
                7 => self.c.on[2] as u16,
                _ => 0,
            },
            _ => match line {
                0 => self.m4.sound_length as u16,
                1 => self.m4.env_initial_value as u16,
                2 => self.m4.env_mode as u16,
                3 => self.m4.env_nb_step as u16,
                4 => self.m4.poly_counter_freq as u16,
                5 => self.m4.poly_counter_step as u16,
                6 => self.m4.poly_counter_div as u16,
                7 => self.m4.counter_cons_sel as u16,
                8 => self.c.to_so1[3] as u16,
                9 => self.c.to_so2[3] as u16,
                10 => self.c.on[3] as u16,
                _ => 0,
            },
        }
    }

    fn update_value(&mut self, mode: u8, line: u8, value: u16) {
        let v = value as u8;
        unsafe {
            match mode {
                0 => match line {
                    0 => { self.c.global_on = v; NR52_REG.write(self.nr52()) }
                    1 => { self.c.vin_so1 = v; NR50_REG.write(self.nr50()) }
                    2 => { self.c.vin_so2 = v; NR50_REG.write(self.nr50()) }
                    3 => { self.c.so1_level = v; NR50_REG.write(self.nr50()) }
                    4 => { self.c.so2_level = v; NR50_REG.write(self.nr50()) }
                    FREQUENCY => {
                        self.update_value(1, FREQUENCY, value);
                        self.update_value(2, FREQUENCY, value);
                        self.update_value(3, FREQUENCY, value);
                    }
                    PLAY => {
                        self.update_value(1, FREQUENCY, self.current_value(1, FREQUENCY));
                        self.update_value(2, FREQUENCY, self.current_value(2, FREQUENCY));
                        self.update_value(3, FREQUENCY, self.current_value(3, FREQUENCY));
                        self.m1.restart = v; self.m2.restart = v; self.m3.restart = v; self.m4.restart = v;
                        NR14_REG.write(self.nr14()); NR24_REG.write(self.nr24());
                        NR34_REG.write(self.nr34()); NR44_REG.write(self.nr44());
                        self.m1.restart = 0; self.m2.restart = 0; self.m3.restart = 0; self.m4.restart = 0;
                    }
                    _ => {}
                },
                1 => match line {
                    0 => { self.m1.sweep_time = v; NR10_REG.write(self.nr10()) }
                    1 => { self.m1.sweep_mode = v; NR10_REG.write(self.nr10()) }
                    2 => { self.m1.sweep_shifts = v; NR10_REG.write(self.nr10()) }
                    3 => { self.m1.pattern_duty = v; NR11_REG.write(self.nr11()) }
                    4 => { self.m1.sound_length = v; NR11_REG.write(self.nr11()) }
                    5 => { self.m1.env_initial_value = v; NR12_REG.write(self.nr12()) }
                    6 => { self.m1.env_mode = v; NR12_REG.write(self.nr12()) }
                    7 => { self.m1.env_nb_sweep = v; NR12_REG.write(self.nr12()) }
                    8 | FREQUENCY => {
                        self.m1.frequency_high = (value >> 8) as u8; self.m1.frequency_low = value as u8;
                        NR13_REG.write(self.nr13()); NR14_REG.write(self.nr14());
                    }
                    9 => { self.m1.counter_cons_sel = v; NR14_REG.write(self.nr14()) }
                    10 => { self.c.to_so1[0] = v; NR51_REG.write(self.nr51()) }
                    11 => { self.c.to_so2[0] = v; NR51_REG.write(self.nr51()) }
                    12 => { self.c.on[0] = v; NR52_REG.write(self.nr52()) }
                    PLAY => {
                        self.update_value(mode, FREQUENCY, self.current_value(mode, FREQUENCY));
                        if self.m1.counter_cons_sel == 1 { NR11_REG.write(self.nr11()) }
                        self.m1.restart = v; NR14_REG.write(self.nr14()); self.m1.restart = 0;
                    }
                    _ => {}
                },
                2 => match line {
                    0 => { self.m2.pattern_duty = v; NR21_REG.write(self.nr21()) }
                    1 => { self.m2.sound_length = v; NR21_REG.write(self.nr21()) }
                    2 => { self.m2.env_initial_value = v; NR22_REG.write(self.nr22()) }
                    3 => { self.m2.env_mode = v; NR22_REG.write(self.nr22()) }
                    4 => { self.m2.env_nb_step = v; NR22_REG.write(self.nr22()) }
                    5 | FREQUENCY => {
                        self.m2.frequency_high = (value >> 8) as u8; self.m2.frequency_low = value as u8;
                        NR23_REG.write(self.nr23()); NR24_REG.write(self.nr24());
                    }
                    6 => { self.m2.counter_cons_sel = v; NR24_REG.write(self.nr24()) }
                    7 => { self.c.to_so1[1] = v; NR51_REG.write(self.nr51()) }
                    8 => { self.c.to_so2[1] = v; NR51_REG.write(self.nr51()) }
                    9 => { self.c.on[1] = v; NR52_REG.write(self.nr52()) }
                    PLAY => {
                        self.update_value(mode, FREQUENCY, self.current_value(mode, FREQUENCY));
                        if self.m2.counter_cons_sel == 1 { NR21_REG.write(self.nr21()) }
                        self.m2.restart = v; NR24_REG.write(self.nr24()); self.m2.restart = 0;
                    }
                    _ => {}
                },
                3 => match line {
                    0 => { self.m3.on_off = v; NR30_REG.write(self.nr30()) }
                    1 => { self.m3.sound_length = v; NR31_REG.write(self.nr31()) }
                    2 => { self.m3.sel_output_level = v; NR32_REG.write(self.nr32()) }
                    3 | FREQUENCY => {
                        self.m3.frequency_high = (value >> 8) as u8; self.m3.frequency_low = value as u8;
                        NR33_REG.write(self.nr33()); NR34_REG.write(self.nr34());
                    }
                    4 => { self.m3.counter_cons_sel = v; NR34_REG.write(self.nr34()) }
                    5 => { self.c.to_so1[2] = v; NR51_REG.write(self.nr51()) }
                    6 => { self.c.to_so2[2] = v; NR51_REG.write(self.nr51()) }
                    7 => { self.c.on[2] = v; NR52_REG.write(self.nr52()) }
                    PLAY => {
                        self.update_value(mode, FREQUENCY, self.current_value(mode, FREQUENCY));
                        if self.m3.counter_cons_sel == 1 { NR31_REG.write(self.nr31()) }
                        self.m3.restart = v; NR34_REG.write(self.nr34()); self.m3.restart = 0;
                    }
                    _ => {}
                },
                _ => match line {
                    0 => { self.m4.sound_length = v; NR41_REG.write(self.nr41()) }
                    1 => { self.m4.env_initial_value = v; NR42_REG.write(self.nr42()) }
                    2 => { self.m4.env_mode = v; NR42_REG.write(self.nr42()) }
                    3 => { self.m4.env_nb_step = v; NR42_REG.write(self.nr42()) }
                    4 => { self.m4.poly_counter_freq = v; NR43_REG.write(self.nr43()) }
                    5 => { self.m4.poly_counter_step = v; NR43_REG.write(self.nr43()) }
                    6 => { self.m4.poly_counter_div = v; NR43_REG.write(self.nr43()) }
                    7 => { self.m4.counter_cons_sel = v; NR44_REG.write(self.nr44()) }
                    8 => { self.c.to_so1[3] = v; NR51_REG.write(self.nr51()) }
                    9 => { self.c.to_so2[3] = v; NR51_REG.write(self.nr51()) }
                    10 => { self.c.on[3] = v; NR52_REG.write(self.nr52()) }
                    PLAY => {
                        if self.m4.counter_cons_sel == 1 { NR41_REG.write(self.nr41()) }
                        self.m4.restart = v; NR44_REG.write(self.nr44()); self.m4.restart = 0;
                    }
                    _ => {}
                },
            }
        }
    }
}

// ===== Display =====

/// Draw the channel screen; returns the last editable line index.
fn draw_screen(mode: u8) -> u8 {
    let p = params(mode);
    clss();
    unsafe { gotoxy(FIRST_X, TITLE_Y) };
    print(p[0].0);
    for (i, param) in p[1..].iter().enumerate() {
        let i = i as u8;
        unsafe { gotoxy(FIRST_X, FIRST_Y + i) };
        print(param.0);
        unsafe { gotoxy(VAL_X, FIRST_Y + i) };
        println(unsafe { sr!().current_value(mode, i) }, 10);
    }
    (p.len() - 2) as u8
}

fn play_music(mode: u8) {
    let mut i = 0;
    while MUSIC[i] != END {
        if MUSIC[i] != SILENCE {
            unsafe {
                sr!().update_value(mode, FREQUENCY, FREQUENCIES[MUSIC[i] as usize]);
                sr!().update_value(mode, PLAY, 1);
            }
        }
        unsafe { delay(500) };
        i += 1;
    }
}

fn show_register_channel(mode: u8) {
    unsafe {
        gotoxy(0, 16);
        let r = &sr!();
        match mode {
            1 => {
                print(b"NR10-14:\0");
                gotoxy(1, 17);
                printn(r.nr10() as u16, 16); print(b", \0");
                printn(r.nr11() as u16, 16); print(b", \0");
                printn(r.nr12() as u16, 16); print(b", \0");
                printn(r.nr13() as u16, 16); print(b", \0");
                printn((0x80 | r.nr14()) as u16, 16);
            }
            2 => {
                print(b"NR21-24:\0");
                gotoxy(1, 17);
                printn(r.nr21() as u16, 16); print(b", \0");
                printn(r.nr22() as u16, 16); print(b", \0");
                printn(r.nr23() as u16, 16); print(b", \0");
                printn((0x80 | r.nr24()) as u16, 16);
            }
            3 => {
                print(b"NR30-34:\0");
                gotoxy(1, 17);
                printn(r.nr30() as u16, 16); print(b", \0");
                printn(r.nr31() as u16, 16); print(b", \0");
                printn(r.nr32() as u16, 16); print(b", \0");
                printn(r.nr33() as u16, 16); print(b", \0");
                printn((0x80 | r.nr34()) as u16, 16);
            }
            4 => {
                print(b"NR41-44:\0");
                gotoxy(1, 17);
                printn(r.nr41() as u16, 16); print(b", \0");
                printn(r.nr42() as u16, 16); print(b", \0");
                printn(r.nr43() as u16, 16); print(b", \0");
                printn((0x80 | r.nr44()) as u16, 16);
            }
            _ => {
                print(b"NR50-52:\0");
                gotoxy(1, 17);
                printn(r.nr50() as u16, 16); print(b", \0");
                printn(r.nr51() as u16, 16); print(b", \0");
                printn(r.nr52() as u16, 16); print(b", \0");
            }
        }
    }
}

fn dump_registers() {
    clss();
    unsafe { gotoxy(FIRST_X, TITLE_Y) };
    print(b"Register Dump\n\n\0");
    unsafe {
        let r = &sr!();
        print(b"NR10:\0"); println(r.nr10() as u16, 16);
        print(b"NR11:\0"); printn(r.nr11() as u16, 16); print(b" NR21:\0"); println(r.nr21() as u16, 16);
        print(b"NR12:\0"); printn(r.nr12() as u16, 16); print(b" NR22:\0"); println(r.nr22() as u16, 16);
        print(b"NR13:\0"); printn(r.nr13() as u16, 16); print(b" NR23:\0"); println(r.nr23() as u16, 16);
        print(b"NR14:\0"); printn((0x80 | r.nr14()) as u16, 16); print(b" NR24:\0"); println((0x80 | r.nr24()) as u16, 16);
        print(b"\n\0");
        print(b"NR30:\0"); println(r.nr30() as u16, 16);
        print(b"NR31:\0"); printn(r.nr31() as u16, 16); print(b" NR41:\0"); println(r.nr41() as u16, 16);
        print(b"NR32:\0"); printn(r.nr32() as u16, 16); print(b" NR42:\0"); println(r.nr42() as u16, 16);
        print(b"NR33:\0"); printn(r.nr33() as u16, 16); print(b" NR43:\0"); println(r.nr43() as u16, 16);
        print(b"NR34:\0"); printn((0x80 | r.nr34()) as u16, 16); print(b" NR44:\0"); println((0x80 | r.nr44()) as u16, 16);
        print(b"\n\0");
        print(b"NR50:\0"); println(r.nr50() as u16, 16);
        print(b"NR51:\0"); println(r.nr51() as u16, 16);
        print(b"NR52:\0"); println(r.nr52() as u16, 16);
    }
}

// ===== Editor loop =====

static mut KEYS: u8 = 0;
static mut PREV_KEYS: u8 = 0;

fn ticked(k: u8) -> bool {
    unsafe { (KEYS & k) != 0 && (PREV_KEYS & k) == 0 }
}
fn pressed(k: u8) -> bool {
    unsafe { (KEYS & k) != 0 }
}

fn wait_event(mut mode: u8) -> ! {
    loop {
        let last_y = draw_screen(mode) + FIRST_Y;
        let mut y = FIRST_Y;
        unsafe { gotoxy(ARROW_X, y); setchar(ARROW_CHAR) };
        show_register_channel(mode);

        loop {
            if ticked(J_UP) {
                unsafe { gotoxy(ARROW_X, y); setchar(SPACE_CHAR) };
                y = if y <= FIRST_Y { last_y } else { y - 1 };
                unsafe { gotoxy(ARROW_X, y); setchar(ARROW_CHAR) };
            } else if ticked(J_DOWN) {
                unsafe { gotoxy(ARROW_X, y); setchar(SPACE_CHAR) };
                y = if y >= last_y { FIRST_Y } else { y + 1 };
                unsafe { gotoxy(ARROW_X, y); setchar(ARROW_CHAR) };
            } else if ticked(J_LEFT) {
                let line = y - FIRST_Y;
                let mut l = unsafe { sr!().current_value(mode, line) };
                if l != 0 {
                    if pressed(J_A) && pressed(J_B) {
                        l = 0;
                    } else if pressed(J_A) {
                        l = l.saturating_sub(10);
                    } else if pressed(J_B) {
                        l = l.saturating_sub(100);
                    } else {
                        l -= 1;
                    }
                    unsafe { sr!().update_value(mode, line, l) };
                }
                unsafe { gotoxy(VAL_X, y) };
                print(b"    \0");
                unsafe { gotoxy(VAL_X, y) };
                println(l, 10);
                show_register_channel(mode);
            } else if ticked(J_RIGHT) {
                let line = y - FIRST_Y;
                let mut l = unsafe { sr!().current_value(mode, line) };
                let m = params(mode)[(line + 1) as usize].1;
                if l != m {
                    if pressed(J_A) && pressed(J_B) {
                        l = m;
                    } else if pressed(J_A) {
                        l = (l + 10).min(m);
                    } else if pressed(J_B) {
                        l = (l + 100).min(m);
                    } else {
                        l += 1;
                    }
                    unsafe { sr!().update_value(mode, line, l) };
                }
                unsafe { gotoxy(VAL_X, y) };
                print(b"    \0");
                unsafe { gotoxy(VAL_X, y) };
                println(l, 10);
                show_register_channel(mode);
            } else if ticked(J_START) {
                if pressed(J_A) {
                    play_music(mode);
                } else {
                    unsafe { sr!().update_value(mode, PLAY, 1) };
                }
            } else if pressed(J_SELECT) {
                if pressed(J_A) {
                    dump_registers();
                } else {
                    mode = (mode + 1) % NB_MODES;
                }
                unsafe {
                    waitpadup();
                    KEYS = 0;
                }
                break;
            }
            unsafe {
                vsync();
                PREV_KEYS = KEYS;
                KEYS = joypad();
            }
        }
    }
}

#[gb_rt::entry]
fn main() -> ! {
    unsafe {
        gbdk_sys::init();

        // Sound is off by default (to save battery); turn it on first.
        NR52_REG.write(0x80);

        // Channel 3 wave RAM (0xFF30..0xFF40): alternating 00/FF like CGB.
        for c in 0..16u8 {
            (0xFF30 as *mut u8).add(c as usize).write_volatile(if c & 1 == 1 { 0x00 } else { 0xFF });
        }

        let r = &mut sr!();
        NR10_REG.write(r.nr10()); NR11_REG.write(r.nr11()); NR12_REG.write(r.nr12());
        NR13_REG.write(r.nr13()); NR14_REG.write(r.nr14());
        NR21_REG.write(r.nr21()); NR22_REG.write(r.nr22()); NR23_REG.write(r.nr23()); NR24_REG.write(r.nr24());
        NR30_REG.write(r.nr30()); NR31_REG.write(r.nr31()); NR32_REG.write(r.nr32());
        NR33_REG.write(r.nr33()); NR34_REG.write(r.nr34());
        NR41_REG.write(r.nr41()); NR42_REG.write(r.nr42()); NR43_REG.write(r.nr43()); NR44_REG.write(r.nr44());
        NR50_REG.write(r.nr50()); NR51_REG.write(r.nr51()); NR52_REG.write(r.nr52());
    }

    clss();
    wait_event(1);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
