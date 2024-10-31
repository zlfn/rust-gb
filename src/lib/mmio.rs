//! Memory Map IO address for the GameBoy components.
//!
//! Documents are provided by [GB Pan Docs](https://gbdev.io/pandocs/Memory_Map.html)

use voladdress::{Safe, Unsafe, VolAddress, VolBlock};

/// [Joypad](https://gbdev.io/pandocs/Joypad_Input.html#ff00--p1joyp-joypad)
///
/// The eight GameBoy action/direction buttons are arranged as a 2x4 matrix.
/// Select either action or direction buttons by writing to this register, then read out the bits
/// 0-3.
///
/// <table class="bit-descrs"><thead><tr><th></th><th>7</th><th>6</th><th>5</th><th>4</th><th>3</th><th>2</th><th>1</th><th>0</th></tr></thead><tbody><tr><td><strong>P1</strong></td><td colspan="2" class="unused-field"></td><td colspan="1">Select buttons</td><td colspan="1">Select d-pad</td><td colspan="1">Start / Down</td><td colspan="1">Select / Up</td><td colspan="1">B / Left</td><td colspan="1">A / Right</td></tr></tbody></table>
///
/// * **Select buttons**: If this bit is `0`, then buttons (SsBA) can be read from the lower
/// nibble.
/// * **Select d-pad** : If this bit is `0`, then directional keys can be read from the lower
/// nibble.
/// * The lower nibble is *Read-only*. Note that, rather unconventionally for GameBoy, a button
/// being pressed is seen as the corresponding bit being `0`, not `1`.
///
/// If neigher buttons nor d-pad is selected (`$30` was written), then the low nibble reads `$F`
/// (all buttons released).
///
/// If both buttons and d-pad is selected, then the low nibble reads d-pad *or* button is pressed.
///
/// # Safety
/// After writing to [`JOYP`], if [`JOYP`] is read in the following instruction, the
/// unexpected value will be read. Therefore, if you want it to work as intended, you have
/// to give a brief delay.
pub const JOYP: VolAddress<u8, Safe, Unsafe> = 
    unsafe { VolAddress::new(0xFF00) };

/// [Serial transfer data](https://gbdev.io/pandocs/Serial_Data_Transfer_(Link_Cable).html#ff01--sb-serial-transfer-data)
pub const SB: VolAddress<u8, Safe, Safe> =
    unsafe { VolAddress::new(0xFF01) };

/// [Serial transfer control](https://gbdev.io/pandocs/Serial_Data_Transfer_(Link_Cable).html#ff02--sc-serial-transfer-control)
pub const SC: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF02) };

/// [Divider](https://gbdev.io/pandocs/Timer_and_Divider_Registers.html#ff04--div-divider-register)
///
/// This register is incremented at a rate of 16384Hz (~16779Hz on SGB). Writing any value to this
/// register resets it to `$00`. Additionally, this register is reset when executing the `stop`
/// instruction, and only begins ticking again once `stop` mode ends. This also occurs during a
/// [speed switch](https://gbdev.io/pandocs/CGB_Registers.html#ff4d--key1-cgb-mode-only-prepare-speed-switch).
///
/// Note: This divider is affected by CGB double speed mode, and will increment at 32768Hz in
/// double speed.
pub const DIV: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF04) };

/// [Timer counter](https://gbdev.io/pandocs/Timer_and_Divider_Registers.html#ff05--tima-timer-counter)
///
/// This timer is incremented at the clock frequency specified by the [`TAC`] register.
/// When the value overflows, it is reset to the value specified in [`TMA`] and an interrupt is
/// requested.
pub const TIMA: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF05) };

/// [Timer modulo](https://gbdev.io/pandocs/Timer_and_Divider_Registers.html#ff06--tma-timer-modulo)
///
/// When TIMA overflows, it is reset to the value in this register and Timer interrupt is
/// requested. Example of use: if TMA is set to $FF, an interrupt is requested at the clock
/// frequency selected in [`TAC`] (because every increment is an overflow). However, if TMA is set
/// to $FE, an interrupt is only requested every two increments, which effectively divides the
/// selected clock by two. Setting TMA to $FD would divide the clock by three, and so on.
///
/// If a TMA write is executed on the same M-cycle as the content of [`TMA`] is transferred to
/// [`TIMA`] due to a timer overflow, the old value is transferred to [`TIMA`]
pub const TMA: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF06) };

