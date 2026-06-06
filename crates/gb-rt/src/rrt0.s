; rrt0.s — Rust Runtime 0 for Game Boy (SM83)
;
; Minimal Game Boy startup. Section addresses are set by the linker script.
; Interrupt handlers use weak symbols.

    ; ── RST 0x20 — indirect call helper ──
    ; SDCC uses .call_hl, LLVM codegen uses _call_hl (mangled to __call_hl)
    .section _RST20, "ax"
    .globl .call_hl
    .globl __call_hl
.call_hl:
__call_hl:
    jp (hl)

    ; ── RST 0x28 — MemsetSmall: fill C bytes at HL with A ──
    .section _RST28, "ax"
    .globl .MemsetSmall
.MemsetSmall:
    ld (hl+), a
    dec c
    jr nz, .MemsetSmall
    ret

    ; ── RST 0x30 — MemcpySmall: copy C bytes from DE to HL ──
    .section _RST30, "ax"
    .globl .MemcpySmall
.MemcpySmall:
    ld a, (de)
    ld (hl+), a
    inc de
    dec c
    jr nz, .MemcpySmall
    ret

    ; ── Interrupt vectors (8 bytes each, use jp to full handler) ──

    .section _INT_VBL, "ax"
    push af
    jp _isr_vblank

    .section _INT_STAT, "ax"
    push af
    jp _isr_lcd_stat

    .section _INT_TIMER, "ax"
    push af
    jp _isr_timer

    .section _INT_SERIAL, "ax"
    push af
    jp _isr_serial

    .section _INT_JOYPAD, "ax"
    push af
    jp _isr_joypad

    ; ── Weak default handlers ──

    .section .text.rrt0, "ax"

    ; ── ISR trampolines ──

_isr_vblank:
    push bc
    push de
    push hl
    call _on_vblank
    pop hl
    pop de
    pop bc
    pop af
    reti

_isr_lcd_stat:
    push bc
    push de
    push hl
    call _on_lcd_stat
    pop hl
    pop de
    pop bc
    pop af
    reti

_isr_timer:
    push bc
    push de
    push hl
    call _on_timer
    pop hl
    pop de
    pop bc
    pop af
    reti

_isr_serial:
    push bc
    push de
    push hl
    call _on_serial
    pop hl
    pop de
    pop bc
    pop af
    reti

_isr_joypad:
    push bc
    push de
    push hl
    call _on_joypad
    pop hl
    pop de
    pop bc
    pop af
    reti

    ; ── Weak default handlers ──

    .weak _on_vblank
_on_vblank:
    .weak _on_lcd_stat
_on_lcd_stat:
    .weak _on_timer
_on_timer:
    .weak _on_serial
_on_serial:
    .weak _on_joypad
_on_joypad:
    ret

    ; ── ROM header ──

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

    .section _HEADER, "ax"
    .byte 0x00          ; DMG (monochrome)
    .byte 0x00, 0x00    ; Licensee
    .byte 0x00          ; SGB flag
    .byte 0x00          ; Cartridge type
    .byte 0x00          ; ROM size (32KB)
    .byte 0x00          ; RAM size
    .byte 0x01          ; Destination (non-JP)
    .byte 0x00          ; Old licensee
    .byte 0x00          ; ROM version
    .byte 0x00          ; Header checksum (patch later)
    .byte 0x00, 0x00    ; Global checksum (patch later)

    ; ── Raw boot registers ──
    ; Captured by `_reset` at boot.
    .section .bss
    .globl __boot_a
__boot_a:
    .byte 0
    .globl __boot_b
__boot_b:
    .byte 0

    ; ── Initialization ──

    .section .text.rrt0

    .globl _reset
_reset:
    di
    ld sp, 0xE000

    ; ── Capture boot registers ──
    ; The Boot ROM leaves identification values in registers when it hands
    ; control to the cartridge at 0x100, valid only at this first instruction:
    ;   A = 0x01 → DMG, 0xFF → MGB (Pocket/Light), 0x11 → CGB (also GBA)
    ;   B = bit 0 set → GBA (Game Boy Advance in GBC mode)
    ; Hold them in D/E (untouched by the WRAM clear below), then store to WRAM
    ; once it is cleared.
    ld d, a             ; D = boot A
    ld e, b             ; E = boot B

    ; Clear all WRAM (0xC000-0xDFFF)
    xor a
    ld hl, 0xC000
    ld bc, 0x2000
.clear_wram:
    ld (hl+), a
    dec bc
    ld a, b
    or c
    ld a, 0
    jr nz, .clear_wram

    ; ── Persist raw boot registers to WRAM (now cleared) ──
    ld a, d
    ld (__boot_a), a
    ld a, e
    ld (__boot_b), a

    ; Clear all HRAM (0xFF80-0xFFFE) so NOLOAD cells (such as the bank shadow)
    ; start at zero, mirroring the WRAM clear above. This zeroes the address range
    ; only, referencing no `_HRAM.*` symbol, so unused cells are still GC'd.
    xor a
    ld hl, 0xFF80
    ld c, 0x7F
.clear_hram:
    ld (hl+), a
    dec c
    jr nz, .clear_hram

    ; ── Reset hardware state ──
    ; Clean up residual state left by the Boot ROM so every program
    ; starts from a known-good baseline, even without GBDK.
    xor a
    ldh ( 0x26 ), a     ; NR52 — sound off (silences Boot ROM jingle)
    ldh ( 0x42 ), a     ; SCY  — scroll Y = 0
    ldh ( 0x43 ), a     ; SCX  — scroll X = 0
    ldh ( 0x41 ), a     ; STAT — clear LCD status mode/interrupt config
    ldh ( 0x4A ), a     ; WY   — window Y = 0
    ld a, 0x07
    ldh ( 0x4B ), a     ; WX   — window X = 7 (standard left edge)

    ; Copy .data initializers
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

    ei
    call _main

.halt_loop:
    halt
    jr .halt_loop
