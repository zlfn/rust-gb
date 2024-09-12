;--------------------------------------------------------
; File Created by SDCC : free open source ISO C Compiler 
; Version 4.4.0 #14620 (Linux)
;--------------------------------------------------------
	.module main
	.optsdcc -msm83
	
;--------------------------------------------------------
; Public variables in this module
;--------------------------------------------------------
	.globl _memcpy
	.globl _calloc
	.globl _circle
	.globl _color
	.globl _free
	.globl _malloc
	.globl _main
	.globl _add
	.globl _rust_begin_unwind
	.globl ___rust_alloc
	.globl ___rust_dealloc
	.globl ___rust_realloc
	.globl ___rust_alloc_zeroed
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
;./out/main.c:60: static __forceinline uint8_t llvm_add_u8(uint8_t a, uint8_t b) {
;	---------------------------------
; Function llvm_add_u8
; ---------------------------------
_llvm_add_u8:
;./out/main.c:61: uint8_t r = a + b;
	add	a, e
;./out/main.c:62: return r;
;./out/main.c:63: }
	ret
;./out/main.c:64: static __forceinline uint32_t llvm_mul_u32(uint32_t a, uint32_t b) {
;	---------------------------------
; Function llvm_mul_u32
; ---------------------------------
_llvm_mul_u32:
;./out/main.c:65: uint32_t r = a * b;
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
;./out/main.c:66: return r;
	call	__mullong
;./out/main.c:67: }
	pop	hl
	add	sp, #4
	jp	(hl)
;./out/main.c:68: static __forceinline uint16_t llvm_OC_umin_OC_i16(uint16_t a, uint16_t b) {
;	---------------------------------
; Function llvm_OC_umin_OC_i16
; ---------------------------------
_llvm_OC_umin_OC_i16:
;./out/main.c:70: r = a < b ? a : b;
	ld	a, e
	sub	a, c
	ld	a, d
	sbc	a, b
	ret	NC
	ld	c, e
	ld	b, d
;./out/main.c:71: return r;
;./out/main.c:72: }
	ret
;./out/main.c:77: void main(void) {
;	---------------------------------
; Function main
; ---------------------------------
_main::
	dec	sp
;./out/main.c:80: /*tail*/ color(3, 0, 0);
	xor	a, a
	rrca
	push	af
	xor	a, a
	ld	a, #0x03
	push	af
	inc	sp
	call	_color
	add	sp, #3
;./out/main.c:81: llvm_cbe_x_2e_sroa_2e_0_2e_06__PHI_TEMPORARY = 20;   /* for PHI node */
	ldhl	sp,	#0
	ld	(hl), #0x14
;./out/main.c:85: llvm_cbe_bb3_2e_preheader:;
00101$:
;./out/main.c:87: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 20, 15, 0);
	xor	a, a
	ld	h, a
	ld	l, #0x0f
	push	hl
	ld	a, #0x14
	push	af
	inc	sp
	ldhl	sp,	#3
	ld	a, (hl)
	push	af
	inc	sp
	call	_circle
	add	sp, #4
;./out/main.c:88: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 30, 15, 0);
	xor	a, a
	ld	h, a
	ld	l, #0x0f
	push	hl
	ld	a, #0x1e
	push	af
	inc	sp
	ldhl	sp,	#3
	ld	a, (hl)
	push	af
	inc	sp
	call	_circle
	add	sp, #4
;./out/main.c:89: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 40, 15, 0);
	xor	a, a
	ld	h, a
	ld	l, #0x0f
	push	hl
	ld	a, #0x28
	push	af
	inc	sp
	ldhl	sp,	#3
	ld	a, (hl)
	push	af
	inc	sp
	call	_circle
	add	sp, #4
;./out/main.c:90: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 50, 15, 0);
	xor	a, a
	ld	h, a
	ld	l, #0x0f
	push	hl
	ld	a, #0x32
	push	af
	inc	sp
	ldhl	sp,	#3
	ld	a, (hl)
	push	af
	inc	sp
	call	_circle
	add	sp, #4