/// [Timer control](https://gbdev.io/pandocs/Timer_and_Divider_Registers.html#ff07--tac-timer-control)
///
/// <table class="bit-descrs"><thead><tr><th></th><th>7</th><th>6</th><th>5</th><th>4</th><th>3</th><th>2</th><th>1</th><th>0</th></tr></thead><tbody><tr><td><strong>TAC</strong></td><td colspan="5" class="unused-field"></td><td colspan="1">Enable</td><td colspan="2">Clock select</td></tr></tbody></table>
///
/// - **Enable**: Controls whether `TIMA` is incremented.
///  Note that `DIV` is **always** counting, regardless of this bit.
/// - **Clock select**: Controls the frequency at which `TIMA` is incremented, as follows:
///  
///  <div class="table-wrapper"><table>
///    <thead>
///     <tr><th rowspan=2>Clock select</th><th rowspan=2>Increment every</th><th colspan=3>Frequency (Hz)</th></tr>
///      <tr><th>DMG, SGB2, CGB in single-speed mode</th><th>SGB1</th><th>CGB in double-speed mode</th></tr>
///    </thead><tbody>
///      <tr><td>00</td><td>256 M-cycles </td><td>  4096</td><td>  ~4194</td><td>  8192</td></tr>
///      <tr><td>01</td><td>4 M-cycles   </td><td>262144</td><td>~268400</td><td>524288</td></tr>
///      <tr><td>10</td><td>16 M-cycles  </td><td> 65536</td><td> ~67110</td><td>131072</td></tr>
///      <tr><td>11</td><td>64 M-cycles  </td><td> 16384</td><td> ~16780</td><td> 32768</td></tr>
///    </tbody>
///  </table></div>
///
/// Note that writing to this register may increase [`TIMA`] once!
/// For more information, please refer [GB Pan Docs](https://gbdev.io/pandocs/Timer_and_Divider_Registers.html#ff07--tac-timer-control)
pub const TAC: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF07) };

/// [Interrupt flag](https://gbdev.io/pandocs/Interrupts.html#ff0f--if-interrupt-flag)
///
/// <table class="bit-descrs"><thead><tr><th></th><th>7</th><th>6</th><th>5</th><th>4</th><th>3</th><th>2</th><th>1</th><th>0</th></tr></thead><tbody><tr><td><strong>IF</strong></td><td colspan="3" class="unused-field"></td><td colspan="1">Joypad</td><td colspan="1">Serial</td><td colspan="1">Timer</td><td colspan="1">LCD</td><td colspan="1">VBlank</td></tr></tbody></table>
///
/// * **VBlank** (Read/Write): Controls whether the VBlank interrupt handler is being requested.
/// * **LCD** (Read/Write) : Controls whether the LCD interrupt handler is being requested.
/// * **Timer** (Read/Write) : Controls whether the Timer interrupt handler is being requested.
/// * **Serial** (Read/Write) : Controls whether the Serial interrupt handler is being requested.
/// * **Joypad** (Read/Write) : Controls whether the Joypad interrupt handler is being requested.
///
///
/// When an interrupt request signal (some internal wire going from the PPU/APU/... to the CPU)
/// changes from low to high, the corresponding bit in the [`IF`] register becomes set. For
/// example, bit 0 becomes set when the PPU enters the VBlank period.
///
/// Any set bits in the [`IF`] register are only **requesting** an interrupt. The actual
/// **execution** of the interrupt handler happnes only if both the `IME` flag and the
/// corresponding bit in the [`IE`] register are set; otherwise the interrupt "waits" until
/// **both** `IME` and [`IE`] allow it to be serviced.
///
/// Since the CPU automatically sets and clears the bits in the [`IF`] register, it is usually not
/// necessary to write to the [`IF`] register. However, the user still do that in order to manually
/// request (or discard) interrupts. Just like real interrupts, a manually requeseted interrupt
/// isn't serviced unless/until `IME` and [`IE`] allow it.
pub const IF: VolAddress<u8, Safe, Safe> =
    unsafe { VolAddress::new(0xFF0F) };

