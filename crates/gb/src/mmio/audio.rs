//! Audio (`NR10`..`NR52`, wave RAM).

use bitfield_struct::{bitenum, bitfield};
use voladdress::{Safe, VolAddress, VolBlock};

/// Square-wave duty cycle (`NR11` / `NR21` bits 7-6).
#[bitenum(all = false)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Duty {
    /// 12.5%.
    Eighth = 0,
    /// 25%.
    Quarter = 1,
    /// 50%.
    Half = 2,
    /// 75% (audibly identical to 25%).
    #[fallback]
    ThreeQuarters = 3,
}

/// Wave-channel output level (`NR32` bits 6-5).
#[bitenum(all = false)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveLevel {
    /// Muted.
    Mute = 0,
    /// 100%, samples used as-is.
    Full = 1,
    /// 50%, samples shifted right once.
    Half = 2,
    /// 25%, samples shifted right twice.
    #[fallback]
    Quarter = 3,
}

/// Channel 1 frequency sweep (`NR10`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7   | —          |     | Unused. |
/// | 6-4 | `pace`     | R/W | An iteration every `pace` 128 Hz ticks; 0 disables the sweep. |
/// | 3   | `subtract` | R/W | Period direction: clear adds, set subtracts. |
/// | 2-0 | `step`     | R/W | Shift applied to the period each iteration. |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct Sweep {
    /// Step: shift applied to the period each iteration.
    #[bits(3)]
    pub step: u8,
    /// Direction: `false` adds to the period, `true` subtracts.
    pub subtract: bool,
    /// Pace: an iteration every `pace` 128 Hz ticks (0 disables sweep).
    #[bits(3)]
    pub pace: u8,
    #[bits(1)]
    __: u8,
}

/// Length timer and duty cycle (`NR11` / `NR21`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7-6 | `duty`   | R/W | Output waveform duty cycle. |
/// | 5-0 | `length` | W   | Initial length timer; higher means a shorter time to cut. |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct PulseLengthDuty {
    /// Initial length timer (write-only): higher means a shorter time to cut.
    #[bits(6, access = WO)]
    pub length: u8,
    /// Output waveform duty cycle.
    #[bits(2)]
    pub duty: Duty,
}

/// Volume and envelope (`NR12` / `NR22` / `NR42`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7-4 | `volume`   | R/W | Initial volume. |
/// | 3   | `increase` | R/W | Direction: set raises volume over time, clear lowers it. |
/// | 2-0 | `pace`     | R/W | A step every `pace` 64 Hz ticks; 0 disables the envelope. |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct Envelope {
    /// Envelope pace: a step every `pace` 64 Hz ticks (0 disables the envelope).
    #[bits(3)]
    pub pace: u8,
    /// Direction: `true` increases volume over time, `false` decreases.
    pub increase: bool,
    /// Initial volume.
    #[bits(4)]
    pub volume: u8,
}

/// Period high bits and channel control (`NR14` / `NR24` / `NR34`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7   | `trigger`       | W   | (Re)start the channel. |
/// | 6   | `length_enable` | R/W | Stop the channel when the length timer expires. |
/// | 5-3 | —               |     | Unused. |
/// | 2-0 | `period_high`   | W   | Upper 3 bits of the 11-bit period (low 8 in `NRx3`). |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct PeriodCtrl {
    /// Upper 3 bits of the 11-bit period (low 8 bits live in `NRx3`).
    #[bits(3, access = WO)]
    pub period_high: u8,
    #[bits(3)]
    __: u8,
    /// Stop the channel when the length timer expires.
    pub length_enable: bool,
    /// Trigger (write-only): (re)start the channel.
    #[bits(1, access = WO)]
    pub trigger: bool,
}

/// Channel 3 DAC power (`NR30`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7   | `dac_on` | R/W | DAC power; turning it off also turns the channel off. |
/// | 6-0 | —        |     | Unused. |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct WaveDac {
    #[bits(7)]
    __: u8,
    /// DAC power. Turning it off also turns the channel off.
    pub dac_on: bool,
}

/// Channel 3 output level (`NR32`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7   | —       |     | Unused. |
/// | 6-5 | `level` | R/W | Coarse output volume. |
/// | 4-0 | —       |     | Unused. |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct WaveOutput {
    #[bits(5)]
    __: u8,
    /// Coarse output volume.
    #[bits(2)]
    pub level: WaveLevel,
    #[bits(1)]
    __: u8,
}

/// Channel 4 frequency and randomness (`NR43`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7-4 | `shift`      | R/W | Clock shift. |
/// | 3   | `short_lfsr` | R/W | LFSR width: set = 7-bit (more tonal), clear = 15-bit. |
/// | 2-0 | `divider`    | R/W | Clock divider. |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct NoiseFreq {
    /// Clock divider.
    #[bits(3)]
    pub divider: u8,
    /// LFSR width: `false` = 15-bit, `true` = 7-bit (more tonal).
    pub short_lfsr: bool,
    /// Clock shift.
    #[bits(4)]
    pub shift: u8,
}

