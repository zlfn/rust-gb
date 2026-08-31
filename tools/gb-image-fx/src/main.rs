//! gb-image-fx — Convert images to Game Boy / Game Boy Color tile data.
//!
//! Outputs (binary mode):
//!   {name}_tiles.bin      — 2bpp tile data (16 bytes per tile)
//!   {name}_map.bin        — tile map (1 byte per grid cell)
//!   {name}_palettes.bin   — GBC RGB555 palettes (8 bytes per palette)
//!   {name}_attributes.bin — GBC tile attributes (1 byte per grid cell)
//!
//! With --preview: outputs a PNG image instead of binary files.

mod quantize;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, process};

use palette::{FromColor, Oklch, Srgb};

// ── GBC RGB555 color ────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Color555(u16);

impl Color555 {
    fn from_rgba(r: u8, g: u8, b: u8) -> Self {
        Self(((b as u16 >> 3) & 0x1f) << 10 | ((g as u16 >> 3) & 0x1f) << 5 | ((r as u16 >> 3) & 0x1f))
    }

    fn to_rgb8(self) -> [u8; 3] {
        let r5 = (self.0 & 0x1f) as u8;
        let g5 = ((self.0 >> 5) & 0x1f) as u8;
        let b5 = ((self.0 >> 10) & 0x1f) as u8;
        [(r5 << 3) | (r5 >> 2), (g5 << 3) | (g5 >> 2), (b5 << 3) | (b5 >> 2)]
    }

    fn to_le_bytes(self) -> [u8; 2] {
        self.0.to_le_bytes()
    }

    fn oklch_lightness(self) -> f32 {
        let r = ((self.0 & 0x1f) as f32) / 31.0;
        let g = (((self.0 >> 5) & 0x1f) as f32) / 31.0;
        let b = (((self.0 >> 10) & 0x1f) as f32) / 31.0;
        let srgb = Srgb::new(r, g, b);
        let oklch = Oklch::from_color(srgb);
        oklch.l
    }
}

// ── Tile ─────────────────────────────────────────────────────────────────────

/// An 8×8 tile stored as palette indices (0-3).
#[derive(Clone, PartialEq, Eq, Hash)]
struct Tile {
    pixels: [u8; 64], // row-major, 0-3
}

impl Tile {
    fn to_2bpp(&self) -> [u8; 16] {
        let mut data = [0u8; 16];
        for row in 0..8 {
            let mut lo = 0u8;
            let mut hi = 0u8;
            for col in 0..8 {
                let px = self.pixels[row * 8 + col];
                if px & 1 != 0 {
                    lo |= 1 << (7 - col);
                }
                if px & 2 != 0 {
                    hi |= 1 << (7 - col);
                }
            }
            data[row * 2] = lo;
            data[row * 2 + 1] = hi;
        }
        data
    }

    fn flip_x(&self) -> Tile {
        let mut pixels = [0u8; 64];
        for row in 0..8 {
            for col in 0..8 {
                pixels[row * 8 + col] = self.pixels[row * 8 + (7 - col)];
            }
        }
        Tile { pixels }
    }

    fn flip_y(&self) -> Tile {
        let mut pixels = [0u8; 64];
        for row in 0..8 {
            pixels[row * 8..row * 8 + 8].copy_from_slice(&self.pixels[(7 - row) * 8..(7 - row) * 8 + 8]);
        }
        Tile { pixels }
    }

    fn flip_xy(&self) -> Tile {
        self.flip_x().flip_y()
    }
}

// ── Image ───────────────────────────────────────────────────────────────────

struct Image {
    width: u32,
    height: u32,
    pixels: Vec<[u8; 4]>, // RGBA
}