/// [Sound channel 1 sweep](https://gbdev.io/pandocs/Audio_Registers.html#ff10--nr10-channel-1-sweep)
pub const NR10: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF10) };

/// [Sound channel 1 length timer & duty cycle](https://gbdev.io/pandocs/Audio_Registers.html#ff11--nr11-channel-1-length-timer--duty-cycle)
pub const NR11: VolAddress<u8, Unsafe, Safe> = 
    unsafe { VolAddress::new(0xFF11) };

/// [Sound channel 1 volume & envelope](https://gbdev.io/pandocs/Audio_Registers.html#ff12--nr12-channel-1-volume--envelope)
pub const NR12: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF12) };

/// [Sound channel 1 period low](https://gbdev.io/pandocs/Audio_Registers.html#ff13--nr13-channel-1-period-low-write-only)
pub const NR13: VolAddress<u8, (), Safe> = 
    unsafe { VolAddress::new(0xFF13) };

/// [Sound channel 1 period high & control](https://gbdev.io/pandocs/Audio_Registers.html#ff14--nr14-channel-1-period-high--control)
pub const NR14: VolAddress<u8, Unsafe, Safe> = 
    unsafe { VolAddress::new(0xFF14) };

/// [Sound channel 2 length timer & duty cycle](https://gbdev.io/pandocs/Audio_Registers.html#sound-channel-2--pulse)
pub const NR21: VolAddress<u8, Unsafe, Safe> = 
    unsafe { VolAddress::new(0xFF16) };

/// [Sound channel 2 volume & envelope](https://gbdev.io/pandocs/Audio_Registers.html#sound-channel-2--pulse)
pub const NR22: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF17) };

/// [Sound channel 2 period low](https://gbdev.io/pandocs/Audio_Registers.html#sound-channel-2--pulse)
pub const NR23: VolAddress<u8, (), Safe> = 
    unsafe { VolAddress::new(0xFF18) };

/// [Sound channel 2 period high & control](https://gbdev.io/pandocs/Audio_Registers.html#sound-channel-2--pulse)
pub const NR24: VolAddress<u8, Unsafe, Safe> =
    unsafe { VolAddress::new(0xFF19) };

/// [Sound channel 3 DAC enable](https://gbdev.io/pandocs/Audio_Registers.html#ff1a--nr30-channel-3-dac-enable)
pub const NR30: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF1A) };

/// [Sound channel 3 length timer](https://gbdev.io/pandocs/Audio_Registers.html#ff1b--nr31-channel-3-length-timer-write-only)
pub const NR31: VolAddress<u8, (), Safe> = 
    unsafe { VolAddress::new(0xFF1B) };

/// [Sound channel 3 output level](https://gbdev.io/pandocs/Audio_Registers.html#ff1c--nr32-channel-3-output-level)
pub const NR32: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF1C) };

/// [Sound channel 3 period low](https://gbdev.io/pandocs/Audio_Registers.html#ff1d--nr33-channel-3-period-low-write-only)
pub const NR33: VolAddress<u8, (), Safe> =
    unsafe { VolAddress::new(0xFF1D) };

/// [Sound channel 3 period high & control](https://gbdev.io/pandocs/Audio_Registers.html#ff1e--nr34-channel-3-period-high--control)
pub const NR34: VolAddress<u8, Unsafe, Safe> = 
    unsafe { VolAddress::new(0xFF1E) };

/// [Sound channel 4 length timer](https://gbdev.io/pandocs/Audio_Registers.html#ff20--nr41-channel-4-length-timer-write-only)
pub const NR41: VolAddress<u8, (), Safe> = 
    unsafe { VolAddress::new(0xFF20) };

/// [Master volume & VIN panning](https://gbdev.io/pandocs/Audio_Registers.html#ff24--nr50-master-volume--vin-panning)
pub const NR50: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF24) };

/// [Sound panning](https://gbdev.io/pandocs/Audio_Registers.html#ff25--nr51-sound-panning)
pub const NR51: VolAddress<u8, Safe, Safe> =
    unsafe { VolAddress::new(0xFF25) };

