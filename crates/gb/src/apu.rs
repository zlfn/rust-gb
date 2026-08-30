//! This module drives the four sound channels.
//!
//! Each channel has a configuration type: [`Pulse`] for channels 1 and 2,
//! [`Wave`] for channel 3, [`Noise`] for channel 4. Building one is a chain of
//! const methods, and `play` writes it out and starts the channel.
//!
//! ```ignore
//! const JUMP: Pulse = Pulse::new(12).note(Note::C, 5).length(8);
//!
//! JUMP.play_ch2();
//! ```
//!
//! A sound played from one place belongs in a constant. A set of them played
//! from the same place belongs in a table.
//!
//! See <https://gbdev.io/pandocs/Audio.html>.
//!
//! # Power
//!
//! The boot ROM leaves the APU on at full volume, so a program that only wants
//! sound has nothing to set up. It does leave channels 3 and 4 reaching the
//! left output alone, which [`set_panning`] settles.
//!
//! [`power_off`] draws less current and is worth having on a pause screen;
//! [`power_on`] comes back from it.
//!
//! # Writing to `DIV`
//!
//! Envelopes, length timers and the channel 1 sweep are all counted off `DIV`,
//! so [`timer::reset_divider`](crate::timer::reset_divider) steps them early.
//! Resetting it while sound is playing is audible.

#[cfg(feature = "cgb")]
use crate::mmio::cgb::{PCM12, PCM34};
use crate::mmio::{
    AudioCtrl, Duty, Envelope, MasterVolume, NR10, NR11, NR12, NR13, NR14, NR21, NR22, NR23, NR24,
    NR30, NR31, NR32, NR33, NR34, NR41, NR42, NR43, NR44, NR50, NR51, NR52, NoiseCtrl, NoiseFreq,
    Panning, PeriodCtrl, PulseLengthDuty, Sweep, WAVE_RAM, WaveDac, WaveLevel, WaveOutput,
};

/// Power the APU on.
///
/// [`power_off`] took the master volume and the panning down with everything
/// else, so [`set_master`] and [`set_panning`] have to follow this or nothing
/// reaches the output.
#[inline]
pub fn power_on() {
    NR52.write(AudioCtrl::new().with_audio_on(true));
}

/// Power the APU off.
///
/// Every audio register is cleared and stays read-only until [`power_on`]. Wave
/// RAM survives. All four DACs go off together, so this clicks unless the
/// channels were quiet already.
#[inline]
pub fn power_off() {
    NR52.write(AudioCtrl::new());
}

/// Set the master volume.
///
/// The output moves as the volume does, so this clicks unless everything is
/// already quiet. Fading a tune out with it is audible on top of the fade.
#[inline]
pub fn set_master(master: MasterVolume) {
    NR50.write(master);
}

/// Set which channels reach which output.
///
/// Taking a channel whose DAC is on off an output, or putting it back, moves
/// that output and clicks. Panning a channel that is playing is heard.
#[inline]
pub fn set_panning(panning: Panning) {
    NR51.write(panning);
}

/// One of the twelve semitones.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Note {
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B,
}

/// The eighth octave in millihertz. Lower ones come from shifting this down,
/// which keeps the division that follows precise.
const OCTAVE_8: [u32; 12] = [
    4_186_009, 4_434_922, 4_698_636, 4_978_032, 5_274_041, 5_587_652, 5_919_911, 6_271_927,
    6_644_875, 7_040_000, 7_458_620, 7_902_133,
];

/// The period value that comes nearest `note` at `octave`, for a channel whose
/// waveform repeats `clock` times a second at a period value of one.
const fn period(note: Note, octave: u8, clock: u32) -> u16 {
    let octave = if octave > 8 { 8 } else { octave };
    let millihertz = OCTAVE_8[note as usize] >> (8 - octave);
    let divider = (clock + millihertz / 2) / millihertz;
    if divider >= 2048 {
        0
    } else {
        (2048 - divider) as u16
    }
}

/// Turn `ticks` of a length timer into the value the register takes, which
/// counts up to `limit` from what is written.
const fn length_from(ticks: u16, limit: u16) -> u8 {
    let ticks = if ticks == 0 {
        1
    } else if ticks > limit {
        limit
    } else {
        ticks
    };
    (limit - ticks) as u8
}

/// A pulse channel's settings.
///
/// Channels 1 and 2 differ only in that 1 has a frequency sweep, so one of
/// these plays on either.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pulse {
    length_duty: PulseLengthDuty,
    envelope: Envelope,
    period_low: u8,
    period_ctrl: PeriodCtrl,
}

