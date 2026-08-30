; gbdk_init.s: GBDK runtime initialization for Game Boy
;
; Bridges rrt0 and the prebuilt libgb.a: the init routine, the interrupt handlers
; libgb.a's dispatch tables need, and the internal symbols it calls by name.

    .section .text, "ax"

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

    ; Default the shadow OAM source page to the high byte of _shadow_OAM (0xC000),
    ; then copy the OAM DMA routine to its HRAM landing site (.refresh_OAM).
    ld a, 0xC0
    ldh (__shadow_OAM_base), a
    ld de, .start_refresh_OAM
    ld hl, .refresh_OAM
    ld c, .end_refresh_OAM - .start_refresh_OAM
    rst 0x30

    ; Set default DMG palettes
    ld a, 0xE4          ; BGP: 11 10 01 00
    ldh ( 0x47 ), a     ; BGP
    ldh ( 0x48 ), a     ; OBP0
    ld a, 0x1B          ; OBP1: 00 01 10 11
    ldh ( 0x49 ), a     ; OBP1

    ; Turn screen on (matching GBDK crt0: LCD on, BG tile data at $8800, BG off)
    ld a, 0xC0          ; LCDCF_ON | LCDCF_WIN9C00 (BG off, OBJ off)
    ldh ( 0x40 ), a     ; LCDC

    ld a, 0x01          ; VBL_IFLAG
    ldh ( 0xFF ), a     ; IE
    xor a
    ldh ( 0x0F ), a     ; IF, clear pending
    ei                  ; rrt0 leaves IME off, GBDK code expects it on

    ret

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

    ; Source of the OAM DMA routine, copied into HRAM at .refresh_OAM. Reads the
    ; shadow OAM page from __shadow_OAM_base and skips when it is zero, matching
    ; GBDK. `.refresh_OAM` runs that guard; `.refresh_OAM_DMA` skips it.

.start_refresh_OAM:
    ldh a, (__shadow_OAM_base)
    or a
    ret z
.start_refresh_OAM_DMA:
    ldh ( 0x46 ), a
    ld a, 40
.dma_wait:
    dec a
    jr nz, .dma_wait
    ret
.end_refresh_OAM:

    ; HRAM landing sites for the routine above. libgb.a calls `.refresh_OAM` (and
    ; `.refresh_OAM_DMA`) by name; the linker places them and __gbdk_init copies the
    ; routine here. Referenced, so kept without an explicit KEEP.
    .section _HRAM.refresh_OAM, "aw"
    .globl .refresh_OAM
    .globl .refresh_OAM_DMA
.refresh_OAM:
    .skip (.start_refresh_OAM_DMA - .start_refresh_OAM)
.refresh_OAM_DMA:
    .skip (.end_refresh_OAM - .start_refresh_OAM_DMA)
    .section .text, "ax"

    ; Shadow OAM source page, read by the routine above. libgb.a sets it;
    ; __gbdk_init defaults it to the high byte of _shadow_OAM.
    .section _HRAM.shadow_base, "aw"
    .globl __shadow_OAM_base
__shadow_OAM_base:
    .skip 1
    .section .text, "ax"

    ; gbdk.ld points each vector here unless the program defines the hook itself.
    ; The stack frame matches GBDK's: the pairs go on as AF, HL, BC, DE and the
    ; dispatch loop is entered with a jump, so above the saved pairs a dispatched
    ; handler finds exactly its call return address and the saved table pointer.
    ; A handler that ends the chain itself (`nowait_int_handler`) drops those two
    ; words and returns from the interrupt.

    .globl _gbdk_on_vblank
_gbdk_on_vblank:
    push af
    push hl
    push bc
    push de

    call .refresh_OAM

    xor a
    ld (_vbl_done), a

    ld hl, _sys_time
    inc (hl)
    jr nz, .vbl_no_carry
    inc hl
    inc (hl)
.vbl_no_carry:

    ld hl, _vbl_table
    jr .dispatch

    .globl _gbdk_on_lcd_stat
_gbdk_on_lcd_stat:
    push af
    push hl
    push bc
    push de
    ld hl, _lcd_table
    jr .dispatch

    .globl _gbdk_on_timer
_gbdk_on_timer:
    push af
    push hl
    push bc
    push de
    ld hl, _tim_table
    jr .dispatch

    .globl _gbdk_on_serial
_gbdk_on_serial:
    push af
    push hl
    push bc
    push de
    ld hl, _sio_table
    jr .dispatch

    .globl _gbdk_on_joypad
_gbdk_on_joypad:
    push af
    push hl
    push bc
    push de
    ld hl, _joy_table

    ; HL = handler table
.dispatch:
    ld a, (hl+)
    ld e, a
    ld a, (hl+)
    ld d, a
    or e
    jr z, .int_tail     ; end of table
    push hl
    ld h, d
    ld l, e
    call .call_hl
    pop hl
    jr .dispatch

    ; Drops the call return address and the saved table pointer, then returns from
    ; the interrupt. The tail waits for the LCD to leave modes 2 and 3, so a handler
    ; returns no earlier than the start of mode 2.

    .globl _wait_int_handler
_wait_int_handler:
    add sp, 4
.int_tail:
    pop de
    pop bc
    pop hl
.wait_stat:
    ldh a, ( 0x41 )     ; STAT
    and 0x02            ; STATF_BUSY, set in modes 2 and 3
    jr nz, .wait_stat
    pop af
    reti

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

    ; HL = handler table, DE = handler to append. A table holds (lo, hi) pairs
    ; terminated by 0x0000.
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
    xor a
    ld (hl+), a
    ld (hl), a
    ret

    ; HL = handler table, DE = handler to drop.
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
    ; Shift the remaining entries down over it.
    dec hl
    dec hl
    push hl
.remove_shift:
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
    dec hl
    ld a, (hl+)
    ld b, a
    ld a, (hl+)
    or b
    jr nz, .remove_shift
    pop hl
    ret

    ; Flags arrive in A, matching GBDK's crt0.

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

    ; Zero-initialized by rrt0's WRAM clear.

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
