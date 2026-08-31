//! Reduce a full-colour image to Game Boy Color tile data: up to eight
//! four-colour palettes, one per 8x8 tile.
//!
//! Ported from rilden's tiledpalettequant
//! (https://github.com/rilden/tiledpalettequant), Copyright (c) 2022 rilden,
//! MIT-licensed.

// ── Colours ─────────────────────────────────────────────────────────────────

/// An sRGB colour with channels in 0..=255, kept as f64 so learning can nudge it
/// by fractional amounts.
type Rgb = [f64; 3];

/// Green-weighted squared distance, a cheap stand-in for perceived difference.
#[inline]
fn dist(a: Rgb, b: Rgb) -> f64 {
    let (dr, dg, db) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    2.0 * dr * dr + 4.0 * dg * dg + db * db
}

/// Relative brightness, used to order dither candidates.
#[inline]
fn brightness(c: Rgb) -> f64 {
    0.299 * c[0] * c[0] + 0.587 * c[1] * c[1] + 0.114 * c[2] * c[2]
}

fn to_u8(c: Rgb) -> [u8; 3] {
    [
        c[0].round().clamp(0.0, 255.0) as u8,
        c[1].round().clamp(0.0, 255.0) as u8,
        c[2].round().clamp(0.0, 255.0) as u8,
    ]
}

/// The value the console shows for `x`: quantized to `bits` per channel.
fn to_nbit(x: f64, bits: usize) -> f64 {
    let step = 255.0 / ((1u32 << bits) - 1) as f64;
    (x / step).round() * step
}

fn reduce(c: Rgb, bits: usize) -> Rgb {
    [to_nbit(c[0], bits), to_nbit(c[1], bits), to_nbit(c[2], bits)]
}

fn reduce_all(palettes: &[Vec<Rgb>], bits: usize) -> Vec<Vec<Rgb>> {
    palettes.iter().map(|p| p.iter().map(|&c| reduce(c, bits)).collect()).collect()
}

/// Index and distance of the palette colour nearest `c`.
fn nearest(pal: &[Rgb], c: Rgb) -> (usize, f64) {
    let mut best = (0, f64::MAX);
    for (i, &p) in pal.iter().enumerate() {
        let d = dist(p, c);
        if d < best.1 {
            best = (i, d);
        }
    }
    best
}

/// The two nearest colours as (index, distance) pairs, nearest first.
fn two_nearest(pal: &[Rgb], c: Rgb) -> ((usize, f64), (usize, f64)) {
    let (mut a, mut b) = ((0, f64::MAX), (0, f64::MAX));
    for (i, &p) in pal.iter().enumerate() {
        let d = dist(p, c);
        if d < a.1 {
            b = a;
            a = (i, d);
        } else if d < b.1 {
            b = (i, d);
        }
    }
    (a, b)
}

// ── Deterministic pixel shuffle ─────────────────────────────────────────────
//
// The original draws random pixels via Math.random; a fixed-seed xorshift keeps
// the output reproducible without changing the algorithm.

struct Shuffle {
    order: Vec<usize>,
    pos: usize,
    state: u32,
}

impl Shuffle {
    fn new(n: usize) -> Self {
        Shuffle { order: (0..n).collect(), pos: n.wrapping_sub(1), state: 0x9E3779B9 }
    }

    fn unit(&mut self) -> f64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x as f64 / (u32::MAX as f64 + 1.0)
    }

    fn reshuffle(&mut self) {
        let n = self.order.len();
        for i in 0..n {
            let j = i + (self.unit() * (n - i) as f64) as usize;
            self.order.swap(i, j.min(n - 1));
        }
    }

    fn next(&mut self) -> usize {
        self.pos += 1;
        if self.pos >= self.order.len() {
            self.reshuffle();
            self.pos = 0;
        }
        self.order[self.pos]
    }
}

// ── Dithering ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
pub enum DitherMethod {
    Blue,
    Bayer,
    /// Fixed 2x2 pattern that places all four candidates once per block, giving
    /// an exact local average at the cost of a visible regular texture.
    Ordered,
}

/// How many candidates the error-feedback dither produces per pixel.
const DITHER_LEVELS: usize = 4;