impl Image {
    fn load(path: &Path) -> Self {
        let img = image::open(path).unwrap_or_else(|e| {
            eprintln!("error: cannot open '{}': {}", path.display(), e);
            process::exit(1);
        });

        let width = img.width();
        let height = img.height();
        let rgba = img.to_rgba8();

        // Alpha is preserved: pixels below ALPHA_OPAQUE are treated as transparent
        // and map to color index 0 (see `is_transparent` / the `--obj` path).
        let pixels: Vec<[u8; 4]> = rgba.pixels()
            .map(|p| [p[0], p[1], p[2], p[3]])
            .collect();

        Image { width, height, pixels }
    }

    fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        self.pixels[(y * self.width + x) as usize]
    }
}

/// Alpha at or above this counts as opaque; below it, the pixel is transparent
/// and maps to color index 0.
const ALPHA_OPAQUE: u8 = 128;

fn is_transparent(px: [u8; 4]) -> bool {
    px[3] < ALPHA_OPAQUE
}

/// Build a padded 4-entry palette from a set of opaque colors, sorted brightest
/// first (index 0 is the lightest). When `obj` is set, index 0 is reserved as the
/// transparent slot and the opaque colors fill indices 1..=3.
fn finalize_palette(opaque: &[Color555], obj: bool) -> [Color555; 4] {
    let mut sorted = opaque.to_vec();
    sorted.sort_by(|a, b| b.oklch_lightness().partial_cmp(&a.oklch_lightness()).unwrap());
    let mut out = [Color555(0); 4];
    let start = usize::from(obj);
    for (i, &c) in sorted.iter().enumerate() {
        if start + i < 4 {
            out[start + i] = c;
        }
    }
    out
}

/// The color index of one pixel within `pal`. Transparent pixels are index 0; an
/// opaque pixel is matched against the opaque slots (which start at 1 when `obj`).
fn pixel_index(px: [u8; 4], pal: &[Color555; 4], obj: bool) -> u8 {
    if obj && is_transparent(px) {
        return 0;
    }
    let c = Color555::from_rgba(px[0], px[1], px[2]);
    let start = usize::from(obj);
    if let Some(i) = pal[start..].iter().position(|&p| p == c) {
        (start + i) as u8
    } else {
        // Not an exact palette member (e.g. a stray color): nearest opaque slot.
        let cl = c.oklch_lightness();
        pal.iter()
            .enumerate()
            .skip(start)
            .min_by(|(_, a), (_, b)| {
                (a.oklch_lightness() - cl)
                    .abs()
                    .partial_cmp(&(b.oklch_lightness() - cl).abs())
                    .unwrap()
            })
            .map(|(i, _)| i as u8)
            .unwrap_or(0)
    }
}

// ── Palette assignment ──────────────────────────────────────────────────────

/// The number of opaque colors a palette can hold: 4 normally, or 3 when `obj`
/// reserves index 0 for transparency.
fn opaque_capacity(obj: bool) -> usize {
    if obj { 3 } else { 4 }
}

/// DMG single-palette assignment: one palette of ≤4 (≤3 with `obj`) shades over
/// the whole image, every tile using it. Errors if the image has too many opaque
/// colors (reduce it first, or use `--quantize`).
fn assign_palette_dmg(img: &Image, obj: bool, tile_count: usize) -> (Vec<[Color555; 4]>, Vec<u8>) {
    let mut colors: Vec<Color555> = Vec::new();
    for &px in &img.pixels {
        if obj && is_transparent(px) {
            continue;
        }
        let c = Color555::from_rgba(px[0], px[1], px[2]);
        if !colors.contains(&c) {
            colors.push(c);
        }
    }
    let cap = opaque_capacity(obj);
    if colors.len() > cap {
        eprintln!(
            "error: --dmg image has {} opaque colors (max {}{}). Reduce it first or use --quantize.",
            colors.len(),
            cap,
            if obj { " with --obj, since index 0 is transparent" } else { "" }
        );
        process::exit(1);
    }
    (vec![finalize_palette(&colors, obj)], vec![0; tile_count])
}