/// [Sound on/off](https://gbdev.io/pandocs/Audio_Registers.html#ff26--nr52-audio-master-control)
pub const NR52: VolAddress<u8, Safe, Safe> =
    unsafe { VolAddress::new(0xFF26) };

/// [Storage for one of the sound channel's waveform](https://gbdev.io/pandocs/Audio_Registers.html#ff30ff3f--wave-pattern-ram)
///
/// Wave RAM is 16 bytes long; each byte holds two "samples", each 4 bits.
///
/// As CH3 plays, it reads wave RAM left to right, upper nibble first. That is, `$FF30`'s upper
/// nibble, `$FF30`'s lower nibble, `$FF31`'s upper nibble, and so on.
///
/// # Warning
/// When CH3 is started, the first sample read is the one at index 1, i.e. the lower nibble of the
/// first byte, NOT the upper nibble.
///
/// # Safety
/// Accessing wave RAM while CH3 is **active** (i.e. playing) causes accesses to misbehave:
/// * On AGB, reads return `$FF`, and writes are ignored.
/// * On monochrome consoles, wave RAM can only be accessed on the same cycle that CH3 does.
/// Otherwise, reads return `$FF`, and writes are ignored.
/// * On other consoles, the byte accessed will be the on CH3 is currently reading; that is, if Ch3
/// is currently reading one of the first two samples, the CPU will really access `$FF30`,
/// regardless of the address being used.
///
/// Wave RAM can be accessed normally even if the DAC is on, as long as the channel is not active.
/// This is especially relevant on GBA whose mixer behaves as if DACs are always enabled.
///
/// The way it works is that wave RAM is a 16-byte memory buffer, and whie it's playing, CH3 has
/// priority over the CPU when choosing which of those 16 bytes is accessed. So, from the CPU's
/// point of view, wave RAM reads out the same byte, reagardless of the address.
pub const WAVE_RAM: VolBlock<u8, Unsafe, Unsafe, 16> = 
    unsafe { VolBlock::new(0xFF30) };

/// [LCD control](https://gbdev.io/pandocs/LCDC.html#ff40--lcdc-lcd-control)
pub const LCDC: VolAddress<u8, Safe, Unsafe> = 
    unsafe { VolAddress::new(0xFF40) };

/// [LCD status](https://gbdev.io/pandocs/STAT.html#ff41--stat-lcd-status)
pub const STAT: VolAddress<u8, Safe, Unsafe> = 
    unsafe { VolAddress::new(0xFF41) };

/// [Viewport Y position](https://gbdev.io/pandocs/Scrolling.html#ff42ff43--scy-scx-background-viewport-y-position-x-position)
pub const SCY: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF42) };

/// [Viewport X position](https://gbdev.io/pandocs/Scrolling.html#ff42ff43--scy-scx-background-viewport-y-position-x-position)
pub const SCX: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF43) };

/// [LCD Y coordinate](https://gbdev.io/pandocs/STAT.html#ff44--ly-lcd-y-coordinate-read-only)
pub const LY: VolAddress<u8, Safe, ()> = 
    unsafe { VolAddress::new(0xFF44) };

/// [LY compare](https://gbdev.io/pandocs/STAT.html#ff45--lyc-ly-compare)
pub const LYC: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF45) };

/// [OAM DMA source address & start](https://gbdev.io/pandocs/OAM_DMA_Transfer.html#ff46--dma-oam-dma-source-address--start)
pub const DMA: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF46) };

/// [BG palette data](https://gbdev.io/pandocs/Palettes.html#ff47--bgp-non-cgb-mode-only-bg-palette-data)
pub const BGP: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF47) };

/// [OBJ palette 0 data](https://gbdev.io/pandocs/Palettes.html#ff48ff49--obp0-obp1-non-cgb-mode-only-obj-palette-0-1-data)
pub const OBP0: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF48) };

/// [OBJ palette 1 data](https://gbdev.io/pandocs/Palettes.html#ff48ff49--obp0-obp1-non-cgb-mode-only-obj-palette-0-1-data)
pub const OBP1: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF49) };

