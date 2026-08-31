# gb-image-fx

Convert images to Game Boy / Game Boy Color tile data. Reads PNG, JPEG, BMP, GIF, WebP, and more.

## Usage

```
gb-image-fx <input> [OPTIONS]
```

An image that is already GB-ready (dimensions a multiple of 8, at most 4 colors per tile) is converted directly. A full-color image needs `--quantize` to fit it to the hardware first.

## Options

| Option | Description |
|---|---|
| `-o <prefix>` | Output file prefix (default: input file stem) |
| `--quantize <WxH>` | Quantize a full-color image to `WxH` and up to 8 palettes |
| `--max-palettes <N>` | Palette limit for `--quantize` (default: 8, the GBC max) |
| `--dither [weight]` | Dither during quantize, weight 0.0-1.0 (default: 0.5) |
| `--dither-method <M>` | Dither pattern: `blue` (default), `bayer`, or `ordered` |
| `--gbc-correction` | Quantize and preview for the GBC LCD, so colors look right on hardware |
| `--obj` | Sprite mode: transparent pixels (alpha < 128) become color 0, opaque colors fill 1-3 |
| `--dmg` | One palette for the whole image; combine with `--obj` for sprites |
| `--metasprite <WxH>` | Emit tiles per sprite cell (e.g. `16x16`): cells row-major, each cell column-major for 8x16 OBJ pairs |
| `--tiles-only` | Output tiles and palettes only (no map/attributes) |
| `--keep-duplicates` | Keep duplicate tiles instead of deduplicating |
| `--no-flip` | Don't match flipped tiles when deduplicating |
| `--preview` | Write a PNG preview instead of the binary files |

## Output files

| File | Description |
|---|---|
| `{prefix}_tiles.bin` | 2bpp tile data, 16 bytes per tile |
| `{prefix}_palettes.bin` | RGB555 palettes, 8 bytes per palette |
| `{prefix}_attributes.bin` | Per-tile attributes (palette index, flip flags) |
| `{prefix}_map.bin` | Tile map, 1 byte per cell (omitted with `--tiles-only`) |

## Examples

Full-screen GBC background from a photo:

```
gb-image-fx photo.jpg --quantize 160x144 --tiles-only -o res/photo
```

Same, dithered:

```
gb-image-fx photo.jpg --quantize 160x144 --dither --tiles-only -o res/photo
```

A 4-direction top-down sprite sheet (16x16 cells):

```
gb-image-fx player.png --obj --dmg --metasprite 16x16 --tiles-only -o res/player
```

A tileset that is already 4 colors:

```
gb-image-fx tileset.png -o res/tileset
```

## Credits

The `--quantize` color reducer is a port of [tiledpalettequant](https://github.com/rilden/tiledpalettequant) by rilden (MIT).

`--gbc-correction` uses the CGB display model (Modern - Balanced) from [SameBoy](https://github.com/LIJI32/SameBoy) by Lior Halphon (MIT).
