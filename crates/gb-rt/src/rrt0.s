; rrt0.s: Rust Runtime 0 for Game Boy (SM83)
;
; Section addresses come from gb.ld. See the crate docs for the interrupt model.

    ; RST 0x20: indirect call to HL
    ; The backend lowers its CALL_HL pseudo to `call __call_hl`; SDCC and GBDK
    ; reach the same routine as `.call_hl`.
    .section _RST20, "ax"
    .globl .call_hl
    .globl __call_hl
.call_hl:
__call_hl:
    jp (hl)

    ; RST 0x28: fill C bytes at HL with A
    .section _RST28, "ax"
    .globl .MemsetSmall
    .globl __MemsetSmall
.MemsetSmall:
__MemsetSmall:
    ld (hl+), a
    dec c
    jr nz, .MemsetSmall
    ret

    ; RST 0x30: copy C bytes from DE to HL
    .section _RST30, "ax"
    .globl .MemcpySmall
    .globl __MemcpySmall
.MemcpySmall:
__MemcpySmall:
    ld a, (de)
    ld (hl+), a
    inc de
    dec c
    jr nz, .MemcpySmall
    ret

    ; Interrupt vectors

    .section _INT_VBL, "ax"
    jp _on_vblank

    .section _INT_STAT, "ax"
    jp _on_lcd_stat

    .section _INT_TIMER, "ax"
    jp _on_timer

    .section _INT_SERIAL, "ax"
    jp _on_serial

    .section _INT_JOYPAD, "ax"
    jp _on_joypad

    .section .text.rrt0, "ax"

    ; What every `_on_*` hook resolves to unless something defines it, bound by
    ; PROVIDE in gb.ld.
    .globl _isr_noop
_isr_noop:
    reti

    ; ROM header

    .section _ENTRY, "ax"
    nop
    jp _reset

    .section _LOGO, "ax"
    .byte 0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B
    .byte 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D
    .byte 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E
    .byte 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99
    .byte 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC
    .byte 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E

    .section _TITLE, "ax"
    .ascii "RUSTGB"
    .byte 0, 0, 0, 0, 0

    ; Cartridge type, ROM size, and both checksums are patched after the link.
    .section _HEADER, "ax"
    .byte 0x00          ; CGB flag
    .byte 0x00, 0x00    ; Licensee
    .byte 0x00          ; SGB flag
    .byte 0x00          ; Cartridge type
    .byte 0x00          ; ROM size
    .byte 0x00          ; RAM size
    .byte 0x01          ; Destination (non-JP)
    .byte 0x00          ; Old licensee
    .byte 0x00          ; ROM version
    .byte 0x00          ; Header checksum
    .byte 0x00, 0x00    ; Global checksum

    ; Boot registers

    .section .bss
    .globl __boot_a
__boot_a:
    .byte 0
    .globl __boot_b
__boot_b:
    .byte 0

    ; Startup

    .section .text.rrt0

    .globl _reset
_reset:
    di
    ld sp, 0xE000

    ; The boot ROM's identification bytes in A and B are valid only here, at the
    ; first instruction. Hold them in D/E until WRAM is cleared.
    ld d, a
    ld e, b

    ; Zero WRAM so `.bss` starts at zero and an uninitialized read is
    ; reproducible.
    xor a
    ld hl, 0xC000
    ld b, 0x20
.wram_page:
    ld c, 0
.wram_byte:
    ld (hl+), a
    dec c
    jr nz, .wram_byte
    dec b
    jr nz, .wram_page

    ld a, d
    ld (__boot_a), a
    ld a, e
    ld (__boot_b), a

    ; Zero HRAM so NOLOAD cells such as the bank shadow start at zero. This names
    ; no `_HRAM.*` symbol, so unused cells are still collected.
    xor a
    ld hl, 0xFF80
    ld c, 0x7F
.clear_hram:
    ld (hl+), a
    dec c
    jr nz, .clear_hram

    ; Drop residual boot ROM state.
    xor a
    ldh ( 0x26 ), a     ; NR52, silences the boot jingle
    ldh ( 0x42 ), a     ; SCY
    ldh ( 0x43 ), a     ; SCX
    ldh ( 0x41 ), a     ; STAT
    ldh ( 0x4A ), a     ; WY
    ld a, 0x07
    ldh ( 0x4B ), a     ; WX, standard left edge

    ; The boot ROM hands over with the background on and the logo in the tilemap.
    ld a, 0xE4          ; BGP, OBP0: 11 10 01 00
    ldh ( 0x47 ), a     ; BGP
    ldh ( 0x48 ), a     ; OBP0
    ld a, 0x1B          ; OBP1: 00 01 10 11
    ldh ( 0x49 ), a     ; OBP1
    ld a, 0xC0          ; LCD on, window map 0x9C00, every layer off
    ldh ( 0x40 ), a     ; LCDC

    ; Copy the `.data` initializers from ROM.
    ld hl, s__INITIALIZER
    ld de, s__INITIALIZED
    ld bc, l__INITIALIZER
    ld a, b
    or c
    jr z, .data_done
.data_copy:
    ld a, (hl+)
    ld (de), a
    inc de
    dec bc
    ld a, b
    or c
    jr nz, .data_copy
.data_done:

    ; IME stays off: turning interrupts on is the program's call.
    jp _main            ; `fn() -> !`, so it never comes back