#[rustfmt::skip]
static BLUE_NOISE: [u8; 256] = [
    209, 161, 104, 199, 19,  172, 87,  158, 22,  213, 143, 224, 93,  235, 117, 84,
    244, 1,   180, 79,  222, 129, 110, 229, 135, 80,  195, 6,   65,  171, 148, 18,
    53,  134, 66,  238, 145, 37,  7,   192, 56,  102, 157, 120, 185, 42,  101, 223,
    168, 115, 190, 25,  98,  205, 73,  249, 174, 17,  240, 34,  210, 251, 74,  193,
    90,  38,  255, 154, 51,  173, 126, 150, 47,  221, 71,  94,  141, 12,  124, 28,
    219, 204, 105, 14,  217, 234, 92,  29,  112, 200, 130, 181, 49,  164, 233, 151,
    5,   138, 72,  183, 116, 63,  9,   212, 166, 83,  0,   227, 109, 202, 82,  59,
    247, 165, 88,  40,  159, 132, 191, 239, 142, 57,  248, 149, 64,  24,  178, 119,
    189, 27,  228, 201, 246, 31,  77,  45,  103, 20,  186, 36,  214, 97,  237, 46,
    107, 67,  128, 2,   96,  146, 218, 179, 125, 207, 86,  118, 162, 133, 10,  147,
    89,  211, 153, 176, 60,  111, 11,  253, 68,  169, 232, 13,  245, 76,  194, 225,
    55,  21,  242, 43,  230, 196, 163, 91,  26,  137, 41,  108, 182, 50,  30,  167,
    114, 198, 136, 85,  121, 33,  215, 52,  156, 226, 197, 62,  144, 220, 127, 254,
    8,   70,  170, 15,  184, 75,  139, 241, 99,  16,  81,  208, 3,   100, 78,  155,
    231, 95,  216, 250, 152, 106, 4,   188, 123, 177, 113, 160, 243, 175, 39,  187,
    140, 32,  122, 48,  61,  236, 206, 44,  69,  252, 35,  54,  131, 23,  203, 58,
];

#[rustfmt::skip]
static BAYER: [u8; 64] = [
     0, 128,  32, 160,   8, 136,  40, 168,
    192,  64, 224,  96, 200,  72, 232, 104,
     48, 176,  16, 144,  56, 184,  24, 152,
    240, 112, 208,  80, 248, 120, 216,  88,
     12, 140,  44, 172,   4, 132,  36, 164,
    204,  76, 236, 108, 196,  68, 228, 100,
     60, 188,  28, 156,  52, 180,  20, 148,
    252, 124, 220,  92, 244, 116, 212,  84,
];

/// Which of the brightness-sorted candidates pixel (x, y) takes.
fn dither_rank(method: DitherMethod, x: u32, y: u32) -> usize {
    let t = match method {
        DitherMethod::Ordered => return [[0, 2], [3, 1]][(x & 1) as usize][(y & 1) as usize],
        DitherMethod::Blue => BLUE_NOISE[((y % 16) * 16 + x % 16) as usize] as f64,
        DitherMethod::Bayer => BAYER[((y % 8) * 8 + x % 8) as usize] as f64,
    };
    ((t / 256.0) * DITHER_LEVELS as f64) as usize
}

/// Pick a colour for one pixel by error-feedback dithering. The candidates are
/// built by repeatedly quantizing the pixel plus its accumulated error (in linear
/// light, so they average optically toward the target), then one is selected by
/// the pattern. Returns the colour index, its distance, and the error-adjusted
/// target that competitive learning should move toward.
fn dither_pick(pal: &[Rgb], color: Rgb, x: u32, y: u32, method: DitherMethod, weight: f64) -> (usize, f64, Rgb) {
    let target = [color[0] * color[0], color[1] * color[1], color[2] * color[2]];
    let mut err = [0.0; 3];
    let mut cand = [(0usize, 0.0f64, 0.0f64, [0.0; 3]); DITHER_LEVELS];
    for slot in &mut cand {
        let adjusted = [
            (target[0] + err[0] * weight).clamp(0.0, 65025.0).sqrt(),
            (target[1] + err[1] * weight).clamp(0.0, 65025.0).sqrt(),
            (target[2] + err[2] * weight).clamp(0.0, 65025.0).sqrt(),
        ];
        let (i, d) = nearest(pal, adjusted);
        *slot = (i, d, brightness(pal[i]), adjusted);
        let shown = reduce(pal[i], 5);
        for k in 0..3 {
            err[k] += target[k] - shown[k] * shown[k];
        }
    }
    cand.sort_by(|a, b| a.2.total_cmp(&b.2));
    let (i, d, _, adjusted) = cand[dither_rank(method, x, y)];
    (i, d, adjusted)
}

