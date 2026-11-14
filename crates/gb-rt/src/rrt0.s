; rrt0.s: Rust Runtime 0 for Game Boy (SM83)
;
    ; RST 0x20: call_hl
    .section _RST20, "ax"
    .globl .call_hl
    .globl __call_hl
.call_hl:
__call_hl:
    jp (hl)

    ; RST 0x28: MemsetSmall
    .section _RST28, "ax"
    .globl .MemsetSmall
    .globl __MemsetSmall
.MemsetSmall:
__MemsetSmall:
.Lmemset_small:
    ld (hl+), a
    dec c
    jr nz, .Lmemset_small
    ret

    ; RST 0x30: MemcpySmall
    .section _RST30, "ax"
    .globl .MemcpySmall
    .globl __MemcpySmall
.MemcpySmall:
__MemcpySmall:
.Lmemcpy_small:
    ld a, (de)
    ld (hl+), a
    inc de
    dec c
    jr nz, .Lmemcpy_small
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

    ; Default interrupt handler
    .globl _isr_noop
_isr_noop:
    reti

    ; ROM header
    .section _ENTRY, "ax"
    nop
    jp _reset

    ; I hope I don't get sued by Nintendo :)
    .section _LOGO, "ax"
    .byte 0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B
    .byte 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D
    .byte 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E
    .byte 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99
    .byte 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC
    .byte 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E

    ; These values are patched by the build tool after the link.
    .section _TITLE, "ax"
    .ascii "RUSTGB"
    .byte 0, 0, 0, 0, 0

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

    ; Temporarily save the boot register while initializing.
    ld d, a
    ; CAUTION: Do not use D or B until they are saved below.

    ; Drop residual boot ROM state.
    xor a
    ldh ( 0x26 ), a     ; NR52 : 00
    ldh ( 0x42 ), a     ; SCY  : 00
    ldh ( 0x43 ), a     ; SCX  : 00
    ldh ( 0x4A ), a     ; WY   : 00
    ld a, 0x07          ; The left edge of the window
    ldh ( 0x4B ), a     ; WX   : 07

    ld a, 0xE4
    ldh ( 0x47 ), a     ; BGP  : 11 10 01 00
    ldh ( 0x48 ), a     ; OBP0 : 11 10 01 00

    ld a, 0x1B
    ldh ( 0x49 ), a     ; OBP1 : 00 01 10 11

    ld a, 0xC0          ; LCD on, window map 0x9C00, every layer off
    ldh ( 0x40 ), a     ; LCDC : C0

    ; Initialize WRAM so that the .bss section starts at 0.
    ; Rust expects the startup code to do this.
    xor a
    ld hl, 0xC000
.clear_wram:
    ld (hl+), a
    ld (hl+), a
    ld (hl+), a
    ld (hl+), a
    bit 5, h
    jr z, .clear_wram

    ; And initialize the HRAM as well.
    ld c, 0x80
.clear_hram:
    ldh (c), a
    inc c
    jr nz, .clear_hram

    ; Store the boot registers.
    ld hl, __boot_a
    ld (hl), d
    inc hl ; __boot_b
    ld (hl), b

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

    ; We expect main's signature to be `fn() -> !`,
    ; so it's safe to just jp.
    jp _main
