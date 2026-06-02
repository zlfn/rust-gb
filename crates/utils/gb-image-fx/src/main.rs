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
    png_palette: Option<Vec<Color555>>, // Original PNG palette if indexed
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

        // Alpha ignored — treat all pixels as opaque RGB
        let pixels: Vec<[u8; 4]> = rgba.pixels()
            .map(|p| [p[0], p[1], p[2], 255])
            .collect();

        Image { width, height, pixels, png_palette: None }
    }

    fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        self.pixels[(y * self.width + x) as usize]
    }
}

// ── Palette assignment ──────────────────────────────────────────────────────

/// Collect unique colors per tile, group tiles into palettes (max 4 colors each).
/// Returns (palettes, tile_palette_assignment).
fn assign_palettes(
    img: &Image,
    tiles_w: u32,
    tiles_h: u32,
) -> (Vec<[Color555; 4]>, Vec<u8>) {
    // If PNG had an indexed palette with ≤4 colors, use it directly
    if let Some(ref png_pal) = img.png_palette {
        if png_pal.len() <= 4 {
            let mut sorted = png_pal.clone();
            sorted.sort_by(|a, b| b.oklch_lightness().partial_cmp(&a.oklch_lightness()).unwrap());
            let mut pal = [Color555(0); 4];
            for (i, &c) in sorted.iter().enumerate() {
                pal[i] = c;
            }
            let tile_count = (tiles_w * tiles_h) as usize;
            return (vec![pal], vec![0; tile_count]);
        }
    }

    // Collect unique colors per tile
    let mut tile_colors: Vec<Vec<Color555>> = Vec::new();
    for ty in 0..tiles_h {
        for tx in 0..tiles_w {
            let mut colors = Vec::new();
            for py in 0..8 {
                for px in 0..8 {
                    let [r, g, b, _] = img.get_pixel(tx * 8 + px, ty * 8 + py);
                    let c = Color555::from_rgba(r, g, b);
                    if !colors.contains(&c) {
                        colors.push(c);
                    }
                }
            }
            if colors.len() > 4 {
                eprintln!("error: tile ({},{}) has {} colors (max 4)", tx, ty, colors.len());
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
            if merged.len() <= 4 {
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

    // Pad palettes to exactly 4 colors, sorted by Oklch lightness (darkest first)
    let palettes: Vec<[Color555; 4]> = palettes.iter().map(|pal| {
        let mut sorted = pal.clone();
        sorted.sort_by(|a, b| b.oklch_lightness().partial_cmp(&a.oklch_lightness()).unwrap());
        let mut out = [Color555(0); 4];
        for (i, &c) in sorted.iter().enumerate() {
            out[i] = c;
        }
        out
    }).collect();

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

fn extract_tiles(
    img: &Image,
    palettes: &[[Color555; 4]],
    tile_pal: &[u8],
    tiles_w: u32,
    tiles_h: u32,
    keep_duplicates: bool,
    detect_flips: bool,
) -> (Vec<Tile>, Vec<TileMapEntry>) {
    let mut unique_tiles: Vec<Tile> = Vec::new();
    let mut tile_map_entries: Vec<TileMapEntry> = Vec::new();
    let mut tile_lookup: HashMap<Tile, usize> = HashMap::new();

    for ty in 0..tiles_h {
        for tx in 0..tiles_w {
            let ti = (ty * tiles_w + tx) as usize;
            let pal = &palettes[tile_pal[ti] as usize];

            // Extract tile pixels as palette indices
            let mut pixels = [0u8; 64];
            for py in 0..8u32 {
                for px in 0..8u32 {
                    let [r, g, b, _] = img.get_pixel(tx * 8 + px, ty * 8 + py);
                    let c = Color555::from_rgba(r, g, b);
                    let idx = pal.iter().position(|&p| p == c).unwrap_or(0);
                    pixels[(py * 8 + px) as usize] = idx as u8;
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
    num_samples: usize,
    preview: bool,
    dither: Option<(f32, quantize::DitherMethod)>, // (strength, method)
    edge_threshold: f32, // edge detection threshold (0.0-1.0)
}

const HELP: &str = "\
gb-image-fx — Convert images to Game Boy / Game Boy Color tile data

USAGE:
    gb-image-fx <input> [OPTIONS]

OPTIONS:
    -o <prefix>           Output file prefix (default: input file stem)
    --keep-duplicates     Don't deduplicate tiles
    --tiles-only          Only output tiles and palettes (no map/attributes)
    --no-flip             Don't detect flipped tiles during dedup
    --quantize <WxH>      Quantize for target resolution (e.g. 160x144)
    --max-palettes <N>    Maximum palettes for quantize (default: 8, GBC max)
    --samples <N>         K-Means initial color samples for quantize (default: 448)
    --preview             Output a PNG preview image instead of GBC binary files
    --dither [strength]   Apply dithering, 0.0-1.0 (default: 0.5)
    --dither-method <M>   Dither method: 'blue' (default) or 'bayer'
    --edge <threshold>    Edge detection threshold for dithering, 0.0-1.0 (default: 0.4)
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
    let dither_strength: Option<f32> = match args.opt_value_from_str("--dither") {
        Ok(v) => v,
        Err(_) => {
            // --dither flag present but no value → use default 0.5
            args.contains("--dither");
            Some(0.5)
        }
    };
    let dither_method_str: Option<String> = args
        .opt_value_from_str("--dither-method")
        .unwrap_or(None);
    let dither_method = match dither_method_str.as_deref() {
        Some("bayer") => quantize::DitherMethod::Bayer,
        _ => quantize::DitherMethod::Blue,
    };
    let dither = dither_strength.map(|s| (s, dither_method));
    let edge_threshold: f32 = args
        .opt_value_from_str("--edge")
        .unwrap_or(None)
        .unwrap_or(0.4);
    let detect_flips = !args.contains("--no-flip");
    let output_prefix: Option<PathBuf> = args.opt_value_from_str("-o").unwrap_or(None);

    let quantize: Option<(u32, u32)> = args
        .opt_value_from_fn("--quantize", parse_wxh)
        .unwrap_or(None);
    let max_palettes: usize = args
        .opt_value_from_str("--max-palettes")
        .unwrap_or(None)
        .unwrap_or(8);
    let num_samples: usize = args
        .opt_value_from_str("--samples")
        .unwrap_or(None)
        .unwrap_or(448);

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
        num_samples,
        preview,
        dither,
        edge_threshold,
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
    let mut img = Image::load(&opts.input);

    // Quantize: box-sample to target, then tile-aware palette clustering
    let quantize_result = if let Some((tw, th)) = opts.quantize {
        if tw % 8 != 0 || th % 8 != 0 {
            eprintln!("error: --quantize dimensions must be multiples of 8");
            process::exit(1);
        }

        // Compute edge map on original resolution, then box-sample both
        eprintln!("Quantizing {}×{} → {}×{}", img.width, img.height, tw, th);
        let edge_map = if opts.dither.is_some() {
            let orig_edges = quantize::compute_edge_map(&img.pixels, img.width, img.height);
            Some(quantize::max_downsample(&orig_edges, img.width, img.height, tw, th))
        } else {
            None
        };

        if img.width != tw || img.height != th {
            img.pixels = quantize::box_sample(&img.pixels, img.width, img.height, tw, th);
            img.width = tw;
            img.height = th;
        }

        let (quantized, palettes, tile_pal) = quantize::quantize_image_tiled(
            &img.pixels, img.width, img.height, opts.max_palettes, opts.num_samples,
            opts.dither, edge_map.as_deref(), opts.edge_threshold,
        );
        img.pixels = quantized;
        img.png_palette = None;

        // Convert palettes: Vec<Vec<[u8;3]>> → Vec<[Color555;4]>
        let palettes: Vec<[Color555; 4]> = palettes.iter().map(|pal| {
            let mut out = [Color555(0); 4];
            let mut sorted: Vec<Color555> = pal.iter()
                .map(|c| Color555::from_rgba(c[0], c[1], c[2]))
                .collect();
            sorted.sort_by(|a, b| b.oklch_lightness().partial_cmp(&a.oklch_lightness()).unwrap());
            for (i, &c) in sorted.iter().enumerate().take(4) {
                out[i] = c;
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

    let tiles_w = img.width / 8;
    let tiles_h = img.height / 8;
    eprintln!(
        "{}×{} image → {}×{} tiles ({} total)",
        img.width, img.height, tiles_w, tiles_h, tiles_w * tiles_h
    );

    // Palette assignment: use quantize result or derive from image
    let (palettes, tile_pal) = if let Some(qr) = quantize_result {
        qr
    } else {
        assign_palettes(&img, tiles_w, tiles_h)
    };
    eprintln!("{} palette(s) detected", palettes.len());

    // Phase 2: Tile extraction & dedup
    let (tiles, map) = extract_tiles(
        &img, &palettes, &tile_pal, tiles_w, tiles_h,
        opts.keep_duplicates, opts.detect_flips,
    );
    eprintln!(
        "{} unique tile(s) (from {} total)",
        tiles.len(),
        tiles_w * tiles_h
    );

    if opts.preview {
        // Reconstruct image from tiles + palettes + map
        let img_w = (tiles_w * 8) as u32;
        let img_h = (tiles_h * 8) as u32;
        let mut out_buf = vec![0u8; (img_w * img_h * 3) as usize];

        for (i, entry) in map.iter().enumerate() {
            let grid_x = (i as u32) % tiles_w;
            let grid_y = (i as u32) / tiles_w;
            let tile = &tiles[entry.tile_idx as usize];
            let pal = &palettes[entry.palette as usize];

            for py in 0..8u32 {
                for px in 0..8u32 {
                    let src_px = if entry.flip_x { 7 - px } else { px };
                    let src_py = if entry.flip_y { 7 - py } else { py };
                    let color_idx = tile.pixels[(src_py * 8 + src_px) as usize];
                    let [r, g, b] = pal[color_idx as usize].to_rgb8();

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
