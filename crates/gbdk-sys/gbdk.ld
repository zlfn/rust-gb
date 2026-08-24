/* gbdk.ld: GBDK fixed-address sections, layered over gb.ld */

/* `_cpu` (GBDK's CPU-type byte) is the raw boot-A value rrt0 captures. Alias it
   to rrt0's `__boot_a` so there is a single source with no copy. */
__cpu = __boot_a;

/* Addresses libgb.a references: the shadow OAM page it DMAs from, and the stack
   top rrt0 sets up. */
_shadow_OAM = 0xC000;
.STACK = 0xE000;

SECTIONS {
    _DRAW_HEADER0 (0x0070) : { KEEP(*(_DRAW_HEADER0)) } > ROM
    _DRAW_HEADER1 (0x00F4) : { KEEP(*(_DRAW_HEADER1)) } > ROM
    _FONT_HEADER0 (0x00F8) : { KEEP(*(_FONT_HEADER0)) } > ROM
}
