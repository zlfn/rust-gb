_test_asm_1::
	ld hl, #0xDE00
	ld a, #0xCD
	ld (hl+), a
	ld a, #0xAB
	ld (hl), a
