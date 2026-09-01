//! Convert images to Game Boy / Game Boy Color tile data.
//!
//! [`convert`] takes RGBA pixels and a [`Config`] and returns a [`Converted`],
//! which hands out the tile data, palettes, attributes and tile map as bytes, or
//! renders a preview. Nothing here touches the filesystem, so the same code runs
//! behind the command line and in a browser.

pub mod quantize;

#[cfg(feature = "wasm")]
mod wasm;

use std::collections::HashMap;
use std::fmt;

use palette::{FromColor, Oklch, Srgb};

pub use quantize::DitherMethod;

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

/// An RGBA image the conversion works on.
pub struct Image {
    width: u32,
    height: u32,
    pixels: Vec<[u8; 4]>, // RGBA
}

impl Image {
    /// Wrap raw RGBA pixels, row-major, `width * height` of them.
    pub fn from_rgba(width: u32, height: u32, pixels: Vec<[u8; 4]>) -> Self {
        assert_eq!(pixels.len(), (width * height) as usize, "pixel count must match the dimensions");
        Image { width, height, pixels }
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        self.pixels[(y * self.width + x) as usize]
    }
}

/// The four shades a DMG's screen shows, lightest first, matching the index
/// order tiles are written in.
const DMG_SHADES: [[u8; 3]; 4] = [
    [0x9b, 0xbc, 0x0f],
    [0x8b, 0xac, 0x0f],
    [0x30, 0x62, 0x30],
    [0x0f, 0x38, 0x0f],
];

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
fn assign_palette_dmg(img: &Image, obj: bool, tile_count: usize) -> Result<(Vec<[Color555; 4]>, Vec<u8>), Error> {
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
        return Err(Error::TooManyDmgColors { found: colors.len(), max: cap, obj });
    }
    Ok((vec![finalize_palette(&colors, obj)], vec![0; tile_count]))
}

/// Collect unique colors per tile, group tiles into palettes (max 4 colors each,
/// or 3 opaque + transparent with `obj`). Returns (palettes, tile_palette_assignment).
fn assign_palettes(
    img: &Image,
    tiles_w: u32,
    tiles_h: u32,
    obj: bool,
) -> Result<(Vec<[Color555; 4]>, Vec<u8>), Error> {
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
                return Err(Error::TileTooManyColors { x: tx, y: ty, found: colors.len(), max: cap });
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
                return Err(Error::TooManyPalettes);
            }
            palettes.push(colors.clone());
            tile_pal.push((palettes.len() - 1) as u8);
        }
    }

    // Pad to 4 entries, sorted brightest-first; reserve index 0 when obj.
    let palettes: Vec<[Color555; 4]> = palettes.iter().map(|pal| finalize_palette(pal, obj)).collect();

    Ok((palettes, tile_pal))
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


// ── Colour correction & grayscale ───────────────────────────────────────────

