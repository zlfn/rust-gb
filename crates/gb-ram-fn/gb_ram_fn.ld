/* gb_ram_fn.ld — placement for gb-ram-fn's RAM-copyable functions.
 * Use with: ld.lld -T gb.ld -T gb_ram_fn.ld ...
 *
 * Collects each `ram_fn!` function and its two boundary markers into ROM. SORT
 * keeps a function's `_0`/`_1`/`_2` sections adjacent, so `END - START` is the
 * function's exact length. Inserted after `.data`, so it follows the `.data`
 * initializers in bank 0, ahead of the banks gb_banked.ld adds.
 */
SECTIONS {
    .ramfn : {
        KEEP(*(SORT(ramfn_*)))
    } > ROM
}
INSERT AFTER .data;