impl Pulse {
    /// A 50% square wave at `volume`, out of 15, and the lowest period.
    ///
    /// A volume of 0 is not a quiet sound but a stopped channel: it turns the
    /// DAC off, and a channel whose DAC is off will not start.
    #[inline]
    pub const fn new(volume: u8) -> Self {
        Pulse {
            length_duty: PulseLengthDuty::new().with_duty(Duty::Half),
            envelope: Envelope::new().with_volume(if volume > 15 { 15 } else { volume }),
            period_low: 0,
            period_ctrl: PeriodCtrl::new().with_trigger(true),
        }
    }

    /// Tune to `note` at `octave`.
    ///
    /// C2 through B5 comes out within about five cents. There is nothing below
    /// C2 to reach and the pitch stops there; above B5 the period value runs
    /// out of resolution and drifts, a quarter tone off by the eighth octave.
    #[inline]
    pub const fn note(self, note: Note, octave: u8) -> Self {
        self.at_period(period(note, octave, 131_072_000))
    }

    /// Tune to a period value, which counts up rather than down: the larger it
    /// is, the higher the pitch. At most `$7FF`.
    #[inline]
    pub const fn at_period(self, period: u16) -> Self {
        let period = if period > 0x7FF { 0x7FF } else { period };
        Pulse {
            period_low: period as u8,
            period_ctrl: self.period_ctrl.with_period_high((period >> 8) as u8 & 7),
            ..self
        }
    }

    /// Set the fraction of each cycle the wave spends high.
    #[inline]
    pub const fn duty(self, duty: Duty) -> Self {
        Pulse {
            length_duty: self.length_duty.with_duty(duty),
            ..self
        }
    }

    /// Step the volume every `pace` ticks of a 64 Hz count, up or down.
    ///
    /// A pace of 0 holds the volume where [`Pulse::new`] put it.
    #[inline]
    pub const fn envelope(self, pace: u8, increase: bool) -> Self {
        Pulse {
            envelope: self
                .envelope
                .with_pace(if pace > 7 { 7 } else { pace })
                .with_increase(increase),
            ..self
        }
    }

    /// Stop the channel after `ticks` of a 256 Hz count, at most 64.
    #[inline]
    pub const fn length(self, ticks: u8) -> Self {
        Pulse {
            length_duty: self.length_duty.with_length(length_from(ticks as u16, 64)),
            period_ctrl: self.period_ctrl.with_length_enable(true),
            ..self
        }
    }

    /// Play on channel 1, with `sweep` bending the pitch as it goes.
    ///
    /// The sweep is asked for because it belongs to the channel rather than to
    /// these settings, and one left over from an earlier sound would bend this
    /// one. [`Sweep::new`] leaves the pitch alone.
    ///
    /// Whether a sweep runs at all is settled here and nowhere else: a note
    /// triggered without one cannot be given one part way through.
    ///
    /// A rising sweep is checked against the top as the note is triggered, so a
    /// step that would carry a high note past `$7FF` in one move cuts it before
    /// anything is heard. The smaller the step number, the larger the move. A
    /// falling sweep cannot reach the top and is never cut.
    #[inline]
    pub fn play_ch1(&self, sweep: Sweep) {
        NR10.write(sweep);
        NR11.write(self.length_duty);
        NR12.write(self.envelope);
        NR13.write(self.period_low);
        NR14.write(self.period_ctrl);
    }

    /// Play on channel 2.
    #[inline]
    pub fn play_ch2(&self) {
        NR21.write(self.length_duty);
        NR22.write(self.envelope);
        NR23.write(self.period_low);
        NR24.write(self.period_ctrl);
    }

    /// Change the pitch on channel 1 without starting the note over.
    ///
    /// A running sweep holds its own copy of the pitch and writes that back at
    /// its next step, so this reaches a channel playing without one.
    #[inline]
    pub fn retune_ch1(&self) {
        NR13.write(self.period_low);
        NR14.write(self.period_ctrl.with_trigger(false));
    }

    /// Change the pitch on channel 2 without starting the note over.
    ///
    /// The envelope and the waveform carry on from where they are, which is
    /// what a slide or a vibrato wants. [`Pulse::play_ch2`] restarts both.
    #[inline]
    pub fn retune_ch2(&self) {
        NR23.write(self.period_low);
        NR24.write(self.period_ctrl.with_trigger(false));
    }