/// Turn the image grayscale the way the DMG will show it, preserving the contrast
/// between colours a plain luminance would map to the same shade. In OBJ mode the
/// alpha goes along, so the colour behind transparent pixels stays out of the fit;
/// elsewhere alpha carries no meaning and every pixel counts.
fn decolorize_pixels(pixels: &mut [[u8; 4]], width: u32, height: u32, obj: bool) {
    let rgba = image::RgbaImage::from_fn(width, height, |x, y| {
        let p = pixels[(y * width + x) as usize];
        image::Rgba([p[0], p[1], p[2], if obj { p[3] } else { 255 }])
    });
    let gray = decolorize::decolorize(&rgba);
    for (p, g) in pixels.iter_mut().zip(gray.pixels()) {
        *p = [g[0], g[0], g[0], p[3]];
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
/// closest. Red is corrected on its own, while green takes in some blue, so the
/// search runs per channel rather than over all 32768. Colours the panel can't
/// reach are approximated.
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
fn validate_metasprite(cell: (u32, u32), img_w: u32, img_h: u32) -> Result<(), Error> {
    let (cw, ch) = cell;
    if cw % 8 != 0 || ch % 8 != 0 {
        return Err(Error::MetaspriteNotMultipleOf8 { cell });
    }
    if img_w % cw != 0 || img_h % ch != 0 {
        return Err(Error::MetaspriteDoesNotDivide { cell, image: (img_w, img_h) });
    }
    Ok(())
}


// ── Public API ──────────────────────────────────────────────────────────────

/// What to make of the image.
#[derive(Clone, Debug)]
pub struct Config {
    /// Sprite mode: transparent pixels become colour 0, opaque ones fill 1-3.
    pub obj: bool,
    /// One palette for the whole image, and no palette or attribute output.
    pub dmg: bool,
    /// Reduce a full-colour image to this size and to `max_palettes` palettes.
    pub quantize: Option<(u32, u32)>,
    /// Palette ceiling for `quantize`.
    pub max_palettes: usize,
    /// Dither weight and pattern to use while quantizing.
    pub dither: Option<(f64, DitherMethod)>,
    /// Pre-compensate colours for the Game Boy Color's LCD.
    pub gbc_correction: bool,
    /// Sprite cell size in pixels, which reorders the tiles cell by cell.
    pub metasprite: Option<(u32, u32)>,
    /// Fold identical tiles together.
    pub dedup: bool,
    /// Fold tiles that match once mirrored.
    pub flip: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            obj: false,
            dmg: false,
            quantize: None,
            max_palettes: 8,
            dither: None,
            gbc_correction: false,
            metasprite: None,
            dedup: false,
            flip: false,
        }
    }
}

impl Config {
    /// Reject the combinations that cannot produce usable output.
    pub fn validate(&self) -> Result<(), Error> {
        if self.flip && !self.dedup {
            return Err(Error::FlipWithoutDedup);
        }
        if self.flip && self.dmg {
            return Err(Error::FlipWithDmg);
        }
        if self.gbc_correction && self.quantize.is_none() {
            return Err(Error::CorrectionWithoutQuantize);
        }
        if self.gbc_correction && self.dmg {
            return Err(Error::CorrectionWithDmg);
        }
        Ok(())
    }
}

/// Why a conversion could not be made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    DedupWithoutMap,
    FlipWithoutDedup,
    FlipWithDmg,
    CorrectionWithoutQuantize,
    CorrectionWithDmg,
    QuantizeNotMultipleOf8 { size: (u32, u32) },
    SizeNotMultipleOf8 { size: (u32, u32) },
    MetaspriteNotMultipleOf8 { cell: (u32, u32) },
    MetaspriteDoesNotDivide { cell: (u32, u32), image: (u32, u32) },
    TooManyDmgColors { found: usize, max: usize, obj: bool },
    TileTooManyColors { x: u32, y: u32, found: usize, max: usize },
    TooManyPalettes,
    TooManyTiles { found: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DedupWithoutMap => {
                write!(f, "--dedup needs --map to record where each tile went")
            }
            Error::FlipWithoutDedup => write!(f, "--flip needs --dedup"),
            Error::FlipWithDmg => {
                write!(f, "--flip needs tile attributes, which --dmg does not emit")
            }
            Error::CorrectionWithoutQuantize => write!(f, "--gbc-correction requires --quantize"),
            Error::CorrectionWithDmg => {
                write!(f, "--gbc-correction has no effect with --dmg, which emits no palette")
            }
            Error::QuantizeNotMultipleOf8 { .. } => {
                write!(f, "--quantize dimensions must be multiples of 8")
            }
            Error::SizeNotMultipleOf8 { size: (w, h) } => write!(
                f,
                "image dimensions {}×{} must be multiples of 8 (use --quantize WxH)",
                w, h
            ),
            Error::MetaspriteNotMultipleOf8 { cell: (w, h) } => {
                write!(f, "--metasprite {}x{} must be multiples of 8", w, h)
            }
            Error::MetaspriteDoesNotDivide { cell: (cw, ch), image: (w, h) } => write!(
                f,
                "--metasprite {}x{} does not divide the {}x{} image",
                cw, ch, w, h
            ),
            Error::TooManyDmgColors { found, max, obj } => write!(
                f,
                "--dmg image has {} opaque colors (max {}{}). Reduce it first or use --quantize.",
                found,
                max,
                if *obj { " with --obj, since index 0 is transparent" } else { "" }
            ),
            Error::TileTooManyColors { x, y, found, max } => write!(
                f,
                "tile ({},{}) has {} opaque colors (max {})",
                x, y, found, max
            ),
            Error::TooManyPalettes => write!(
                f,
                "image requires more than 8 palettes (too many unique color combinations)"
            ),
            Error::TooManyTiles { found } => write!(
                f,
                "{} unique tiles, but a tile map byte only addresses 256.\n\
                 Reduce the tile count (smaller --quantize, or drop --dither), or drop\n\
                 --map and place the tiles yourself.",
                found
            ),
        }
    }
}