/// Collect unique colors per tile, group tiles into palettes (max 4 colors each,
/// or 3 opaque + transparent with `obj`). Returns (palettes, tile_palette_assignment).
fn assign_palettes(
    img: &Image,
    tiles_w: u32,
    tiles_h: u32,
    obj: bool,
) -> (Vec<[Color555; 4]>, Vec<u8>) {
    let cap = opaque_capacity(obj);

    // Collect unique opaque colors per tile (transparent pixels take index 0)
    let mut tile_colors: Vec<Vec<Color555>> = Vec::new();
    for ty in 0..tiles_h {
        for tx in 0..tiles_w {
            let mut colors = Vec::new();
            for py in 0..8 {
                for px in 0..8 {
                    let p = img.get_pixel(tx * 8 + px, ty * 8 + py);
                    if obj && is_transparent(p) {
                        continue;
                    }
                    let c = Color555::from_rgba(p[0], p[1], p[2]);
                    if !colors.contains(&c) {
                        colors.push(c);
                    }
                }
            }
            if colors.len() > cap {
                eprintln!("error: tile ({},{}) has {} opaque colors (max {})", tx, ty, colors.len(), cap);
                process::exit(1);
            }
            tile_colors.push(colors);
        }
    }

    // Greedy palette assignment: try to fit each tile's colors into an existing palette
    let mut palettes: Vec<Vec<Color555>> = Vec::new();
    let mut tile_pal: Vec<u8> = Vec::new();

    for colors in &tile_colors {
        // Try to find an existing palette that can fit these colors
        let mut assigned = None;
        for (pi, pal) in palettes.iter().enumerate() {
            let mut merged = pal.clone();
            for c in colors {
                if !merged.contains(c) {
                    merged.push(*c);
                }
            }
            if merged.len() <= cap {
                assigned = Some(pi);
                break;
            }
        }

        if let Some(pi) = assigned {
            // Merge colors into existing palette
            for c in colors {
                if !palettes[pi].contains(c) {
                    palettes[pi].push(*c);
                }
            }
            tile_pal.push(pi as u8);
        } else {
            // New palette needed
            if palettes.len() >= 8 {
                eprintln!("error: image requires more than 8 palettes (too many unique color combinations)");
                process::exit(1);
            }
            palettes.push(colors.clone());
            tile_pal.push((palettes.len() - 1) as u8);
        }
    }

    // Pad to 4 entries, sorted brightest-first; reserve index 0 when obj.
    let palettes: Vec<[Color555; 4]> = palettes.iter().map(|pal| finalize_palette(pal, obj)).collect();

    (palettes, tile_pal)
}

// ── Tile extraction & dedup ─────────────────────────────────────────────────

struct TileMapEntry {
    tile_idx: u16,
    palette: u8,
    flip_x: bool,
    flip_y: bool,
}

impl TileMapEntry {
    fn attribute_byte(&self) -> u8 {
        let mut attr = self.palette & 0x07;
        if self.flip_x {
            attr |= 0x20;
        }
        if self.flip_y {
            attr |= 0x40;
        }
        attr
    }
}

/// Emit tile coordinates in the order tiles should be laid out.
///
/// Without `metasprite`: whole-image row-major (background/tilemap order).
/// With `metasprite (cw, ch)` in pixels: cells row-major, and within each cell
/// column-major, so a 16×16 cell in 8×16 OBJ mode yields
/// `[top-left, bottom-left, top-right, bottom-right]` — sprite 0 = the first
/// pair, sprite 1 = the second.
fn tile_order(tiles_w: u32, tiles_h: u32, metasprite: Option<(u32, u32)>) -> Vec<(u32, u32)> {
    match metasprite {
        None => {
            let mut v = Vec::with_capacity((tiles_w * tiles_h) as usize);
            for ty in 0..tiles_h {
                for tx in 0..tiles_w {
                    v.push((tx, ty));
                }
            }
            v
        }
        Some((cw, ch)) => {
            let ctw = cw / 8;
            let cth = ch / 8;
            let cells_w = tiles_w / ctw;
            let cells_h = tiles_h / cth;
            let mut v = Vec::with_capacity((tiles_w * tiles_h) as usize);
            for cy in 0..cells_h {
                for cx in 0..cells_w {
                    for lx in 0..ctw {
                        for ly in 0..cth {
                            v.push((cx * ctw + lx, cy * cth + ly));
                        }
                    }
                }
            }
            v
        }
    }
}