    /// Change the duty on channel 1 without starting the note over.
    ///
    /// The length timer shares the register and starts over with it.
    #[inline]
    pub fn set_duty_ch1(&self) {
        NR11.write(self.length_duty);
    }

    /// Change the duty on channel 2 without starting the note over.
    ///
    /// The length timer shares the register and starts over with it.
    #[inline]
    pub fn set_duty_ch2(&self) {
        NR21.write(self.length_duty);
    }
}

/// Channel 3's settings, which play whatever [`load_wave`] last put in wave RAM.
///
/// There is no envelope here, and the volume is the four steps of [`WaveLevel`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Wave {
    length: u8,
    output: WaveOutput,
    period_low: u8,
    period_ctrl: PeriodCtrl,
}

impl Wave {
    /// The wave at `level`, at the lowest period.
    #[inline]
    pub const fn new(level: WaveLevel) -> Self {
        Wave {
            length: 0,
            output: WaveOutput::new().with_level(level),
            period_low: 0,
            period_ctrl: PeriodCtrl::new().with_trigger(true),
        }
    }

    /// Tune to `note` at `octave`.
    ///
    /// This channel reads its waveform half as fast as a pulse channel, so its
    /// range sits an octave below theirs: C1 through B4 within about five
    /// cents, nothing lower to reach, and the same drift above.
    #[inline]
    pub const fn note(self, note: Note, octave: u8) -> Self {
        self.at_period(period(note, octave, 65_536_000))
    }

    /// Tune to a period value, which counts up rather than down: the larger it
    /// is, the higher the pitch. At most `$7FF`.
    #[inline]
    pub const fn at_period(self, period: u16) -> Self {
        let period = if period > 0x7FF { 0x7FF } else { period };
        Wave {
            period_low: period as u8,
            period_ctrl: self.period_ctrl.with_period_high((period >> 8) as u8 & 7),
            ..self
        }
    }

    /// Stop the channel after `ticks` of a 256 Hz count, at most 256.
    #[inline]
    pub const fn length(self, ticks: u16) -> Self {
        Wave {
            length: length_from(ticks, 256),
            period_ctrl: self.period_ctrl.with_length_enable(true),
            ..self
        }
    }

    /// Play on channel 3.
    ///
    /// Triggering the channel while it is already playing can corrupt wave RAM
    /// on an original Game Boy. [`stop`] first avoids that and costs a click.
    #[inline]
    pub fn play(&self) {
        NR30.write(WaveDac::new().with_dac_on(true));
        NR31.write(self.length);
        NR32.write(self.output);
        NR33.write(self.period_low);
        NR34.write(self.period_ctrl);
    }

    /// Change the pitch without starting the waveform over.
    #[inline]
    pub fn retune(&self) {
        NR33.write(self.period_low);
        NR34.write(self.period_ctrl.with_trigger(false));
    }

    /// Change the output level without starting the waveform over.
    ///
    /// Channel 3 has no envelope, so this is how its volume moves during a
    /// note.
    #[inline]
    pub fn set_level(&self) {
        NR32.write(self.output);
    }
}

/// Load the 32 four-bit samples channel 3 plays, high nibble of each byte first.
///
/// Channel 3 stops first, because wave RAM reached while it is playing does not
/// answer for the address asked for. That means a click if it was playing, so
/// this belongs in a quiet moment. Play a [`Wave`] to start it again.
#[inline]
pub fn load_wave(samples: &[u8; 16]) {
    NR30.write(WaveDac::new());
    let mut i = 0;
    while i < 16 {
        WAVE_RAM.index(i).write(samples[i]);
        i += 1;
    }
}

/// Channel 4's settings.
///
/// The pitch is a clock divider and shift rather than a note, since what comes
/// out is noise and is chosen by ear.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Noise {
    length: u8,
    envelope: Envelope,
    freq: NoiseFreq,
    ctrl: NoiseCtrl,
}

impl Noise {
    /// Noise at `volume`, out of 15, clocked at `262144 / (divider << shift)`
    /// hertz, where a divider of 0 counts as a half.
    ///
    /// A volume of 0 is not a quiet sound but a stopped channel: it turns the
    /// DAC off, and a channel whose DAC is off will not start.
    ///
    /// A shift of 14 or 15 stops the clock, and neither is reachable here.
    #[inline]
    pub const fn new(volume: u8, divider: u8, shift: u8) -> Self {
        Noise {
            length: 0,
            envelope: Envelope::new().with_volume(if volume > 15 { 15 } else { volume }),
            freq: NoiseFreq::new()
                .with_divider(if divider > 7 { 7 } else { divider })
                .with_shift(if shift > 13 { 13 } else { shift }),
            ctrl: NoiseCtrl::new().with_trigger(true),
        }
    }

