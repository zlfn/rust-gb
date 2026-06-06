/* gb_hram.ld — High RAM cell allocation for gb-hram.
 * Use with: ld.lld -T gb.ld -T gb_hram.ld ...
 *
 * Collects `_HRAM.*` cells into the HRAM region (the region itself is defined by
 * gb-rt's gb.ld). Sorted by name, so a leading reservation such as GBDK's OAM DMA
 * span (`_HRAM.00_oamdma`) lands first and ordinary cells follow.
 */
SECTIONS {
    .hram (NOLOAD) : { *(SORT(_HRAM*)) } > HRAM
}
