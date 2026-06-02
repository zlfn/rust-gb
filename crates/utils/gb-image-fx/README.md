# gb-image-fx

Convert any image to Game Boy / Game Boy Color tile data. Supports PNG, JPEG, BMP, GIF, WebP, and more.

## Usage

```
gb-image-fx <input> [OPTIONS]
```

### Options

| Option | Description |
|---|---|
| `-o <prefix>` | Output file prefix (default: input file stem) |
| `--quantize <WxH>` | Quantize arbitrary image to target resolution (e.g. `160x144`) |
| `--max-palettes <N>` | Maximum palettes for quantize (default: 8, GBC max) |
| `--samples <N>` | K-Means initial color samples for quantize (default: 448) |
| `--tiles-only` | Only output tiles and palettes (no tile map) |
| `--keep-duplicates` | Don't deduplicate tiles |
| `--no-flip` | Don't detect flipped tiles during dedup |

### Output Files

| File | Description |
|---|---|
| `{prefix}_tiles.bin` | 2bpp tile data (16 bytes per tile) |
| `{prefix}_palettes.bin` | GBC RGB555 palettes (8 bytes per palette) |
| `{prefix}_attributes.bin` | GBC tile attributes with palette index (1 byte per tile) |
| `{prefix}_map.bin` | Tile map (1 byte per grid cell, omitted with `--tiles-only`) |

## Quantize Pipeline

`--quantize` converts an arbitrary photo into GBC-compatible tile data:

1. **Box Sampling** — Downsample to target resolution
2. **K-Means** — Extract N initial representative colors from all pixels (default: 448)
3. **Farthest Point Sampling** — Select 32 diverse candidates in Oklch color space
4. **Tile Analysis** — For each 8x8 tile, find top 4 candidates by pixel frequency
5. **Agglomerative Clustering** — Merge tiles with overlapping color sets until max palettes reached
6. **Palette Extraction** — K-Means(k=4) per group on actual pixels for final 4-color palette
7. **Pixel Snap** — Map each pixel to nearest color in its tile's palette

All color operations use [Oklch](https://bottosson.github.io/posts/oklab/) for perceptual accuracy. The algorithm is fully deterministic.

## Palette Ordering

Colors within each palette are sorted by Oklch lightness (brightest first).
This matches the DMG convention where palette index 0 = white and index 3 = black.

## Constraints

- Without `--quantize`: image dimensions must be multiples of 8, max 4 colors per tile.
- With `--quantize`: any input resolution, automatically resized and quantized.
- Maximum 8 BG palettes (GBC hardware limit), configurable with `--max-palettes`.
- Maximum 256 unique tiles for tile map mode (u8 index).

## Examples

Convert a photo for full-screen GBC display (APA mode):
```
gb-image-fx photo.jpg --quantize 160x144 --tiles-only -o res/photo
```

Convert with fewer palettes:
```
gb-image-fx photo.png --quantize 160x144 --max-palettes 4 --tiles-only -o res/photo
```

Tune color sampling:
```
gb-image-fx photo.webp --quantize 160x144 --samples 640 --tiles-only -o res/photo
```

Pre-made tileset (already 4 colors, no quantize needed):
```
gb-image-fx tileset.png -o res/tileset
```