impl std::error::Error for Error {}

/// What the conversion found along the way, for callers that report progress.
#[derive(Clone, Copy, Debug)]
pub struct Stats {
    /// Image size once quantizing has resized it.
    pub size: (u32, u32),
    /// Size of the tile grid.
    pub grid: (u32, u32),
    /// Cells in the grid.
    pub total_tiles: usize,
    /// Tiles actually written, after any folding.
    pub unique_tiles: usize,
    /// Palettes the image needs.
    pub palettes: usize,
    /// Residual error left by quantizing, when it ran.
    pub quantize_error: Option<f64>,
}

/// A finished conversion, holding everything the output files are made of.
pub struct Converted {
    tiles: Vec<Tile>,
    palettes: Vec<[Color555; 4]>,
    map: Vec<TileMapEntry>,
    order: Vec<(u32, u32)>,
    config: Config,
    stats: Stats,
}

impl Converted {
    /// What the conversion found along the way.
    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// 2bpp tile data, 16 bytes per tile.
    pub fn tiles(&self) -> Vec<u8> {
        self.tiles.iter().flat_map(|t| t.to_2bpp()).collect()
    }

    /// RGB555 palettes, 8 bytes each. A DMG reads its shades from BGP instead.
    pub fn palettes(&self) -> Option<Vec<u8>> {
        if self.config.dmg {
            return None;
        }
        Some(self.palettes.iter().flat_map(|p| p.iter().flat_map(|c| c.to_le_bytes())).collect())
    }

    /// Per-cell attributes. A DMG background has no attribute map.
    pub fn attributes(&self) -> Option<Vec<u8>> {
        if self.config.dmg {
            return None;
        }
        Some(self.map.iter().map(|e| e.attribute_byte()).collect())
    }

    /// Per-cell tile indices. A map entry is one byte, so this fails once more
    /// tiles were kept than one can name; without a map the program places the
    /// tiles itself and can use as many as it can load.
    pub fn map(&self) -> Result<Vec<u8>, Error> {
        if self.tiles.len() > 256 {
            return Err(Error::TooManyTiles { found: self.tiles.len() });
        }
        Ok(self.map.iter().map(|e| e.tile_idx as u8).collect())
    }

