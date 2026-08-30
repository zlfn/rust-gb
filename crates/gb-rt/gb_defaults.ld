/* gb_defaults.ld: what the interrupt vectors resolve to when nothing else says.

   PROVIDE yields to any object definition, weak or strong, and also to an
   earlier PROVIDE. The build pipeline passes this script after every other one
   so a library offering its own handler is the one that lands, and these are
   reached only when none does. */

PROVIDE(_on_vblank   = _isr_noop);
PROVIDE(_on_lcd_stat = _isr_noop);
PROVIDE(_on_timer    = _isr_noop);
PROVIDE(_on_serial   = _isr_noop);
PROVIDE(_on_joypad   = _isr_noop);
