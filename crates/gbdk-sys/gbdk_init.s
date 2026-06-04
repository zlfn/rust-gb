; gbdk_init.s — GBDK runtime initialization for Game Boy
;
; Provides:
;   __gbdk_init — call from Rust to initialize GBDK runtime
;   _on_vblank  — strong override of rrt0 weak VBlank handler
;   GBDK internal symbols (.display_off, .mode, .add_int, .remove_int)
;
; Assembled with: clang --target=sm83 -c gbdk_init.s -o gbdk_init.o

    .section .text, "ax"

    ; ── GBDK init (called by user from main) ──

    .globl __gbdk_init
__gbdk_init:
    ; __cpu aliases rrt0's raw boot-A (see gbdk.ld), so it is already populated.
    ; Derive __is_GBA from the raw boot-B that rrt0 captured (bit 0 = GBA).
    ld a, (__boot_b)
    and 1
    ld (__is_GBA), a

    ; Clear OAM shadow
    xor a
    ld hl, 0xC000
    ld c, 0xA0
.clear_oam:
    ld (hl+), a
    dec c
    jr nz, .clear_oam

    ; Copy OAM DMA routine to HRAM (0xFF80)
    ld hl, _oam_dma_src
    ld de, 0xFF80
    ld c, _oam_dma_end - _oam_dma_src
.copy_dma:
    ld a, (hl+)
    ld (de), a
    inc de
    dec c
    jr nz, .copy_dma

    ; Set default DMG palettes
    ld a, 0xE4          ; BGP: 11 10 01 00
    ldh ( 0x47 ), a     ; BGP
    ldh ( 0x48 ), a     ; OBP0
    ld a, 0x1B          ; OBP1: 00 01 10 11
    ldh ( 0x49 ), a     ; OBP1

    ; Turn screen on (matching GBDK crt0: LCD on, BG tile data at $8800, BG off)
    ld a, 0xC0          ; LCDCF_ON | LCDCF_WIN9C00 (BG off, OBJ off)
    ldh ( 0x40 ), a     ; LCDC

    ; Enable VBlank interrupt
    ld a, 0x01          ; VBL_IFLAG
    ldh ( 0xFF ), a     ; IE
    xor a
    ldh ( 0x0F ), a     ; IF — clear pending

    ret

    ; ── Display off ──

    .globl _display_off
    .globl .display_off
_display_off:
.display_off:
    ldh a, ( 0x40 )
    bit 7, a
    ret z
.wait_vbl:
    ldh a, ( 0x44 )
    cp 144
    jr nz, .wait_vbl
    ldh a, ( 0x40 )
    res 7, a
    ldh ( 0x40 ), a
    ret

    ; ── OAM DMA source (copied to HRAM) ──

_oam_dma_src:
    ld a, 0xC0
    ldh ( 0x46 ), a
    ld a, 40
.dma_wait:
    dec a
    jr nz, .dma_wait
    ret
_oam_dma_end:

    ; ── VBlank handler (strong override) ──

    ; ── VBlank handler — dispatches registered handlers ──

    .globl _on_vblank
_on_vblank:
    ; OAM DMA
    call 0xFF80

    ; Clear vbl_done flag
    xor a
    ld (_vbl_done), a

    ; Increment sys_time (16-bit VBlank tick counter)
    ld hl, _sys_time
    inc (hl)
    jr nz, .vbl_no_carry
    inc hl
    inc (hl)
.vbl_no_carry:

    ; Dispatch VBL handler table
    ld hl, _vbl_table
    jp _dispatch_table

    ; ── LCD STAT handler ──

    .globl _on_lcd_stat
_on_lcd_stat:
    ld hl, _lcd_table
    jp _dispatch_table

    ; ── Timer handler ──

    .globl _on_timer
_on_timer:
    ld hl, _tim_table
    jp _dispatch_table

    ; ── Serial handler ──

    .globl _on_serial
_on_serial:
    ld hl, _sio_table
    jp _dispatch_table

    ; ── Joypad handler ──

    .globl _on_joypad
_on_joypad:
    ld hl, _joy_table
    jp _dispatch_table

    ; ── Handler table dispatcher ──
    ; HL = pointer to handler table