/// Channel 4 control (`NR44`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7   | `trigger`       | W   | (Re)start the channel. |
/// | 6   | `length_enable` | R/W | Stop the channel when the length timer expires. |
/// | 5-0 | —               |     | Unused. |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct NoiseCtrl {
    #[bits(6)]
    __: u8,
    /// Stop the channel when the length timer expires.
    pub length_enable: bool,
    /// Trigger (write-only): (re)start the channel.
    #[bits(1, access = WO)]
    pub trigger: bool,
}

/// Master volume and VIN panning (`NR50`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7   | `vin_left`     | R/W | Mix external VIN into the left output. |
/// | 6-4 | `left_volume`  | R/W | Left output volume (0 treated as 1, 7 as 8). |
/// | 3   | `vin_right`    | R/W | Mix external VIN into the right output. |
/// | 2-0 | `right_volume` | R/W | Right output volume (0 treated as 1, 7 as 8). |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct MasterVolume {
    /// Right output volume (0 is treated as 1, 7 as 8).
    #[bits(3)]
    pub right_volume: u8,
    /// Mix external VIN into the right output.
    pub vin_right: bool,
    /// Left output volume (0 is treated as 1, 7 as 8).
    #[bits(3)]
    pub left_volume: u8,
    /// Mix external VIN into the left output.
    pub vin_left: bool,
}

/// Sound panning (`NR51`): which channels reach each output.
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7   | `ch4_left`  | R/W | Channel 4 (noise) reaches the left output. |
/// | 6   | `ch3_left`  | R/W | Channel 3 (wave) reaches the left output. |
/// | 5   | `ch2_left`  | R/W | Channel 2 (pulse) reaches the left output. |
/// | 4   | `ch1_left`  | R/W | Channel 1 (pulse) reaches the left output. |
/// | 3   | `ch4_right` | R/W | Channel 4 (noise) reaches the right output. |
/// | 2   | `ch3_right` | R/W | Channel 3 (wave) reaches the right output. |
/// | 1   | `ch2_right` | R/W | Channel 2 (pulse) reaches the right output. |
/// | 0   | `ch1_right` | R/W | Channel 1 (pulse) reaches the right output. |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct Panning {
    /// Channel 1 (pulse) reaches the right output.
    pub ch1_right: bool,
    /// Channel 2 (pulse) reaches the right output.
    pub ch2_right: bool,
    /// Channel 3 (wave) reaches the right output.
    pub ch3_right: bool,
    /// Channel 4 (noise) reaches the right output.
    pub ch4_right: bool,
    /// Channel 1 (pulse) reaches the left output.
    pub ch1_left: bool,
    /// Channel 2 (pulse) reaches the left output.
    pub ch2_left: bool,
    /// Channel 3 (wave) reaches the left output.
    pub ch3_left: bool,
    /// Channel 4 (noise) reaches the left output.
    pub ch4_left: bool,
}

/// Audio master control (`NR52`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7   | `audio_on` | R/W | Master audio power; off clears and locks the other registers. |
/// | 6-4 | —          |     | Unused. |
/// | 3   | `ch4_on`   | RO  | Channel 4 (noise) is running. |
/// | 2   | `ch3_on`   | RO  | Channel 3 (wave) is running. |
/// | 1   | `ch2_on`   | RO  | Channel 2 (pulse) is running. |
/// | 0   | `ch1_on`   | RO  | Channel 1 (pulse) is running. |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct AudioCtrl {
    /// Channel 1 (pulse) is running (read-only status).
    #[bits(1, access = RO)]
    pub ch1_on: bool,
    /// Channel 2 (pulse) is running (read-only status).
    #[bits(1, access = RO)]
    pub ch2_on: bool,
    /// Channel 3 (wave) is running (read-only status).
    #[bits(1, access = RO)]
    pub ch3_on: bool,
    /// Channel 4 (noise) is running (read-only status).
    #[bits(1, access = RO)]
    pub ch4_on: bool,
    #[bits(3)]
    __: u8,
    /// Master audio power. Turning it off clears and locks the other registers.
    pub audio_on: bool,
}

// ── Channel 1 (square + sweep) ──────────────────────────────────────────────