// ── Tiles ───────────────────────────────────────────────────────────────────

struct Pixel {
    color: Rgb,
    x: u32,
    y: u32,
    tile: usize,
}

/// The distinct colours of one tile (with pixel counts) and the pixels it holds.
struct Tile {
    colors: Vec<Rgb>,
    counts: Vec<f64>,
    pixels: Vec<usize>,
}

/// How much error results from covering `tile` with `pal`.
fn tile_cost(pal: &[Rgb], tile: &Tile) -> f64 {
    tile.colors.iter().zip(&tile.counts).map(|(&c, &n)| nearest(pal, c).1 * n).sum()
}

fn tile_cost_dithered(pal: &[Rgb], tile: &Tile, pixels: &[Pixel], method: DitherMethod, weight: f64) -> f64 {
    tile.pixels
        .iter()
        .map(|&p| {
            let px = &pixels[p];
            dither_pick(pal, px.color, px.x, px.y, method, weight).1
        })
        .sum()
}

/// The palette that covers `tile` most cheaply.
fn best_palette(palettes: &[Vec<Rgb>], tile: &Tile, dither: Option<(f64, DitherMethod)>, pixels: &[Pixel]) -> usize {
    if palettes.len() == 1 {
        return 0;
    }
    let mut best = (0, f64::MAX);
    for (i, pal) in palettes.iter().enumerate() {
        let c = match dither {
            Some((w, m)) => tile_cost_dithered(pal, tile, pixels, m, w),
            None => tile_cost(pal, tile),
        };
        if c < best.1 {
            best = (i, c);
        }
    }
    best.0
}

/// Total error of the current palettes over all tiles (best-fit assignment).
fn total_error(palettes: &[Vec<Rgb>], tiles: &[Tile]) -> f64 {
    tiles.iter().map(|t| tile_cost(&palettes[best_palette(palettes, t, None, &[])], t)).sum()
}

// ── Competitive learning ────────────────────────────────────────────────────

/// Move `c` a fraction `alpha` toward `target`.
fn nudge(c: &mut Rgb, target: Rgb, alpha: f64) {
    for k in 0..3 {
        c[k] = (1.0 - alpha) * c[k] + alpha * target[k];
    }
}

/// One learning step: route a pixel through its tile's best palette and pull the
/// winning colour toward it.
fn learn(palettes: &mut [Vec<Rgb>], tiles: &[Tile], pixels: &[Pixel], pi: usize, alpha: f64, dither: Option<(f64, DitherMethod)>) {
    let px = &pixels[pi];
    let tile = &tiles[px.tile];
    let p = best_palette(palettes, tile, dither, pixels);
    let (ci, target) = match dither {
        Some((w, m)) => {
            let (ci, _, adjusted) = dither_pick(&palettes[p], px.color, px.x, px.y, m, w);
            (ci, adjusted)
        }
        None => (nearest(&palettes[p], px.color).0, px.color),
    };
    nudge(&mut palettes[p][ci], target, alpha);
}

// ── Growing palettes ────────────────────────────────────────────────────────

/// Grow the palette *count* from one to `count`, splitting the worst-fitting
/// palette each time and letting competitive learning separate the copies.
fn grow_palettes(tiles: &[Tile], pixels: &[Pixel], shuffle: &mut Shuffle, count: usize, iters: usize, alpha: f64, dither: Option<(f64, DitherMethod)>) -> Vec<Vec<Rgb>> {
    let mut mean = [0.0; 3];
    for px in pixels {
        for k in 0..3 {
            mean[k] += px.color[k];
        }
    }
    let inv = 1.0 / pixels.len().max(1) as f64;
    let mut palettes = vec![vec![[mean[0] * inv, mean[1] * inv, mean[2] * inv]]];

    let mut worst = 0;
    while palettes.len() < count {
        palettes.push(palettes[worst].clone());
        for _ in 0..iters {
            let pi = shuffle.next();
            learn(&mut palettes, tiles, pixels, pi, alpha, dither);
        }
        let mut err = vec![0.0; palettes.len()];
        for tile in tiles {
            let p = best_palette(&palettes, tile, None, &[]);
            err[p] += tile_cost(&palettes[p], tile);
        }
        worst = argmax(&err);
    }
    palettes
}