fn extract_tiles(
    img: &Image,
    palettes: &[[Color555; 4]],
    tile_pal: &[u8],
    tiles_w: u32,
    order: &[(u32, u32)],
    obj: bool,
    keep_duplicates: bool,
    detect_flips: bool,
) -> (Vec<Tile>, Vec<TileMapEntry>) {
    let mut unique_tiles: Vec<Tile> = Vec::new();
    let mut tile_map_entries: Vec<TileMapEntry> = Vec::new();
    let mut tile_lookup: HashMap<Tile, usize> = HashMap::new();

    for &(tx, ty) in order {
        {
            let ti = (ty * tiles_w + tx) as usize;
            let pal = &palettes[tile_pal[ti] as usize];

            // Extract tile pixels as palette indices
            let mut pixels = [0u8; 64];
            for py in 0..8u32 {
                for px in 0..8u32 {
                    let p = img.get_pixel(tx * 8 + px, ty * 8 + py);
                    pixels[(py * 8 + px) as usize] = pixel_index(p, pal, obj);
                }
            }
            let tile = Tile { pixels };

            if keep_duplicates {
                let idx = unique_tiles.len();
                unique_tiles.push(tile);
                tile_map_entries.push(TileMapEntry {
                    tile_idx: idx as u16,
                    palette: tile_pal[ti],
                    flip_x: false,
                    flip_y: false,
                });
            } else {
                // Try to find match (original, flipped)
                let mut found = None;

                if let Some(&idx) = tile_lookup.get(&tile) {
                    found = Some((idx, false, false));
                }

                if found.is_none() && detect_flips {
                    let fx = tile.flip_x();
                    if let Some(&idx) = tile_lookup.get(&fx) {
                        found = Some((idx, true, false));
                    }
                    if found.is_none() {
                        let fy = tile.flip_y();
                        if let Some(&idx) = tile_lookup.get(&fy) {
                            found = Some((idx, false, true));
                        }
                        if found.is_none() {
                            let fxy = tile.flip_xy();
                            if let Some(&idx) = tile_lookup.get(&fxy) {
                                found = Some((idx, true, true));
                            }
                        }
                    }
                }

                if let Some((idx, fx, fy)) = found {
                    tile_map_entries.push(TileMapEntry {
                        tile_idx: idx as u16,
                        palette: tile_pal[ti],
                        flip_x: fx,
                        flip_y: fy,
                    });
                } else {
                    let idx = unique_tiles.len();
                    if idx == 256 {
                        eprintln!("warning: tile count exceeds 256 (overflows u8 map index)");
                    }
                    tile_lookup.insert(tile.clone(), idx);
                    unique_tiles.push(tile);
                    tile_map_entries.push(TileMapEntry {
                        tile_idx: idx as u16,
                        palette: tile_pal[ti],
                        flip_x: false,
                        flip_y: false,
                    });
                }
            }
        }
    }

    (unique_tiles, tile_map_entries)
}

// ── CLI & main ──────────────────────────────────────────────────────────────

struct Options {
    input: PathBuf,
    output_prefix: PathBuf,
    keep_duplicates: bool,
    tiles_only: bool,
    detect_flips: bool,
    quantize: Option<(u32, u32)>, // target resolution (w, h)
    max_palettes: usize,
    preview: bool,
    dither: Option<(f64, quantize::DitherMethod)>, // (weight, method) from --dither / --dither-method
    obj: bool,                     // OBJ/sprite mode: color index 0 is transparent
    dmg: bool,                     // DMG mode: one 2bpp palette for the whole image
    metasprite: Option<(u32, u32)>, // sprite cell size in pixels (column-major tile order)
    gbc_correction: bool,          // quantize and preview for the GBC LCD's colors
}