/// [Window Y position](https://gbdev.io/pandocs/Scrolling.html#ff4aff4b--wy-wx-window-y-position-x-position-plus-7)
pub const WY: VolAddress<u8, Safe, Unsafe> = 
    unsafe { VolAddress::new(0xFF4A) };

/// [Window X position plus 7](https://gbdev.io/pandocs/Scrolling.html#ff4aff4b--wy-wx-window-y-position-x-position-plus-7)
pub const WX: VolAddress<u8, Safe, Unsafe> = 
    unsafe { VolAddress::new(0xFF4B) };

/// [Prepare speed switch](https://gbdev.io/pandocs/CGB_Registers.html#ff4d--key1-cgb-mode-only-prepare-speed-switch)
#[cfg(any(feature="color", doc))]
#[doc(cfg(feature="color"))]
pub const KEY1: VolAddress<u8, Safe, Unsafe> =
    unsafe { VolAddress::new(0xFF4D) };

/// [VRAM bank](https://gbdev.io/pandocs/CGB_Registers.html#ff4f--vbk-cgb-mode-only-vram-bank)
///
/// This register can be written to change VRAM banks. Only bit 0 matters, all other bits are
/// ignored.
#[cfg(any(feature="color", doc))]
#[doc(cfg(feature="color"))]
pub const VBK: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF4F) };

/// [VRAM DMA source high](https://gbdev.io/pandocs/CGB_Registers.html#ff51ff52--hdma1-hdma2-cgb-mode-only-vram-dma-source-high-low-write-only)
#[cfg(any(feature="color", doc))]
#[doc(cfg(feature="color"))]
pub const HDMA1: VolAddress<u8, (), Safe> = 
    unsafe { VolAddress::new(0xFF51) };

/// [VRAM DMA source low](https://gbdev.io/pandocs/CGB_Registers.html#ff51ff52--hdma1-hdma2-cgb-mode-only-vram-dma-source-high-low-write-only)
#[cfg(any(feature="color", doc))]
#[doc(cfg(feature="color"))]
pub const HDMA2: VolAddress<u8, (), Safe> = 
    unsafe { VolAddress::new(0xFF52) };

/// [VRAM DMA destination high](https://gbdev.io/pandocs/CGB_Registers.html#ff53ff54--hdma3-hdma4-cgb-mode-only-vram-dma-destination-high-low-write-only)
#[cfg(any(feature="color", doc))]
#[doc(cfg(feature="color"))]
pub const HDMA3: VolAddress<u8, (), Safe> = 
    unsafe { VolAddress::new(0xFF53) };

/// [VRAM DMA destination low](https://gbdev.io/pandocs/CGB_Registers.html#ff53ff54--hdma3-hdma4-cgb-mode-only-vram-dma-destination-high-low-write-only)
#[cfg(any(feature="color", doc))]
#[doc(cfg(feature="color"))]
pub const HDMA4: VolAddress<u8, (), Safe> = 
    unsafe { VolAddress::new(0xFF54) };

/// [VRAM DMA length/mode/start](https://gbdev.io/pandocs/CGB_Registers.html#ff55--hdma5-cgb-mode-only-vram-dma-lengthmodestart)
#[cfg(any(feature="color", doc))]
#[doc(cfg(feature="color"))]
pub const HDMA5: VolAddress<u8, Safe, Unsafe> = 
    unsafe { VolAddress::new(0xFF55) };

/// [Infrared communications port](https://gbdev.io/pandocs/CGB_Registers.html#ff56--rp-cgb-mode-only-infrared-communications-port)
#[cfg(any(feature="color", doc))]
#[doc(cfg(feature="color"))]
pub const RP: VolAddress<u8, Safe, Unsafe> = 
    unsafe { VolAddress::new(0xFF56) };

/// [Background color palette specification](https://gbdev.io/pandocs/Palettes.html#ff68--bcpsbgpi-cgb-mode-only-background-color-palette-specification--background-palette-index)
#[cfg(any(feature="color", doc))]
#[doc(cfg(feature="color"))]
pub const BCPS: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF68) };