/// Add one colour to every palette, splitting each palette's worst-fitting slot.
fn grow_colors(palettes: &mut [Vec<Rgb>], tiles: &[Tile], pixels: &[Pixel], shuffle: &mut Shuffle, iters: usize, alpha: f64, dither: Option<(f64, DitherMethod)>) {
    let mut split = vec![0usize; palettes.len()];
    if palettes[0].len() > 1 {
        let mut err: Vec<Vec<f64>> = palettes.iter().map(|p| vec![0.0; p.len()]).collect();
        for tile in tiles {
            let p = best_palette(palettes, tile, None, &[]);
            for (&c, &n) in tile.colors.iter().zip(&tile.counts) {
                let (i, d) = nearest(&palettes[p], c);
                err[p][i] += d * n;
            }
        }
        for (p, e) in err.iter().enumerate() {
            split[p] = argmax(e);
        }
    }
    for (p, pal) in palettes.iter_mut().enumerate() {
        pal.push(pal[split[p]]);
    }
    for _ in 0..iters {
        let pi = shuffle.next();
        learn(palettes, tiles, pixels, pi, alpha, dither);
    }
}

// ── Reallocating wasted capacity ────────────────────────────────────────────

/// Reclaim a palette or a colour slot that is barely pulling its weight and hand
/// it to whichever is most overloaded, so budget follows the error. Returns a new
/// palette set; the caller re-runs learning to settle the clones apart.
fn reallocate(palettes: &[Vec<Rgb>], tiles: &[Tile], pixels: &[Pixel], dither: Option<(f64, DitherMethod)>) -> Vec<Vec<Rgb>> {
    let np = palettes.len();
    let cost = |pal: &[Rgb], t: &Tile| match dither {
        Some((w, m)) => tile_cost_dithered(pal, t, pixels, m, w),
        None => tile_cost(pal, t),
    };
    let assign: Vec<usize> = tiles.iter().map(|t| best_palette(palettes, t, dither, pixels)).collect();

    // Per palette: error it absorbs, and error if it were dropped and its tiles
    // reassigned elsewhere.
    let mut absorbed = vec![0.0; np];
    let mut without = vec![0.0; np];
    for (t, tile) in tiles.iter().enumerate() {
        let p = assign[t];
        absorbed[p] += cost(&palettes[p], tile);
        let mut second = f64::MAX;
        for (q, pal) in palettes.iter().enumerate() {
            if q != p {
                second = second.min(cost(pal, tile));
            }
        }
        if second.is_finite() {
            without[p] += second;
        }
    }
    let overloaded = argmax(&absorbed);
    let expendable = argmin(&without);

    // Per palette: same idea one level down, for each colour slot.
    let mut result = palettes.to_vec();
    if palettes[0].len() > 1 {
        for (p, pal) in palettes.iter().enumerate() {
            let mut absorbed_c = vec![0.0; pal.len()];
            let mut without_c = vec![0.0; pal.len()];
            for (t, tile) in tiles.iter().enumerate() {
                if assign[t] != p {
                    continue;
                }
                for (&c, &n) in tile.colors.iter().zip(&tile.counts) {
                    let (first, second) = two_nearest(pal, c);
                    absorbed_c[first.0] += first.1 * n;
                    without_c[first.0] += second.1 * n;
                }
            }
            let over = argmax(&absorbed_c);
            let exp = argmin(&without_c);
            if exp != over && without_c[exp] < 0.5 * absorbed_c[over] {
                result[p][exp] = pal[over];
            }
        }
    }

    if expendable != overloaded && without[expendable] < 0.5 * absorbed[overloaded] {
        result[expendable] = result[overloaded].clone();
    }
    result
}

// ── Batch refinement (k-means) ──────────────────────────────────────────────