const HELP: &str = "\
gb-image-fx — Convert images to Game Boy / Game Boy Color tile data

USAGE:
    gb-image-fx <input> [OPTIONS]

OPTIONS:
    -o <prefix>           Output file prefix (default: input file stem)
    --obj                 OBJ/sprite mode: transparent pixels (alpha < 128) become
                          color index 0, and opaque colors fill indices 1-3 (max 3)
    --dmg                 DMG mode: one palette for the whole image (no per-8x8
                          palette grouping). Combine with --obj for sprites.
    --metasprite <WxH>    Emit tiles per sprite cell (e.g. 16x16): cells row-major,
                          each cell column-major (8x16 OBJ pairs)
    --keep-duplicates     Don't deduplicate tiles
    --tiles-only          Only output tiles and palettes (no map/attributes)
    --no-flip             Don't detect flipped tiles during dedup
    --quantize <WxH>      Quantize a full-color image to target resolution
                          (e.g. 160x144) and up to 8 GBC palettes
    --max-palettes <N>    Maximum palettes for quantize (default: 8, GBC max)
    --preview             Output a PNG preview image instead of GBC binary files
    --dither [weight]     Dither during quantize; weight ~0.0-1.0 (default: 0.5)
    --dither-method <M>   Dither pattern: 'blue' (default), 'bayer', or 'ordered'
    --gbc-correction      Quantize (and preview) for the GBC LCD's washed-out
                          colors, so the result looks right on real hardware
    -h, --help            Show this help
";

fn parse_args() -> Options {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        eprint!("{}", HELP);
        process::exit(0);
    }

    let keep_duplicates = args.contains("--keep-duplicates");
    let tiles_only = args.contains("--tiles-only");
    let preview = args.contains("--preview");
    let obj = args.contains("--obj");
    let dmg = args.contains("--dmg");
    let metasprite: Option<(u32, u32)> = args
        .opt_value_from_fn("--metasprite", parse_wxh)
        .unwrap_or(None);
    let dither_weight: Option<f64> = match args.opt_value_from_str("--dither") {
        Ok(v) => v,
        Err(_) => {
            // --dither flag present but no value → use default weight 0.5
            args.contains("--dither");
            Some(0.5)
        }
    };
    let dither_method = match args.opt_value_from_str::<_, String>("--dither-method").unwrap_or(None).as_deref() {
        Some("bayer") => quantize::DitherMethod::Bayer,
        Some("ordered") => quantize::DitherMethod::Ordered,
        _ => quantize::DitherMethod::Blue,
    };
    let dither = dither_weight.map(|w| (w, dither_method));
    // DMG backgrounds have no tile attributes, so they can't flip; disable flip
    // dedup there (sprites flip at runtime through OAM, not through the tile map).
    let detect_flips = !args.contains("--no-flip") && !dmg;
    let gbc_correction = args.contains("--gbc-correction");
    let output_prefix: Option<PathBuf> = args.opt_value_from_str("-o").unwrap_or(None);

    let quantize: Option<(u32, u32)> = args
        .opt_value_from_fn("--quantize", parse_wxh)
        .unwrap_or(None);
    let max_palettes: usize = args
        .opt_value_from_str("--max-palettes")
        .unwrap_or(None)
        .unwrap_or(8);

    let input: PathBuf = args.free_from_str().unwrap_or_else(|_| {
        eprint!("{}", HELP);
        process::exit(1);
    });

    let remaining = args.finish();
    if !remaining.is_empty() {
        eprintln!("unknown arguments: {:?}", remaining);
        process::exit(1);
    }

    let output_prefix = output_prefix.unwrap_or_else(|| {
        PathBuf::from(input.file_stem().unwrap().to_string_lossy().as_ref())
    });

    Options {
        input,
        output_prefix,
        keep_duplicates: keep_duplicates || tiles_only,
        tiles_only,
        detect_flips,
        quantize,
        max_palettes,
        preview,
        dither,
        obj,
        dmg,
        metasprite,
        gbc_correction,
    }
}