/// [Background color palette data](https://gbdev.io/pandocs/Palettes.html#ff69--bcpdbgpd-cgb-mode-only-background-color-palette-data--background-palette-data)
#[cfg(any(feature="color", doc))]
#[doc(cfg(feature="color"))]
pub const BCPD: VolAddress<u8, Unsafe, Unsafe> = 
    unsafe { VolAddress::new(0xFF69) };

/// [OBJ color palette specification](https://gbdev.io/pandocs/Palettes.html#ff6aff6b--ocpsobpi-ocpdobpd-cgb-mode-only-obj-color-palette-specification--obj-palette-index-obj-color-palette-data--obj-palette-data)
#[cfg(any(feature="color", doc))]
#[doc(cfg(feature="color"))]
pub const OCPS: VolAddress<u8, Unsafe, Unsafe> = 
    unsafe { VolAddress::new(0xFF6A) };

/// [OBJ color palette data](https://gbdev.io/pandocs/Palettes.html#ff6aff6b--ocpsobpi-ocpdobpd-cgb-mode-only-obj-color-palette-specification--obj-palette-index-obj-color-palette-data--obj-palette-data)
#[cfg(any(feature="color", doc))]
#[doc(cfg(feature="color"))]
pub const OCPD: VolAddress<u8, Unsafe, Unsafe> = 
    unsafe { VolAddress::new(0xFF6B) };

/// [Object priority mode](https://gbdev.io/pandocs/CGB_Registers.html#ff6c--opri-cgb-mode-only-object-priority-mode)
#[cfg(any(feature="color", doc))]
#[doc(cfg(feature="color"))]
pub const OPRI: VolAddress<u8, Unsafe, Unsafe> = 
    unsafe { VolAddress::new(0xFF6C) };

/// [WRAM bank](https://gbdev.io/pandocs/CGB_Registers.html#ff70--svbk-cgb-mode-only-wram-bank)
#[cfg(any(feature="color", doc))]
#[doc(cfg(feature="color"))]
pub const SVBK: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFF70) };

/// [Audio digital outputs 1 & 2](https://gbdev.io/pandocs/Audio_details.html#ff76--pcm12-cgb-mode-only-digital-outputs-1--2-read-only)
///
/// The low nibble is a copy of a sound channel 1's digital output, the high nibble is a copy of
/// sound channel 2's
#[cfg(any(feature="color", doc))]
#[doc(cfg(feature="color"))]
pub const PCM12: VolAddress<u8, Safe, ()> = 
    unsafe { VolAddress::new(0xFF76) };

/// [Audio digital outputs 3 & 4](https://gbdev.io/pandocs/Audio_details.html#ff77--pcm34-cgb-mode-only-digital-outputs-3--4-read-only)
///
/// Same with [`PCM12`], but with channels 3 and 4.
#[cfg(any(feature="color", doc))]
#[doc(cfg(feature="color"))]
pub const PCM34: VolAddress<u8, Safe, ()> = 
    unsafe { VolAddress::new(0xFF77) };

/// [Interrupt enable](https://gbdev.io/pandocs/Interrupts.html#ffff--ie-interrupt-enable)
///
/// <table class="bit-descrs"><thead><tr><th></th><th>7</th><th>6</th><th>5</th><th>4</th><th>3</th><th>2</th><th>1</th><th>0</th></tr></thead><tbody><tr><td><strong>IE</strong></td><td colspan="3" class="unused-field"></td><td colspan="1">Joypad</td><td colspan="1">Serial</td><td colspan="1">Timer</td><td colspan="1">LCD</td><td colspan="1">VBlank</td></tr></tbody></table>
///
/// * **VBlank** (Read/Write) : Controls whether the VBlank interrupt handler may be called (see
/// [`IF`]).
/// * **LCD** (Read/Write) : Controls whether the LCD interrupt handler may be called (see [`IF`])
/// * **Timer** (Read/Write) : Controls whether the Timer interrupt handler may be calle (see
/// [`IF`])
/// * **Serial** (Read/Write) : Controls whether the Serial interrupt handler may be called (see
/// [`IF`])
/// * **Joypad** (Read/Write) : Controls whether the Joypad interrupt handler may be called (see
/// [`IF`])
pub const IE: VolAddress<u8, Safe, Safe> = 
    unsafe { VolAddress::new(0xFFFF) };