/// Move every palette colour to the mean of the pixels that chose it.
fn kmeans(palettes: &[Vec<Rgb>], tiles: &[Tile]) -> Vec<Vec<Rgb>> {
    let mut sum: Vec<Vec<[f64; 3]>> = palettes.iter().map(|p| vec![[0.0; 3]; p.len()]).collect();
    let mut count: Vec<Vec<f64>> = palettes.iter().map(|p| vec![0.0; p.len()]).collect();
    for tile in tiles {
        let p = best_palette(palettes, tile, None, &[]);
        for (&c, &n) in tile.colors.iter().zip(&tile.counts) {
            let i = nearest(&palettes[p], c).0;
            for k in 0..3 {
                sum[p][i][k] += c[k] * n;
            }
            count[p][i] += n;
        }
    }
    palettes
        .iter()
        .enumerate()
        .map(|(p, pal)| {
            pal.iter()
                .enumerate()
                .map(|(i, &c)| {
                    if count[p][i] == 0.0 {
                        c
                    } else {
                        let inv = 1.0 / count[p][i];
                        [sum[p][i][0] * inv, sum[p][i][1] * inv, sum[p][i][2] * inv]
                    }
                })
                .collect()
        })
        .collect()
}

fn argmax(v: &[f64]) -> usize {
    (0..v.len()).max_by(|&a, &b| v[a].total_cmp(&v[b])).unwrap_or(0)
}

fn argmin(v: &[f64]) -> usize {
    (0..v.len()).min_by(|&a, &b| v[a].total_cmp(&v[b])).unwrap_or(0)
}

// ── Downscaling ─────────────────────────────────────────────────────────────

/// Area-average an image to a new size, blending in linear light. With `obj`,
/// colour is weighted by coverage *and* alpha so fully transparent pixels don't
/// bleed their RGB into opaque neighbours; alpha itself is averaged by coverage
/// alone. Without `obj` there is no transparency, so colour uses coverage only.
pub fn box_sample(src: &[[u8; 4]], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32, obj: bool) -> Vec<[u8; 4]> {
    let mut dst = vec![[0u8; 4]; (dst_w * dst_h) as usize];
    let sx = src_w as f64 / dst_w as f64;
    let sy = src_h as f64 / dst_h as f64;
    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let (x0, y0) = (dx as f64 * sx, dy as f64 * sy);
            let (x1, y1) = (((dx + 1) as f64 * sx).min(src_w as f64), ((dy + 1) as f64 * sy).min(src_h as f64));
            let mut acc = [0.0f64; 4];
            let (mut color_w, mut weight) = (0.0, 0.0);
            for py in y0.floor() as u32..(y1.ceil() as u32).min(src_h) {
                let wy = (py as f64 + 1.0).min(y1) - (py as f64).max(y0);
                for px in x0.floor() as u32..(x1.ceil() as u32).min(src_w) {
                    let wx = (px as f64 + 1.0).min(x1) - (px as f64).max(x0);
                    let w = wx * wy;
                    let p = src[(py * src_w + px) as usize];
                    let cw = if obj { w * (p[3] as f64 / 255.0) } else { w };
                    for k in 0..3 {
                        acc[k] += (p[k] as f64).powi(2) * cw;
                    }
                    acc[3] += p[3] as f64 * w;
                    color_w += cw;
                    weight += w;
                }
            }
            if weight > 0.0 {
                let cinv = if color_w > 0.0 { 1.0 / color_w } else { 0.0 };
                dst[(dy * dst_w + dx) as usize] = [
                    (acc[0] * cinv).sqrt().round().clamp(0.0, 255.0) as u8,
                    (acc[1] * cinv).sqrt().round().clamp(0.0, 255.0) as u8,
                    (acc[2] * cinv).sqrt().round().clamp(0.0, 255.0) as u8,
                    (acc[3] / weight).round() as u8,
                ];
            }
        }
    }
    dst
}

// ── Driver ──────────────────────────────────────────────────────────────────