/// Channel 1 sweep.
pub const NR10: VolAddress<Sweep, Safe, Safe> = unsafe { VolAddress::new(0xFF10) };
/// Channel 1 length timer and duty cycle.
pub const NR11: VolAddress<PulseLengthDuty, Safe, Safe> = unsafe { VolAddress::new(0xFF11) };
/// Channel 1 volume and envelope.
pub const NR12: VolAddress<Envelope, Safe, Safe> = unsafe { VolAddress::new(0xFF12) };
/// Channel 1 period low byte (write-only).
pub const NR13: VolAddress<u8, (), Safe> = unsafe { VolAddress::new(0xFF13) };
/// Channel 1 period high byte and control.
pub const NR14: VolAddress<PeriodCtrl, Safe, Safe> = unsafe { VolAddress::new(0xFF14) };

// ── Channel 2 (square) ──────────────────────────────────────────────────────

/// Channel 2 length timer and duty cycle.
pub const NR21: VolAddress<PulseLengthDuty, Safe, Safe> = unsafe { VolAddress::new(0xFF16) };
/// Channel 2 volume and envelope.
pub const NR22: VolAddress<Envelope, Safe, Safe> = unsafe { VolAddress::new(0xFF17) };
/// Channel 2 period low byte (write-only).
pub const NR23: VolAddress<u8, (), Safe> = unsafe { VolAddress::new(0xFF18) };
/// Channel 2 period high byte and control.
pub const NR24: VolAddress<PeriodCtrl, Safe, Safe> = unsafe { VolAddress::new(0xFF19) };

// ── Channel 3 (wave) ────────────────────────────────────────────────────────

/// Channel 3 DAC power.
pub const NR30: VolAddress<WaveDac, Safe, Safe> = unsafe { VolAddress::new(0xFF1A) };
/// Channel 3 length timer (write-only).
pub const NR31: VolAddress<u8, (), Safe> = unsafe { VolAddress::new(0xFF1B) };
/// Channel 3 output level.
pub const NR32: VolAddress<WaveOutput, Safe, Safe> = unsafe { VolAddress::new(0xFF1C) };
/// Channel 3 period low byte (write-only).
pub const NR33: VolAddress<u8, (), Safe> = unsafe { VolAddress::new(0xFF1D) };
/// Channel 3 period high byte and control.
pub const NR34: VolAddress<PeriodCtrl, Safe, Safe> = unsafe { VolAddress::new(0xFF1E) };

// ── Channel 4 (noise) ───────────────────────────────────────────────────────

/// Channel 4 length timer (write-only).
pub const NR41: VolAddress<u8, (), Safe> = unsafe { VolAddress::new(0xFF20) };
/// Channel 4 volume and envelope.
pub const NR42: VolAddress<Envelope, Safe, Safe> = unsafe { VolAddress::new(0xFF21) };
/// Channel 4 frequency and randomness.
pub const NR43: VolAddress<NoiseFreq, Safe, Safe> = unsafe { VolAddress::new(0xFF22) };
/// Channel 4 control.
pub const NR44: VolAddress<NoiseCtrl, Safe, Safe> = unsafe { VolAddress::new(0xFF23) };

// ── Control ─────────────────────────────────────────────────────────────────

/// Master volume and VIN panning.
pub const NR50: VolAddress<MasterVolume, Safe, Safe> = unsafe { VolAddress::new(0xFF24) };
/// Sound panning: which channels go to which output.
pub const NR51: VolAddress<Panning, Safe, Safe> = unsafe { VolAddress::new(0xFF25) };
/// Audio master control: power and per-channel status.
pub const NR52: VolAddress<AudioCtrl, Safe, Safe> = unsafe { VolAddress::new(0xFF26) };

/// Wave pattern RAM: 16 bytes of channel 3 sample data at `0xFF30`.
pub const WAVE_RAM: VolBlock<u8, Safe, Safe, 16> = unsafe { VolBlock::new(0xFF30) };

const _: () = {
    assert!(Sweep::new().with_pace(0x7).into_bits() == 0b0111_0000);
    assert!(PulseLengthDuty::new().with_duty(Duty::ThreeQuarters).into_bits() == 0b1100_0000);
    assert!(Envelope::new().with_volume(0xF).into_bits() == 0b1111_0000);
    assert!(PeriodCtrl::new().with_trigger(true).into_bits() == 0b1000_0000);
    assert!(PeriodCtrl::new().with_length_enable(true).into_bits() == 0b0100_0000);
    assert!(WaveDac::new().with_dac_on(true).into_bits() == 0b1000_0000);
    assert!(WaveOutput::new().with_level(WaveLevel::Quarter).into_bits() == 0b0110_0000);
    assert!(NoiseFreq::new().with_shift(0xF).into_bits() == 0b1111_0000);
    assert!(MasterVolume::new().with_vin_left(true).into_bits() == 0b1000_0000);
    assert!(Panning::new().with_ch1_right(true).into_bits() == 0b0000_0001);
    assert!(Panning::new().with_ch4_left(true).into_bits() == 0b1000_0000);
    assert!(AudioCtrl::new().with_audio_on(true).into_bits() == 0b1000_0000);
};