    /// The image as the console would show it: RGBA, four bytes per pixel. In
    /// OBJ mode the transparent index comes back fully transparent.
    pub fn preview(&self) -> (u32, u32, Vec<u8>) {
        let (grid_w, grid_h) = self.stats.grid;
        let (img_w, img_h) = (grid_w * 8, grid_h * 8);
        let mut out = vec![0u8; (img_w * img_h * 4) as usize];

        for (i, entry) in self.map.iter().enumerate() {
            // Map entries follow `order`, so place each at its source grid cell.
            let (grid_x, grid_y) = self.order[i];
            let tile = &self.tiles[entry.tile_idx as usize];
            let pal = &self.palettes[entry.palette as usize];

            for py in 0..8u32 {
                for px in 0..8u32 {
                    let src_px = if entry.flip_x { 7 - px } else { px };
                    let src_py = if entry.flip_y { 7 - py } else { py };
                    let color_idx = tile.pixels[(src_py * 8 + src_px) as usize];
                    let rgba = if self.config.obj && color_idx == 0 {
                        // In OBJ mode index 0 is the transparent one.
                        [0, 0, 0, 0]
                    } else if self.config.dmg {
                        // A DMG only ever shows these four, picked by the index.
                        let s = DMG_SHADES[color_idx as usize];
                        [s[0], s[1], s[2], 255]
                    } else {
                        let rgb = pal[color_idx as usize].to_rgb8();
                        let rgb = if self.config.gbc_correction { gbc_correct(rgb) } else { rgb };
                        [rgb[0], rgb[1], rgb[2], 255]
                    };

                    let offset = (((grid_y * 8 + py) * img_w + grid_x * 8 + px) * 4) as usize;
                    out[offset..offset + 4].copy_from_slice(&rgba);
                }
            }
        }
        (img_w, img_h, out)
    }
}

/// Turn an image into Game Boy tile data.
pub fn convert(mut img: Image, config: &Config) -> Result<Converted, Error> {
    config.validate()?;

    let mut quantize_error = None;
    let quantized = if let Some((tw, th)) = config.quantize {
        if tw % 8 != 0 || th % 8 != 0 {
            return Err(Error::QuantizeNotMultipleOf8 { size: (tw, th) });
        }
        if img.width != tw || img.height != th {
            img.pixels = quantize::box_sample(&img.pixels, img.width, img.height, tw, th, config.obj);
            img.width = tw;
            img.height = th;
        }

        // Pre-compensate so the panel's darkening lands back on the source colors.
        if config.gbc_correction {
            gbc_precompensate(&mut img.pixels);
        }

        // A DMG shows four shades, so the image has to become grayscale. Decolorizing
        // for contrast keeps colours apart that differ in hue but not in brightness,
        // which a plain luminance would flatten into the same shade.
        if config.dmg {
            decolorize_pixels(&mut img.pixels, img.width, img.height, config.obj);
        }

        // DMG uses a single palette for the whole image.
        let max_pals = if config.dmg { 1 } else { config.max_palettes };
        let result = quantize::quantize_image_tiled(
            &img.pixels, img.width, img.height, max_pals, config.dither, config.obj,
        );
        img.pixels = result.pixels;
        quantize_error = Some(result.error);

        // In OBJ mode the opaque colors start at index 1, leaving 0 transparent.
        let start = usize::from(config.obj);
        let palettes: Vec<[Color555; 4]> = result.palettes.iter().map(|pal| {
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

        Some((palettes, result.tile_palettes))
    } else {
        None
    };

    if img.width % 8 != 0 || img.height % 8 != 0 {
        return Err(Error::SizeNotMultipleOf8 { size: (img.width, img.height) });
    }
    if let Some(cell) = config.metasprite {
        validate_metasprite(cell, img.width, img.height)?;
    }

    let tiles_w = img.width / 8;
    let tiles_h = img.height / 8;
    let tile_count = (tiles_w * tiles_h) as usize;

    let (palettes, tile_pal) = match quantized {
        Some(q) => q,
        None if config.dmg => assign_palette_dmg(&img, config.obj, tile_count)?,
        None => assign_palettes(&img, tiles_w, tiles_h, config.obj)?,
    };

    let order = tile_order(tiles_w, tiles_h, config.metasprite);
    let (tiles, map) = extract_tiles(
        &img, &palettes, &tile_pal, tiles_w, &order, config.obj, !config.dedup, config.flip,
    );

    let stats = Stats {
        size: (img.width, img.height),
        grid: (tiles_w, tiles_h),
        total_tiles: tile_count,
        unique_tiles: tiles.len(),
        palettes: palettes.len(),
        quantize_error,
    };
    Ok(Converted { tiles, palettes, map, order, config: config.clone(), stats })
}
