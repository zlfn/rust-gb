_test_asm_2::
	ld hl, #0xDE02
	ld a, #0xCD
	ld (hl+), a
	ld a, #0xAB
	ld (hl), a
