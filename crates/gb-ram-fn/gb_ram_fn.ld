/* gb_ram_fn.ld — placement for gb-ram-fn's RAM-copyable functions.
 * Use with: ld.lld -T gb.ld -T gb_ram_fn.ld ...
 *
 * Collects each `ram_fn!` function and its two boundary markers into ROM. SORT
 * keeps a function's `_0`/`_1`/`_2` sections adjacent, so `END - START` is the
 * function's exact length. Placed after gb-rt's ROM sections (no fixed address),
 * so it does not collide with the resident `.text`/`.data` load addresses.
 */
SECTIONS {
    .ramfn (LOADADDR(.bss) + SIZEOF(.bss)) : AT(LOADADDR(.bss) + SIZEOF(.bss)) {
        KEEP(*(SORT(ramfn_*)))
    } > ROM
}