    /// Run the shift register seven bits wide, which repeats often enough to
    /// carry a pitch.
    #[inline]
    pub const fn short_lfsr(self) -> Self {
        Noise {
            freq: self.freq.with_short_lfsr(true),
            ..self
        }
    }

    /// Step the volume every `pace` ticks of a 64 Hz count, up or down.
    ///
    /// A pace of 0 holds the volume where [`Noise::new`] put it.
    #[inline]
    pub const fn envelope(self, pace: u8, increase: bool) -> Self {
        Noise {
            envelope: self
                .envelope
                .with_pace(if pace > 7 { 7 } else { pace })
                .with_increase(increase),
            ..self
        }
    }

    /// Stop the channel after `ticks` of a 256 Hz count, at most 64.
    #[inline]
    pub const fn length(self, ticks: u8) -> Self {
        Noise {
            length: length_from(ticks as u16, 64),
            ctrl: self.ctrl.with_length_enable(true),
            ..self
        }
    }

    /// Play on channel 4.
    #[inline]
    pub fn play(&self) {
        NR41.write(self.length);
        NR42.write(self.envelope);
        NR43.write(self.freq);
        NR44.write(self.ctrl);
    }
}

/// One of the four sound channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    /// Pulse, with a frequency sweep.
    One,
    /// Pulse.
    Two,
    /// Wave.
    Three,
    /// Noise.
    Four,
}

/// Take a channel down to nothing without stopping it.
///
/// The DAC stays on, which is the difference from [`stop`]: switching one off
/// moves the output enough to click.
///
/// On channels 1 and 2 the pitch goes down with the volume, so playing again is
/// how one of those comes back.
#[inline]
pub fn silence(ch: Channel) {
    // Volume 0 rising, rather than a plain 0, is what keeps the DAC on. The
    // channel then has to be triggered for the new volume to reach it.
    const QUIET: Envelope = Envelope::new().with_increase(true);
    const RETRIGGER: PeriodCtrl = PeriodCtrl::new().with_trigger(true);
    match ch {
        Channel::One => {
            NR12.write(QUIET);
            NR14.write(RETRIGGER);
        }
        Channel::Two => {
            NR22.write(QUIET);
            NR24.write(RETRIGGER);
        }
        // Channel 3 has an output level instead of an envelope, and it takes
        // effect where it is written.
        Channel::Three => NR32.write(WaveOutput::new().with_level(WaveLevel::Mute)),
        Channel::Four => {
            NR42.write(QUIET);
            NR44.write(NoiseCtrl::new().with_trigger(true));
        }
    }
}

/// Stop a channel by turning its DAC off.
///
/// Clicks, so [`silence`] is the one to reach for between notes.
#[inline]
pub fn stop(ch: Channel) {
    match ch {
        Channel::One => NR12.write(Envelope::new()),
        Channel::Two => NR22.write(Envelope::new()),
        Channel::Three => NR30.write(WaveDac::new()),
        Channel::Four => NR42.write(Envelope::new()),
    }
}

/// Whether a channel is still running.
///
/// A channel stops when a length timer set by [`Pulse::length`] and its
/// counterparts expires, when its DAC goes off, or, on channel 1, when the
/// sweep carries the pitch past the top.
#[inline]
pub fn playing(ch: Channel) -> bool {
    let status = NR52.read();
    match ch {
        Channel::One => status.ch1_on(),
        Channel::Two => status.ch2_on(),
        Channel::Three => status.ch3_on(),
        Channel::Four => status.ch4_on(),
    }
}

/// What a channel is putting out, from 0 to 15.
///
/// This is the digital value on its way to the DAC, so it moves with the
/// envelope and the waveform rather than with the volume the mixer applies. A
/// program drawing its own sound reads it here.
#[cfg(feature = "cgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "cgb")))]
#[inline]
pub fn output(ch: Channel) -> u8 {
    match ch {
        Channel::One => PCM12.read().low(),
        Channel::Two => PCM12.read().high(),
        Channel::Three => PCM34.read().low(),
        Channel::Four => PCM34.read().high(),
    }
}