// SameBoy's CGB colour correction (Modern - Balanced), the model behind accurate
// GBC displays. Per-channel response curve, then a touch of blue mixed into green.
// Ported from SameBoy's Core/display.c (MIT, Copyright (c) 2015-2024 Lior Halphon).
const CGB_CURVE: [u8; 32] = [
    0, 6, 12, 20, 28, 36, 45, 56, 66, 76, 88, 100, 113, 125, 137, 149, 161, 172, 182, 192, 202,
    210, 218, 225, 232, 238, 243, 247, 250, 252, 254, 255,
];

/// The colour an accurate GBC display shows for an RGB555 value.
fn gbc_correct(rgb: [u8; 3]) -> [u8; 3] {
    let r = CGB_CURVE[(rgb[0] >> 3) as usize];
    let g = CGB_CURVE[(rgb[1] >> 3) as usize];
    let b = CGB_CURVE[(rgb[2] >> 3) as usize];
    let ng = if g != b {
        let gamma = 1.6;
        let mix = ((g as f64 / 255.0).powf(gamma) * 3.0 + (b as f64 / 255.0).powf(gamma)) / 4.0;
        (mix.powf(1.0 / gamma) * 255.0).round() as u8
    } else {
        g
    };
    [r, ng, b]
}

/// Pre-compensate every pixel so that after quantization and display it lands back
/// on the source colour: pick the RGB555 value whose corrected appearance is
/// closest. Correction is separable — red comes from its channel alone, while
/// green and blue are coupled (green mixes in some blue) — so the nearest value is
/// found channel-wise instead of scanning all 32768. Colours the panel can't reach
/// are approximated.
fn gbc_precompensate(pixels: &mut [[u8; 4]]) {
    let expand = |c5: usize| ((c5 << 3) | (c5 >> 2)) as u8;
    // Corrected green for every (blue5, green5) pair.
    let mut green = [[0u8; 32]; 32];
    for (b5, row) in green.iter_mut().enumerate() {
        for (g5, cell) in row.iter_mut().enumerate() {
            *cell = gbc_correct([0, expand(g5), expand(b5)])[1];
        }
    }
    let mut cache: HashMap<[u8; 3], [u8; 3]> = HashMap::new();
    for p in pixels.iter_mut() {
        let key = [p[0] as i32, p[1] as i32, p[2] as i32];
        let comp = *cache.entry([p[0], p[1], p[2]]).or_insert_with(|| {
            // Red: nearest curve entry on its own.
            let r5 = (0..32)
                .min_by_key(|&c| (CGB_CURVE[c] as i32 - key[0]).pow(2))
                .unwrap();
            // Blue and green together, since blue shifts the green output too.
            let (mut best, mut bd) = ((0usize, 0usize), i32::MAX);
            for b5 in 0..32 {
                let db = (CGB_CURVE[b5] as i32 - key[2]).pow(2);
                for g5 in 0..32 {
                    let d = db + (green[b5][g5] as i32 - key[1]).pow(2);
                    if d < bd {
                        bd = d;
                        best = (b5, g5);
                    }
                }
            }
            [expand(r5), expand(best.1), expand(best.0)]
        });
        *p = [comp[0], comp[1], comp[2], p[3]];
    }
}

