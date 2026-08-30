/* gbdk.ld: GBDK fixed-address sections, layered over gb.ld */

/* `_cpu` (GBDK's CPU-type byte) is the raw boot-A value rrt0 captures. Alias it
   to rrt0's `__boot_a` so there is a single source with no copy. */
__cpu = __boot_a;

/* GBDK's dispatchers stand at the vectors unless the program puts its own
   handler there. PROVIDE yields to any definition, so `#[gb_rt::interrupt]`
   takes a vector back. gb_defaults.ld is read after this one and its no-ops
   are reached only where GBDK is not linked at all. */
PROVIDE(_on_vblank   = _gbdk_on_vblank);
PROVIDE(_on_lcd_stat = _gbdk_on_lcd_stat);
PROVIDE(_on_timer    = _gbdk_on_timer);
PROVIDE(_on_serial   = _gbdk_on_serial);
PROVIDE(_on_joypad   = _gbdk_on_joypad);

/* Addresses libgb.a references: the shadow OAM page it DMAs from, and the stack
   top rrt0 sets up. */
_shadow_OAM = 0xC000;
.STACK = 0xE000;

SECTIONS {
    _DRAW_HEADER0 (0x0070) : { KEEP(*(_DRAW_HEADER0)) } > ROM
    _DRAW_HEADER1 (0x00F4) : { KEEP(*(_DRAW_HEADER1)) } > ROM
    _FONT_HEADER0 (0x00F8) : { KEEP(*(_FONT_HEADER0)) } > ROM
}