;./out/main.c:91: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 60, 15, 0);
	xor	a, a
	ld	h, a
	ld	l, #0x0f
	push	hl
	ld	a, #0x3c
	push	af
	inc	sp
	ldhl	sp,	#3
	ld	a, (hl)
	push	af
	inc	sp
	call	_circle
	add	sp, #4
;./out/main.c:92: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 70u, 15, 0);
	xor	a, a
	ld	h, a
	ld	l, #0x0f
	push	hl
	ld	a, #0x46
	push	af
	inc	sp
	ldhl	sp,	#3
	ld	a, (hl)
	push	af
	inc	sp
	call	_circle
	add	sp, #4
;./out/main.c:93: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 80u, 15, 0);
	xor	a, a
	ld	h, a
	ld	l, #0x0f
	push	hl
	ld	a, #0x50
	push	af
	inc	sp
	ldhl	sp,	#3
	ld	a, (hl)
	push	af
	inc	sp
	call	_circle
	add	sp, #4
;./out/main.c:94: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 90u, 15, 0);
	xor	a, a
	ld	h, a
	ld	l, #0x0f
	push	hl
	ld	a, #0x5a
	push	af
	inc	sp
	ldhl	sp,	#3
	ld	a, (hl)
	push	af
	inc	sp
	call	_circle
	add	sp, #4
;./out/main.c:95: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 100u, 15, 0);
	xor	a, a
	ld	h, a
	ld	l, #0x0f
	push	hl
	ld	a, #0x64
	push	af
	inc	sp
	ldhl	sp,	#3
	ld	a, (hl)
	push	af
	inc	sp
	call	_circle
	add	sp, #4
;./out/main.c:96: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 110u, 15, 0);
	xor	a, a
	ld	h, a
	ld	l, #0x0f
	push	hl
	ld	a, #0x6e
	push	af
	inc	sp
	ldhl	sp,	#3
	ld	a, (hl)
	push	af
	inc	sp
	call	_circle
	add	sp, #4
;./out/main.c:97: /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 120u, 15, 0);
	xor	a, a
	ld	h, a
	ld	l, #0x0f
	push	hl
	ld	a, #0x78
	push	af
	inc	sp
	ldhl	sp,	#3
	ld	a, (hl)
	push	af
	inc	sp
	call	_circle
	add	sp, #4
;./out/main.c:98: if ((((uint8_t)llvm_cbe_x_2e_sroa_2e_0_2e_06) < ((uint8_t)((uint8_t)-121)))) {
	ldhl	sp,	#0
;./out/main.c:61: uint8_t r = a + b;
	ld	a,(hl)
	cp	a,#0x87
	jr	NC, 00108$
	add	a, #0x0a
	ld	(hl), a
;./out/main.c:100: goto llvm_cbe_bb3_2e_preheader;
	jp	00101$
;./out/main.c:106: llvm_cbe_bb7:;
00108$:
;./out/main.c:107: return;
;./out/main.c:108: }
	inc	sp
	ret
;./out/main.c:111: uint32_t add(uint32_t llvm_cbe_left, uint32_t llvm_cbe_right) {
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
;./out/main.c:112: return (llvm_mul_u32(llvm_cbe_right, llvm_cbe_left));
	ldhl	sp,	#6
	ld	a, (hl+)
	ld	c, a
	ld	a, (hl+)
	ld	b, a
	ld	a, (hl+)
	ld	e, a
	ld	d, (hl)
;./out/main.c:65: uint32_t r = a * b;
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
;./out/main.c:112: return (llvm_mul_u32(llvm_cbe_right, llvm_cbe_left));
	call	__mullong
;./out/main.c:113: }
	add	sp, #4
	pop	hl
	add	sp, #4
	jp	(hl)
;./out/main.c:116: __noreturn void rust_begin_unwind(void* llvm_cbe_info) {
;	---------------------------------
; Function rust_begin_unwind
; ---------------------------------
_rust_begin_unwind::
;./out/main.c:117: /*tail*/ rust_begin_unwind(llvm_cbe_info);
	call	_rust_begin_unwind
;./out/main.c:118: __builtin_unreachable();
00102$:
;./out/main.c:120: }
	jr	00102$
