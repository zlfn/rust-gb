;--------------------------------------------------------
; File Created by SDCC : free open source ISO C Compiler 
; Version 4.4.0 #14620 (Linux)
;--------------------------------------------------------
	.module main
	.optsdcc -msm83
	
;--------------------------------------------------------
; Public variables in this module
;--------------------------------------------------------
	.globl _circle
	.globl _color
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
;./out/main.c:52: static __forceinline uint8_t llvm_add_u8(uint8_t a, uint8_t b) {
;	---------------------------------
; Function llvm_add_u8
; ---------------------------------
_llvm_add_u8:
;./out/main.c:53: uint8_t r = a + b;
	add	a, e
;./out/main.c:54: return r;
;./out/main.c:55: }
	ret
;./out/main.c:56: static __forceinline uint32_t llvm_mul_u32(uint32_t a, uint32_t b) {
;	---------------------------------
; Function llvm_mul_u32
; ---------------------------------
_llvm_mul_u32:
;./out/main.c:57: uint32_t r = a * b;
	ldhl	sp,	#4
	ld	a, (hl+)
	ld	h, (hl)
;	spillPairReg hl
;	spillPairReg hl
	ld	l, a
;	spillPairReg hl
;	spillPairReg hl
	push	hl
	ldhl	sp,	#4
	ld	a, (hl+)
	ld	h, (hl)
;	spillPairReg hl
;	spillPairReg hl
	ld	l, a
;	spillPairReg hl
;	spillPairReg hl
	push	hl
;./out/main.c:58: return r;
	call	__mullong
;./out/main.c:59: }
	pop	hl
	add	sp, #4
	jp	(hl)
;./out/main.c:64: void main(void) {
;	---------------------------------
; Function main
; ---------------------------------
_main::
	dec	sp
;./out/main.c:67: /*tail*/ color(3, 0, 0);
	xor	a, a
	push	af
	inc	sp
	ld	e, #0x00
	ld	a, #0x03
	call	_color
;./out/main.c:68: llvm_cbe_x_2e_sroa_2e_0_2e_06__PHI_TEMPORARY = 20;   /* for PHI node */
	ldhl	sp,	#0
	ld	(hl), #0x14
;./out/main.c:72: llvm_cbe_bb3_2e_preheader:;
00101$:
;./out/main.c:74: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 20, 15, 0);
	ld	hl, #0x0f
	push	hl
	ld	e, #0x14
	ldhl	sp,	#2
	ld	a, (hl)
	call	_circle
;./out/main.c:75: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 30, 15, 0);
	ld	hl, #0x0f
	push	hl
	ld	e, #0x1e
	ldhl	sp,	#2
	ld	a, (hl)
	call	_circle
;./out/main.c:76: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 40, 15, 0);
	ld	hl, #0x0f
	push	hl
	ld	e, #0x28
	ldhl	sp,	#2
	ld	a, (hl)
	call	_circle
;./out/main.c:77: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 50, 15, 0);
	ld	hl, #0x0f
	push	hl
	ld	e, #0x32
	ldhl	sp,	#2
	ld	a, (hl)
	call	_circle
;./out/main.c:78: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 60, 15, 0);
	ld	hl, #0x0f
	push	hl
	ld	e, #0x3c
	ldhl	sp,	#2
	ld	a, (hl)
	call	_circle
;./out/main.c:79: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 70u, 15, 0);
	ld	hl, #0x0f
	push	hl
	ld	e, #0x46
	ldhl	sp,	#2
	ld	a, (hl)
	call	_circle
;./out/main.c:80: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 80u, 15, 0);
	ld	hl, #0x0f
	push	hl
	ld	e, #0x50
	ldhl	sp,	#2
	ld	a, (hl)
	call	_circle
;./out/main.c:81: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 90u, 15, 0);
	ld	hl, #0x0f
	push	hl
	ld	e, #0x5a
	ldhl	sp,	#2
	ld	a, (hl)
	call	_circle
;./out/main.c:82: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 100u, 15, 0);
	ld	hl, #0x0f
	push	hl
	ld	e, #0x64
	ldhl	sp,	#2
	ld	a, (hl)
	call	_circle
;./out/main.c:83: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 110u, 15, 0);
	ld	hl, #0x0f
	push	hl
	ld	e, #0x6e
	ldhl	sp,	#2
	ld	a, (hl)
	call	_circle
;./out/main.c:84: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 120u, 15, 0);
	ld	hl, #0x0f
	push	hl
	ld	e, #0x78
	ldhl	sp,	#2
	ld	a, (hl)
	call	_circle
;./out/main.c:85: if ((((uint8_t)llvm_cbe_x_2e_sroa_2e_0_2e_06) < ((uint8_t)((uint8_t)-121)))) {
	ldhl	sp,	#0
	ld	a, (hl)
	sub	a, #0x87
	jr	NC, 00108$
;./out/main.c:53: uint8_t r = a + b;
	ld	a, (hl)
	add	a, #0x0a
	ld	(hl), a
;./out/main.c:87: goto llvm_cbe_bb3_2e_preheader;
	jp	00101$
;./out/main.c:93: llvm_cbe_bb7:;
00108$:
;./out/main.c:94: return;
;./out/main.c:95: }
	inc	sp
	ret
;./out/main.c:98: uint32_t add(uint32_t llvm_cbe_left, uint32_t llvm_cbe_right) {
;	---------------------------------
; Function add
; ---------------------------------
_add::
	add	sp, #-2
	push	bc
	ldhl	sp,	#2
	ld	a, e
	ld	(hl+), a
	ld	(hl), d
;./out/main.c:99: return (llvm_mul_u32(llvm_cbe_right, llvm_cbe_left));
	ldhl	sp,	#6
	ld	a, (hl+)
	ld	c, a
	ld	a, (hl+)
	ld	b, a
	ld	a, (hl+)
	ld	e, a
	ld	d, (hl)
;./out/main.c:57: uint32_t r = a * b;
	ldhl	sp,	#2
	ld	a, (hl+)
	ld	h, (hl)
;	spillPairReg hl
;	spillPairReg hl
	ld	l, a
;	spillPairReg hl
;	spillPairReg hl
	push	hl
	ldhl	sp,	#2
	ld	a, (hl+)
	ld	h, (hl)
;	spillPairReg hl
;	spillPairReg hl
	ld	l, a
;	spillPairReg hl
;	spillPairReg hl
	push	hl
;./out/main.c:99: return (llvm_mul_u32(llvm_cbe_right, llvm_cbe_left));
	call	__mullong
;./out/main.c:100: }
	add	sp, #4
	pop	hl
	add	sp, #4
	jp	(hl)
;./out/main.c:103: __noreturn void rust_begin_unwind(void* llvm_cbe_info) {
;	---------------------------------
; Function rust_begin_unwind
; ---------------------------------
_rust_begin_unwind::
;./out/main.c:104: /*tail*/ rust_begin_unwind(llvm_cbe_info);
	call	_rust_begin_unwind
;./out/main.c:105: __builtin_unreachable();
00102$:
;./out/main.c:107: }
	jr	00102$
	.area _CODE
	.area _INITIALIZER
	.area _CABS (ABS)