_dispatch_table:
    ld a, (hl+)
    ld e, a
    ld a, (hl+)
    ld d, a
    or e
    ret z               ; end of table
    push hl
    ld h, d
    ld l, e
    call .call_hl
    pop hl
    jr _dispatch_table

    ; ── add/remove VBL/LCD helpers ──

    .globl _add_VBL
    .globl .add_VBL
_add_VBL:
.add_VBL:
    ld hl, _vbl_table
    jp .add_int

    .globl _remove_VBL
    .globl .remove_VBL
_remove_VBL:
.remove_VBL:
    ld hl, _vbl_table
    jp .remove_int

    .globl _add_LCD
    .globl .add_LCD
_add_LCD:
.add_LCD:
    ld hl, _lcd_table
    jp .add_int

    .globl _remove_LCD
    .globl .remove_LCD
_remove_LCD:
.remove_LCD:
    ld hl, _lcd_table
    jp .remove_int

    ; ── wait_vbl_done / vsync ──

    .globl _wait_vbl_done
    .globl .wait_vbl_done
    .globl _vsync
    .globl .vsync
_wait_vbl_done:
.wait_vbl_done:
_vsync:
.vsync:
    ld a, 1
    ld (_vbl_done), a
.wait_loop:
    halt
    ld a, (_vbl_done)
    and a
    jr nz, .wait_loop
    ret

    ; ── GBDK internal stubs ──

    ; .add_int — Add handler to interrupt table
    ; HL = pointer to handler table, DE = handler function
    ; Table format: pairs of (func_lo, func_hi), terminated by 0x0000
    .globl .add_int
.add_int:
    ld a, (hl+)
    ld b, a
    ld a, (hl)
    or b
    jr z, .add_int_found
    inc hl
    jr .add_int
.add_int_found:
    dec hl
    ld a, e
    ld (hl+), a
    ld a, d
    ld (hl+), a
    ; Zero-terminate
    xor a
    ld (hl+), a
    ld (hl), a
    ret

    ; .remove_int — Remove handler from interrupt table
    ; HL = pointer to handler table, DE = handler function to remove
    .globl .remove_int
.remove_int:
    ld a, (hl+)
    ld b, a
    ld a, (hl+)
    ld c, a
    or b
    ret z               ; end of table, not found
    ld a, b
    cp e
    jr nz, .remove_int
    ld a, c
    cp d
    jr nz, .remove_int
    ; Found — shift remaining entries down
    dec hl
    dec hl
    push hl
.remove_shift:
    ; Copy next entry over current
    inc hl
    inc hl
    ld a, (hl)
    dec hl
    dec hl
    ld (hl+), a
    inc hl
    ld a, (hl)
    dec hl
    ld (hl), a
    ; Check if we just copied a terminator
    dec hl
    ld a, (hl+)
    ld b, a
    ld a, (hl+)
    or b
    jr nz, .remove_shift
    pop hl
    ret

    ; ── set_interrupts(flags) — flags passed in A (matches GBDK crt0) ──
    ; Sets the IE register and clears any pending IF, like GBDK's set_interrupts.

    .globl _set_interrupts
    .globl .set_interrupts
_set_interrupts:
.set_interrupts:
    di
    ld (0xFFFF), a      ; IE_REG = flags
    xor a
    ei
    ld (0xFF0F), a      ; IF_REG = 0 (clear pending)
    ret

    ; ── BSS data (zero-initialized by rrt0) ──

    .section .bss

    ; CPU detection. __cpu aliases rrt0's __boot_a (see gbdk.ld); __is_GBA is
    ; derived from the raw boot-B by __gbdk_init.
    .globl __is_GBA
__is_GBA:
    .byte 0

    .globl _vbl_done
_vbl_done:
    .byte 0

    .globl .mode
.mode:
    .byte 0

    ; System time, in VBlank ticks; incremented by the VBL handler.
    .globl _sys_time
    .globl .sys_time
_sys_time:
.sys_time:
    .byte 0
    .byte 0

    ; Interrupt handler tables (function pointer pairs, zero-terminated)
    ; Each table: up to 4 handlers + terminator = 10 bytes
    .globl _vbl_table
_vbl_table:
    .space 10

    .globl _lcd_table
_lcd_table:
    .space 10

    .globl _tim_table
_tim_table:
    .space 10

    .globl _sio_table
_sio_table:
    .space 10

    .globl _joy_table
_joy_table:
    .space 10