/// Quantize `pixels` (RGBA, `width`x`height`) into up to `max_palettes` GBC
/// palettes. `dither` gives the strength and pattern, or `None` for flat output.
/// With `obj`, pixels with alpha below 128 are transparent: they are left out of
/// clustering and each palette holds three opaque colours instead of four.
///
/// Returns the quantized RGBA image (transparent pixels keep alpha 0), the
/// palettes as RGB triplets, and the palette index chosen for each tile
/// (row-major).
pub fn quantize_image_tiled(
    pixels_in: &[[u8; 4]],
    width: u32,
    height: u32,
    max_palettes: usize,
    dither: Option<(f64, DitherMethod)>,
    obj: bool,
) -> (Vec<[u8; 4]>, Vec<Vec<[u8; 3]>>, Vec<u8>) {
    const BITS: usize = 5;
    let colors_per_palette = if obj { 3 } else { 4 };
    let max_palettes = max_palettes.max(1);
    let dithering = dither.is_some();
    let (tiles_w, tiles_h) = (width / 8, height / 8);

    // Without dithering, snap the source to the display depth first: neighbouring
    // tiles in a gradient then share colours and pick the same palette.
    let source: Vec<[u8; 4]> = if dithering {
        pixels_in.to_vec()
    } else {
        pixels_in
            .iter()
            .map(|p| [to_nbit(p[0] as f64, BITS) as u8, to_nbit(p[1] as f64, BITS) as u8, to_nbit(p[2] as f64, BITS) as u8, p[3]])
            .collect()
    };

    let mut tiles = Vec::new();
    let mut pixels = Vec::new();
    for ty in 0..tiles_h {
        for tx in 0..tiles_w {
            let id = tiles.len();
            let (mut colors, mut counts, mut members) = (Vec::new(), Vec::new(), Vec::new());
            for py in 0..8 {
                for px in 0..8 {
                    let (x, y) = (tx * 8 + px, ty * 8 + py);
                    let p = source[(y * width + x) as usize];
                    // Transparent pixels stay index 0; keep them out of clustering.
                    if obj && p[3] < 128 {
                        continue;
                    }
                    let color = [p[0] as f64, p[1] as f64, p[2] as f64];
                    match colors.iter().position(|&c| c == color) {
                        Some(i) => counts[i] += 1.0,
                        None => {
                            colors.push(color);
                            counts.push(1.0);
                        }
                    }
                    members.push(pixels.len());
                    pixels.push(Pixel { color, x, y, tile: id });
                }
            }
            tiles.push(Tile { colors, counts, pixels: members });
        }
    }

    let base = (0.1 * pixels.len() as f64) as usize;
    let (iters, alpha, final_alpha) = if dithering { (base / 5, 0.1, 0.02) } else { (base, 0.3, 0.05) };
    let mut shuffle = Shuffle::new(pixels.len());

    let mut palettes = grow_palettes(&tiles, &pixels, &mut shuffle, max_palettes, iters, alpha, dither);
    for _ in 1..colors_per_palette {
        grow_colors(&mut palettes, &tiles, &pixels, &mut shuffle, iters, alpha, dither);
    }

    let mut best = palettes.clone();
    let mut best_error = total_error(&palettes, &tiles);
    for _ in 0..10 {
        palettes = reallocate(&palettes, &tiles, &pixels, dither);
        for _ in 0..iters {
            let pi = shuffle.next();
            learn(&mut palettes, &tiles, &pixels, pi, alpha, dither);
        }
        let e = total_error(&palettes, &tiles);
        if e < best_error {
            best_error = e;
            best = palettes.clone();
        }
    }
    palettes = best;

    if !dithering {
        palettes = reduce_all(&palettes, BITS);
    }
    for _ in 0..iters * 10 {
        let pi = shuffle.next();
        learn(&mut palettes, &tiles, &pixels, pi, final_alpha, dither);
    }
    if !dithering {
        palettes = reduce_all(&palettes, BITS);
        for _ in 0..3 {
            palettes = kmeans(&palettes, &tiles);
        }
    }
    let palettes = reduce_all(&palettes, BITS);

    // Assign tiles and paint.
    let mut tile_pal = vec![0u8; tiles.len()];
    let mut out = vec![[0u8; 4]; pixels_in.len()];
    for (id, tile) in tiles.iter().enumerate() {
        let p = best_palette(&palettes, tile, dither, &pixels);
        tile_pal[id] = p as u8;
        for &m in &tile.pixels {
            let px = &pixels[m];
            let ci = match dither {
                Some((w, method)) => dither_pick(&palettes[p], px.color, px.x, px.y, method, w).0,
                None => nearest(&palettes[p], px.color).0,
            };
            let rgb = to_u8(palettes[p][ci]);
            out[(px.y * width + px.x) as usize] = [rgb[0], rgb[1], rgb[2], 255];
        }
    }

    eprintln!("  {} palette(s), error {:.0}", palettes.len(), best_error);
    let palettes_rgb = palettes.iter().map(|p| p.iter().map(|&c| to_u8(c)).collect()).collect();
    (out, palettes_rgb, tile_pal)
}
