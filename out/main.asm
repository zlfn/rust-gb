;--------------------------------------------------------
; File Created by SDCC : free open source ISO C Compiler 
; Version 4.4.0 #14620 (Linux)
;--------------------------------------------------------
	.module main
	.optsdcc -mz80
	
;--------------------------------------------------------
; Public variables in this module
;--------------------------------------------------------
	.globl _delay
	.globl _main
	.globl _add
	.globl _rust_begin_unwind
;--------------------------------------------------------
; special function registers
;--------------------------------------------------------
;--------------------------------------------------------
; ram data
;--------------------------------------------------------
	.area _DATA
;--------------------------------------------------------
; ram data
;--------------------------------------------------------
	.area _INITIALIZED
;--------------------------------------------------------
; absolute external ram data
;--------------------------------------------------------
	.area _DABS (ABS)
;--------------------------------------------------------
; global & static initialisations
;--------------------------------------------------------
	.area _HOME
	.area _GSINIT
	.area _GSFINAL
	.area _GSINIT
;--------------------------------------------------------
; Home
;--------------------------------------------------------
	.area _HOME
	.area _HOME
;--------------------------------------------------------
; code
;--------------------------------------------------------
	.area _CODE
;./out/main.c:59: int main(void) {
;	---------------------------------
; Function main
; ---------------------------------
_main::
;./out/main.c:60: /*tail*/ delay(10000);
	ld	hl, #0x2710
	call	_delay
;./out/main.c:61: return 42;
	ld	de, #0x002a
;./out/main.c:62: }
	ret
;./out/main.c:65: uint8_t add(uint8_t llvm_cbe_left, uint8_t llvm_cbe_right) {
;	---------------------------------
; Function add
; ---------------------------------
_add::
;./out/main.c:52: uint8_t r = a + b;
	add	a, l
;./out/main.c:66: return (llvm_add_u8(llvm_cbe_right, llvm_cbe_left));
;./out/main.c:67: }
	ret
;./out/main.c:70: __noreturn void rust_begin_unwind(void* llvm_cbe_info) {
;	---------------------------------
; Function rust_begin_unwind
; ---------------------------------
_rust_begin_unwind::
;./out/main.c:71: /*tail*/ rust_begin_unwind(llvm_cbe_info);
	call	_rust_begin_unwind
;./out/main.c:72: __builtin_unreachable();
00102$:
;./out/main.c:74: }
	jp	00102$
	.area _CODE
	.area _INITIALIZER
	.area _CABS (ABS)