;./out/main.c:123: void* __rust_alloc(uint16_t llvm_cbe_size, uint16_t llvm_cbe_align) {
;	---------------------------------
; Function __rust_alloc
; ---------------------------------
___rust_alloc::
;./out/main.c:125: void* llvm_cbe__4_2e_i =  /*tail*/ malloc(llvm_cbe_size);
;./out/main.c:126: return llvm_cbe__4_2e_i;
;./out/main.c:127: }
	jp	_malloc
;./out/main.c:130: void __rust_dealloc(void* llvm_cbe_ptr, uint16_t llvm_cbe_size, uint16_t llvm_cbe_align) {
;	---------------------------------
; Function __rust_dealloc
; ---------------------------------
___rust_dealloc::
;./out/main.c:131: /*tail*/ free(llvm_cbe_ptr);
	call	_free
;./out/main.c:132: }
	pop	hl
	pop	af
	jp	(hl)
;./out/main.c:135: void* __rust_realloc(void* llvm_cbe_ptr, uint16_t llvm_cbe_size, uint16_t llvm_cbe_align, uint16_t llvm_cbe_new_size) {
;	---------------------------------
; Function __rust_realloc
; ---------------------------------
___rust_realloc::
	add	sp, #-4
	ldhl	sp,	#2
	ld	a, e
	ld	(hl+), a
	ld	(hl), d
	inc	sp
	inc	sp
	push	bc
;./out/main.c:138: llvm_cbe__4_2e_i_2e_i =  /*tail*/ malloc(llvm_cbe_new_size);
	ldhl	sp,	#8
	ld	a, (hl+)
	ld	e, a
	ld	d, (hl)
	call	_malloc
	ld	e, c
	ld	d, b
;./out/main.c:139: if ((llvm_cbe__4_2e_i_2e_i == ((void*)/*NULL*/0))) {
	ld	a, d
	or	a, e
	jr	Z, 00105$
;./out/main.c:146: uint16_t llvm_cbe_size_2e_sroa_2e_0_2e_0_2e_i =  /*tail*/ llvm_OC_umin_OC_i16(llvm_cbe_size, llvm_cbe_new_size);
	ldhl	sp,	#8
	ld	a, (hl+)
	ld	c, a
	ld	b, (hl)
	ldhl	sp,	#0
	ld	a, (hl+)
	sub	a, c
	ld	a, (hl)
	sbc	a, b
	jr	NC, 00109$
	pop	bc
	push	bc
00109$:
;./out/main.c:147: void* _1 = memcpy(llvm_cbe__4_2e_i_2e_i, llvm_cbe_ptr, llvm_cbe_size_2e_sroa_2e_0_2e_0_2e_i);
	push	de
	push	bc
	ldhl	sp,	#6
	ld	c, (hl)
	inc	hl
	ld	b, (hl)
	call	_memcpy
	ldhl	sp,	#4
	ld	e, (hl)
	inc	hl
	ld	d, (hl)
	call	_free
	pop	de
;./out/main.c:151: llvm_cbe__ZN68__24_LT_24_main_2e__2e_LibcAlloc_24_u20_24_as_24_u20_24_core_2e__2e_alloc_2e__2e_global_2e__2e_GlobalAlloc_24_GT_24_7realloc17h3ef0212bdae7f97aE_2e_exit:;
00105$:
;./out/main.c:152: return llvm_cbe__4_2e_i_2e_i;
	ld	c, e
	ld	b, d
;./out/main.c:153: }
	add	sp, #4
	pop	hl
	add	sp, #4
	jp	(hl)
;./out/main.c:156: void* __rust_alloc_zeroed(uint16_t llvm_cbe_size, uint16_t llvm_cbe_align) {
;	---------------------------------
; Function __rust_alloc_zeroed
; ---------------------------------
___rust_alloc_zeroed::
	ld	c, e
	ld	b, d
;./out/main.c:158: void* llvm_cbe_calloc_2e_i =  /*tail*/ calloc(1, llvm_cbe_size);
	ld	de, #0x0001
;./out/main.c:159: return llvm_cbe_calloc_2e_i;
;./out/main.c:160: }
	jp	_calloc
	.area _CODE
	.area _INITIALIZER
	.area _CABS (ABS)
