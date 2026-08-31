/* gbdk.ld: GBDK fixed-address sections, layered over gb.ld */

/* `_cpu` (GBDK's CPU-type byte) is the raw boot-A value rrt0 captures. Alias it
   to rrt0's `__boot_a` so there is a single source with no copy. */
__cpu = __boot_a;

/* GBDK's dispatchers stand at the vectors unless the program puts its own
   handler there. PROVIDE yields to any definition, so `#[gb_rt::interrupt]`
   takes a vector back.

   The build pipeline hands the linker every script it finds, this one included,
   for a program that never links GBDK. DEFINED keeps the dispatchers optional,
   falling back to what gb_defaults.ld would otherwise have supplied. */
PROVIDE(_on_vblank   = DEFINED(_gbdk_on_vblank)   ? _gbdk_on_vblank   : _isr_noop);
PROVIDE(_on_lcd_stat = DEFINED(_gbdk_on_lcd_stat) ? _gbdk_on_lcd_stat : _isr_noop);
PROVIDE(_on_timer    = DEFINED(_gbdk_on_timer)    ? _gbdk_on_timer    : _isr_noop);
PROVIDE(_on_serial   = DEFINED(_gbdk_on_serial)   ? _gbdk_on_serial   : _isr_noop);
PROVIDE(_on_joypad   = DEFINED(_gbdk_on_joypad)   ? _gbdk_on_joypad   : _isr_noop);

/* Addresses libgb.a references: the shadow OAM page it DMAs from, and the stack
   top rrt0 sets up. */
_shadow_OAM = 0xC000;
.STACK = 0xE000;

SECTIONS {
    _DRAW_HEADER0 (0x0070) : { KEEP(*(_DRAW_HEADER0)) } > ROM
    _DRAW_HEADER1 (0x00F4) : { KEEP(*(_DRAW_HEADER1)) } > ROM
    _FONT_HEADER0 (0x00F8) : { KEEP(*(_FONT_HEADER0)) } > ROM
}