/// Metasprite cell size in pixels must be a multiple of 8 and divide the image.
fn validate_metasprite(cell: (u32, u32), img_w: u32, img_h: u32) {
    let (cw, ch) = cell;
    if cw % 8 != 0 || ch % 8 != 0 {
        eprintln!("error: --metasprite {}x{} must be multiples of 8", cw, ch);
        process::exit(1);
    }
    if img_w % cw != 0 || img_h % ch != 0 {
        eprintln!(
            "error: --metasprite {}x{} does not divide the {}x{} image",
            cw, ch, img_w, img_h
        );
        process::exit(1);
    }
}

fn parse_wxh(s: &str) -> Result<(u32, u32), String> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() != 2 {
        return Err(format!("expected WxH format, got '{s}'"));
    }
    let w = parts[0].parse::<u32>().map_err(|e| e.to_string())?;
    let h = parts[1].parse::<u32>().map_err(|e| e.to_string())?;
    if w == 0 || h == 0 {
        return Err("dimensions must be non-zero".into());
    }
    Ok((w, h))
}

fn main() {
    let opts = parse_args();

    // Correction pre-compensates source colors before clustering, so it only has
    // meaning together with --quantize.
    if opts.gbc_correction && opts.quantize.is_none() {
        eprintln!("error: --gbc-correction requires --quantize");
        process::exit(1);
    }

    let mut img = Image::load(&opts.input);

    // Quantize: box-sample to target, then tile-aware palette clustering
    let quantize_result = if let Some((tw, th)) = opts.quantize {
        if tw % 8 != 0 || th % 8 != 0 {
            eprintln!("error: --quantize dimensions must be multiples of 8");
            process::exit(1);
        }

        eprintln!("Quantizing {}×{} → {}×{}", img.width, img.height, tw, th);
        if img.width != tw || img.height != th {
            img.pixels = quantize::box_sample(&img.pixels, img.width, img.height, tw, th, opts.obj);
            img.width = tw;
            img.height = th;
        }

        // Pre-compensate so the panel's darkening lands back on the source colors.
        if opts.gbc_correction {
            gbc_precompensate(&mut img.pixels);
        }

        // DMG uses a single palette for the whole image.
        let max_pals = if opts.dmg { 1 } else { opts.max_palettes };
        let (quantized, palettes, tile_pal) = quantize::quantize_image_tiled(
            &img.pixels, img.width, img.height, max_pals, opts.dither, opts.obj,
        );
        img.pixels = quantized;

        // Convert palettes: Vec<Vec<[u8;3]>> → Vec<[Color555;4]>. In OBJ mode the
        // opaque colors start at index 1, leaving index 0 for transparency.
        let start = usize::from(opts.obj);
        let palettes: Vec<[Color555; 4]> = palettes.iter().map(|pal| {
            let mut out = [Color555(0); 4];
            let mut sorted: Vec<Color555> = pal.iter()
                .map(|c| Color555::from_rgba(c[0], c[1], c[2]))
                .collect();
            sorted.sort_by(|a, b| b.oklch_lightness().partial_cmp(&a.oklch_lightness()).unwrap());
            for (i, &c) in sorted.iter().enumerate().take(4 - start) {
                out[start + i] = c;
            }
            out
        }).collect();

        Some((palettes, tile_pal))
    } else {
        None
    };

    if img.width % 8 != 0 || img.height % 8 != 0 {
        eprintln!("error: image dimensions {}×{} must be multiples of 8 (use --quantize WxH)", img.width, img.height);
        process::exit(1);
    }

    if let Some(cell) = opts.metasprite {
        validate_metasprite(cell, img.width, img.height);
    }

    let tiles_w = img.width / 8;
    let tiles_h = img.height / 8;
    let tile_count = (tiles_w * tiles_h) as usize;
    eprintln!(
        "{}×{} image → {}×{} tiles ({} total)",
        img.width, img.height, tiles_w, tiles_h, tile_count
    );

    // Palette assignment: quantize result, DMG single palette, or per-tile derive.
    let (palettes, tile_pal) = if let Some(qr) = quantize_result {
        qr
    } else if opts.dmg {
        assign_palette_dmg(&img, opts.obj, tile_count)
    } else {
        assign_palettes(&img, tiles_w, tiles_h, opts.obj)
    };
    eprintln!("{} palette(s) detected", palettes.len());

    // Phase 2: Tile extraction & dedup, in layout order (metasprite-aware).
    let order = tile_order(tiles_w, tiles_h, opts.metasprite);
    let (tiles, map) = extract_tiles(
        &img, &palettes, &tile_pal, tiles_w, &order, opts.obj,
        opts.keep_duplicates, opts.detect_flips,
    );
    eprintln!("{} unique tile(s) (from {} total)", tiles.len(), tile_count);

    if opts.preview {
        // Reconstruct image from tiles + palettes + map
        let img_w = (tiles_w * 8) as u32;
        let img_h = (tiles_h * 8) as u32;
        let mut out_buf = vec![0u8; (img_w * img_h * 3) as usize];

        for (i, entry) in map.iter().enumerate() {
            // Map entries follow `order`, so place each at its source grid cell.
            let (grid_x, grid_y) = order[i];
            let tile = &tiles[entry.tile_idx as usize];
            let pal = &palettes[entry.palette as usize];

            for py in 0..8u32 {
                for px in 0..8u32 {
                    let src_px = if entry.flip_x { 7 - px } else { px };
                    let src_py = if entry.flip_y { 7 - py } else { py };
                    let color_idx = tile.pixels[(src_py * 8 + src_px) as usize];
                    // In OBJ mode index 0 is transparent; show it as magenta.
                    let [r, g, b] = if opts.obj && color_idx == 0 {
                        [255, 0, 255]
                    } else {
                        let rgb = pal[color_idx as usize].to_rgb8();
                        if opts.gbc_correction { gbc_correct(rgb) } else { rgb }
                    };

                    let out_x = grid_x * 8 + px;
                    let out_y = grid_y * 8 + py;
                    let offset = ((out_y * img_w + out_x) * 3) as usize;
                    out_buf[offset] = r;
                    out_buf[offset + 1] = g;
                    out_buf[offset + 2] = b;
                }
            }
        }

        let preview_path = format!("{}_preview.png", opts.output_prefix.display());
        let out_img = image::RgbImage::from_raw(img_w, img_h, out_buf).unwrap();
        out_img.save(&preview_path).unwrap();
        eprintln!("  {} ({}×{})", preview_path, img_w, img_h);
    } else {
        // Output tiles
        let tiles_path = format!("{}_tiles.bin", opts.output_prefix.display());
        let tiles_data: Vec<u8> = tiles.iter().flat_map(|t| t.to_2bpp()).collect();
        fs::write(&tiles_path, &tiles_data).unwrap();
        eprintln!("  {} ({} bytes)", tiles_path, tiles_data.len());

        // Output palettes
        let pal_path = format!("{}_palettes.bin", opts.output_prefix.display());
        let pal_data: Vec<u8> = palettes.iter()
            .flat_map(|p| p.iter().flat_map(|c| c.to_le_bytes()))
            .collect();
        fs::write(&pal_path, &pal_data).unwrap();
        eprintln!("  {} ({} bytes, {} palette(s))", pal_path, pal_data.len(), palettes.len());

        // Output GBC attributes (always — needed for multi-palette CGB display)
        let attr_path = format!("{}_attributes.bin", opts.output_prefix.display());
        let attr_data: Vec<u8> = map.iter().map(|e| e.attribute_byte()).collect();
        fs::write(&attr_path, &attr_data).unwrap();
        eprintln!("  {} ({} bytes)", attr_path, attr_data.len());

        if !opts.tiles_only {
            // Output tile map
            let map_path = format!("{}_map.bin", opts.output_prefix.display());
            let map_data: Vec<u8> = map.iter().map(|e| e.tile_idx as u8).collect();
            fs::write(&map_path, &map_data).unwrap();
            eprintln!("  {} ({} bytes)", map_path, map_data.len());
        }
    }
}
