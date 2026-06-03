//! Image quantization for Game Boy — reduce arbitrary images to 4-color-per-tile palettes.
//!
//! Pipeline:
//!   1. K-Means → FPS to find globally diverse candidate colors
//!   2. Per-tile frequency analysis → 4 candidate colors per tile
//!   3. Agglomerative clustering → merge tiles into ≤ max_palettes groups
//!   4. K-Means(k=4) per group → final 4-color palettes
//!   5. Map pixels to palette colors (with optional blue noise dithering + edge detection)

use palette::{FromColor, Oklch, Srgb};

// ── Oklch color space ─────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct OklchPixel {
    l: f32,
    c: f32,
    h: f32,
}

impl OklchPixel {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let srgb = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
        let oklch = Oklch::from_color(srgb);
        Self {
            l: oklch.l,
            c: oklch.chroma,
            h: oklch.hue.into_raw_degrees(),
        }
    }

    fn to_rgb(self) -> [u8; 3] {
        let oklch = Oklch::new(self.l, self.c, self.h);
        let srgb: Srgb = Srgb::from_color(oklch);
        [
            (srgb.red.clamp(0.0, 1.0) * 255.0).round() as u8,
            (srgb.green.clamp(0.0, 1.0) * 255.0).round() as u8,
            (srgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8,
        ]
    }

    fn distance_sq(self, other: Self) -> f32 {
        let dl = self.l - other.l;
        let (sa, sb) = self.to_ab();
        let (oa, ob) = other.to_ab();
        let da = sa - oa;
        let db = sb - ob;
        dl * dl + da * da + db * db
    }

    fn to_ab(self) -> (f32, f32) {
        let h_rad = self.h.to_radians();
        (self.c * h_rad.cos(), self.c * h_rad.sin())
    }

    fn from_ab(l: f32, a: f32, b: f32) -> Self {
        let c = (a * a + b * b).sqrt();
        let h = b.atan2(a).to_degrees();
        Self { l, c, h }
    }
}

fn nearest_idx(pixel: &OklchPixel, candidates: &[OklchPixel]) -> usize {
    let mut best = 0;
    let mut best_dist = f32::MAX;
    for (i, c) in candidates.iter().enumerate() {
        let d = pixel.distance_sq(*c);
        if d < best_dist {
            best_dist = d;
            best = i;
        }
    }
    best
}

// ── Resampling ────────────────────────────────────────────────────────────

/// Box-sample (downsample) an RGBA image.
/// Averages in Oklch space with fractional area weighting for perceptually
/// correct color blending.
pub fn box_sample(
    src: &[[u8; 4]],
    src_w: u32, src_h: u32,
    dst_w: u32, dst_h: u32,
) -> Vec<[u8; 4]> {
    let mut dst = vec![[0u8; 4]; (dst_w * dst_h) as usize];
    let sx = src_w as f64 / dst_w as f64;
    let sy = src_h as f64 / dst_h as f64;

    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let fx0 = dx as f64 * sx;
            let fy0 = dy as f64 * sy;
            let fx1 = ((dx + 1) as f64 * sx).min(src_w as f64);
            let fy1 = ((dy + 1) as f64 * sy).min(src_h as f64);

            let ix0 = fx0.floor() as u32;
            let iy0 = fy0.floor() as u32;
            let ix1 = (fx1.ceil() as u32).min(src_w);
            let iy1 = (fy1.ceil() as u32).min(src_h);

            let mut l_sum = 0.0f64;
            let mut a_sum = 0.0f64;
            let mut b_sum = 0.0f64;
            let mut alpha_sum = 0.0f64;
            let mut weight_sum = 0.0f64;

            for py in iy0..iy1 {
                let wy = (py as f64 + 1.0).min(fy1) - (py as f64).max(fy0);
                for px in ix0..ix1 {
                    let wx = (px as f64 + 1.0).min(fx1) - (px as f64).max(fx0);
                    let w = wx * wy;

                    let p = src[(py * src_w + px) as usize];
                    let oklch = OklchPixel::from_rgb(p[0], p[1], p[2]);
                    let (oa, ob) = oklch.to_ab();

                    l_sum += oklch.l as f64 * w;
                    a_sum += oa as f64 * w;
                    b_sum += ob as f64 * w;
                    alpha_sum += p[3] as f64 * w;
                    weight_sum += w;
                }
            }

            if weight_sum > 0.0 {
                let avg = OklchPixel::from_ab(
                    (l_sum / weight_sum) as f32,
                    (a_sum / weight_sum) as f32,
                    (b_sum / weight_sum) as f32,
                );
                let rgb = avg.to_rgb();
                dst[(dy * dst_w + dx) as usize] = [
                    rgb[0], rgb[1], rgb[2],
                    (alpha_sum / weight_sum).round() as u8,
                ];
            }
        }
    }
    dst
}

/// Downsample a float map using max pooling (preserves edge peaks).
pub fn max_downsample(
    src: &[f32],
    src_w: u32, src_h: u32,
    dst_w: u32, dst_h: u32,
) -> Vec<f32> {
    let mut dst = vec![0.0f32; (dst_w * dst_h) as usize];
    let sx = src_w as f64 / dst_w as f64;
    let sy = src_h as f64 / dst_h as f64;

    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let x0 = (dx as f64 * sx) as u32;
            let y0 = (dy as f64 * sy) as u32;
            let x1 = (((dx + 1) as f64 * sx).ceil() as u32).min(src_w);
            let y1 = (((dy + 1) as f64 * sy).ceil() as u32).min(src_h);

            let mut max_val = 0.0f32;
            for py in y0..y1 {
                for px in x0..x1 {
                    let v = src[(py * src_w + px) as usize];
                    if v > max_val {
                        max_val = v;
                    }
                }
            }
            dst[(dy * dst_w + dx) as usize] = max_val;
        }
    }
    dst
}

// ── Edge detection ────────────────────────────────────────────────────────

/// Compute per-pixel edge strength using Sobel filter on luminance.
/// Returns values normalized to [0, 1].
pub fn compute_edge_map(pixels: &[[u8; 4]], width: u32, height: u32) -> Vec<f32> {
    let w = width as usize;
    let h = height as usize;

    let lum: Vec<f32> = pixels
        .iter()
        .map(|p| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32)
        .collect();

    let mut edges = vec![0.0f32; w * h];
    let mut max_mag = 0.0f32;

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let tl = lum[(y - 1) * w + (x - 1)];
            let tc = lum[(y - 1) * w + x];
            let tr = lum[(y - 1) * w + (x + 1)];
            let ml = lum[y * w + (x - 1)];
            let mr = lum[y * w + (x + 1)];
            let bl = lum[(y + 1) * w + (x - 1)];
            let bc = lum[(y + 1) * w + x];
            let br = lum[(y + 1) * w + (x + 1)];

            let gx = -tl + tr - 2.0 * ml + 2.0 * mr - bl + br;
            let gy = -tl - 2.0 * tc - tr + bl + 2.0 * bc + br;
            let mag = (gx * gx + gy * gy).sqrt();

            edges[y * w + x] = mag;
            if mag > max_mag {
                max_mag = mag;
            }
        }
    }

    if max_mag > 0.0 {
        for e in &mut edges {
            *e /= max_mag;
        }
    }

    edges
}

// ── Clustering helpers ────────────────────────────────────────────────────

/// K-Means clustering in Oklch space.
fn kmeans(pixels: &[OklchPixel], k: usize, max_iter: usize) -> Vec<OklchPixel> {
    if pixels.is_empty() || k == 0 {
        return Vec::new();
    }

    let n = pixels.len();
    let k = k.min(n);

    let mut sorted_indices: Vec<usize> = (0..n).collect();
    sorted_indices.sort_by(|&a, &b| pixels[a].l.partial_cmp(&pixels[b].l).unwrap());
    let mut centroids: Vec<OklchPixel> = (0..k)
        .map(|i| pixels[sorted_indices[i * n / k]])
        .collect();

    let mut assignments = vec![0usize; n];

    for _ in 0..max_iter {
        let mut changed = false;
        for (i, px) in pixels.iter().enumerate() {
            let best = nearest_idx(px, &centroids);
            if assignments[i] != best {
                assignments[i] = best;
                changed = true;
            }
        }

        if !changed {
            break;
        }

        let mut sums_l = vec![0.0f64; k];
        let mut sums_a = vec![0.0f64; k];
        let mut sums_b = vec![0.0f64; k];
        let mut counts = vec![0u64; k];

        for (i, px) in pixels.iter().enumerate() {
            let ci = assignments[i];
            let (a, b) = px.to_ab();
            sums_l[ci] += px.l as f64;
            sums_a[ci] += a as f64;
            sums_b[ci] += b as f64;
            counts[ci] += 1;
        }

        for ci in 0..k {
            if counts[ci] > 0 {
                let n = counts[ci] as f64;
                centroids[ci] = OklchPixel::from_ab(
                    (sums_l[ci] / n) as f32,
                    (sums_a[ci] / n) as f32,
                    (sums_b[ci] / n) as f32,
                );
            }
        }
    }

    centroids
}

/// Quantize a region to `num_colors` using K-Means.
fn quantize_region(pixels: &[[u8; 4]], num_colors: usize) -> (Vec<[u8; 3]>, Vec<u8>) {
    let oklch_pixels: Vec<OklchPixel> = pixels
        .iter()
        .map(|p| OklchPixel::from_rgb(p[0], p[1], p[2]))
        .collect();

    let centroids = kmeans(&oklch_pixels, num_colors, 30);
    let palette: Vec<[u8; 3]> = centroids.iter().map(|c| c.to_rgb()).collect();
    let indices: Vec<u8> = oklch_pixels
        .iter()
        .map(|px| nearest_idx(px, &centroids) as u8)
        .collect();

    (palette, indices)
}

/// Farthest Point Sampling: select `k` maximally spread points.
fn farthest_point_sampling(points: &[OklchPixel], k: usize) -> Vec<OklchPixel> {
    if points.len() <= k {
        return points.to_vec();
    }

    let first = points.iter().enumerate()
        .max_by(|(_, a), (_, b)| a.l.partial_cmp(&b.l).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0);

    let mut selected = vec![first];
    let mut min_dist: Vec<f32> = points.iter()
        .map(|p| p.distance_sq(points[first]))
        .collect();

    for _ in 1..k {
        let next = min_dist.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        selected.push(next);

        for (i, p) in points.iter().enumerate() {
            let d = p.distance_sq(points[next]);
            if d < min_dist[i] {
                min_dist[i] = d;
            }
        }
    }

    selected.iter().map(|&i| points[i]).collect()
}

// ── Dither threshold maps ─────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
pub enum DitherMethod {
    Blue,
    Bayer,
}

/// Pre-computed 16×16 blue noise threshold map (provided by fishcu).
#[rustfmt::skip]
static BLUE_NOISE_16X16: [u8; 256] = [
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

/// 8×8 Bayer ordered dither matrix.
#[rustfmt::skip]
static BAYER_8X8: [u8; 64] = [
     0, 128,  32, 160,   8, 136,  40, 168,
    192,  64, 224,  96, 200,  72, 232, 104,
     48, 176,  16, 144,  56, 184,  24, 152,
    240, 112, 208,  80, 248, 120, 216,  88,
     12, 140,  44, 172,   4, 132,  36, 164,
    204,  76, 236, 108, 196,  68, 228, 100,
     60, 188,  28, 156,  52, 180,  20, 148,
    252, 124, 220,  92, 244, 116, 212,  84,
];

/// Get dither threshold for a pixel position, normalized to [0, 1].
fn dither_threshold(method: DitherMethod, x: usize, y: usize) -> f32 {
    match method {
        DitherMethod::Blue => {
            BLUE_NOISE_16X16[(y % 16) * 16 + (x % 16)] as f32 / 255.0
        }
        DitherMethod::Bayer => {
            BAYER_8X8[(y % 8) * 8 + (x % 8)] as f32 / 255.0
        }
    }
}

// ── Dithering helper ──────────────────────────────────────────────────────

/// Pick a palette color for one pixel using ordered dithering.
/// At edges (high edge_val), falls back to nearest color for sharpness.
fn dither_pixel(
    pixel: OklchPixel,
    pal_oklch: &[OklchPixel],
    threshold: f32,
    strength: f32,
    edge_val: f32,
    edge_threshold: f32,
) -> usize {
    let mut dists: [(usize, f32); 4] = [
        (0, pal_oklch[0].distance_sq(pixel)),
        (1, pal_oklch[1].distance_sq(pixel)),
        (2, pal_oklch[2].distance_sq(pixel)),
        (3, pal_oklch[3].distance_sq(pixel)),
    ];
    dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let (i1, d1) = dists[0];
    let (i2, _d2) = dists[1];

    let d1s = d1.sqrt();
    let d2s = _d2.sqrt();

    // Smoothstep edge suppression
    let lo = (edge_threshold - 0.2).max(0.0);
    let hi = (edge_threshold + 0.2).min(1.0);
    let edge_factor = if edge_val < lo {
        1.0
    } else if edge_val > hi {
        0.0
    } else {
        let x = (edge_val - lo) / (hi - lo);
        1.0 - x * x * (3.0 - 2.0 * x)
    };

    let t = d1s / (d1s + d2s) * strength * edge_factor;
    if threshold < t { i2 } else { i1 }
}

// ── Main quantization pipeline ────────────────────────────────────────────

/// Quantize an image for Game Boy Color.
///
/// Returns (quantized RGBA pixels, palettes as RGB, tile-to-palette index).
pub fn quantize_image_tiled(
    pixels: &[[u8; 4]],
    width: u32,
    height: u32,
    max_palettes: usize,
    num_samples: usize,
    dither: Option<(f32, DitherMethod)>,
    edge_map: Option<&[f32]>,
    edge_threshold: f32,
) -> (Vec<[u8; 4]>, Vec<Vec<[u8; 3]>>, Vec<u8>) {
    use std::collections::BTreeSet;

    let tiles_w = width / 8;
    let tiles_h = height / 8;
    let tile_count = (tiles_w * tiles_h) as usize;
    let num_candidates = (max_palettes * 4).min(tile_count * 4);
    let num_initial = num_samples.max(num_candidates);

    // Step 1: global candidate colors
    let all_oklch: Vec<OklchPixel> = pixels
        .iter()
        .map(|p| OklchPixel::from_rgb(p[0], p[1], p[2]))
        .collect();
    let initial = kmeans(&all_oklch, num_initial, 30);
    let candidates = farthest_point_sampling(&initial, num_candidates);
    eprintln!("  {} candidate colors ({} initial → {} after FPS)",
        candidates.len(), initial.len(), candidates.len());

    // Step 2: per-tile top-4 candidates by frequency
    let mut tile_color_sets: Vec<BTreeSet<usize>> = Vec::with_capacity(tile_count);
    for ty in 0..tiles_h {
        for tx in 0..tiles_w {
            let mut freq = vec![0u32; candidates.len()];
            for py in 0..8u32 {
                for px in 0..8u32 {
                    let idx = ((ty * 8 + py) * width + tx * 8 + px) as usize;
                    freq[nearest_idx(&all_oklch[idx], &candidates)] += 1;
                }
            }
            let mut ranked: Vec<(usize, u32)> = freq.into_iter().enumerate()
                .filter(|&(_, f)| f > 0)
                .collect();
            ranked.sort_by(|a, b| b.1.cmp(&a.1));
            tile_color_sets.push(ranked.iter().take(4).map(|&(ci, _)| ci).collect());
        }
    }

    // Step 3: agglomerative clustering by color-set overlap
    let mut group_members: Vec<Vec<usize>> = (0..tile_count).map(|i| vec![i]).collect();
    let mut group_colors: Vec<BTreeSet<usize>> = tile_color_sets.clone();
    let mut alive: Vec<bool> = vec![true; tile_count];

    while alive.iter().filter(|&&a| a).count() > max_palettes {
        let mut best_i = 0;
        let mut best_j = 0;
        let mut best_cost = usize::MAX;

        let alive_indices: Vec<usize> = alive.iter().enumerate()
            .filter(|&(_, &a)| a)
            .map(|(i, _)| i)
            .collect();

        for (ai, &i) in alive_indices.iter().enumerate() {
            for &j in &alive_indices[ai + 1..] {
                let union_size = group_colors[i].union(&group_colors[j]).count();
                if union_size < best_cost {
                    best_cost = union_size;
                    best_i = i;
                    best_j = j;
                }
            }
        }

        let j_members = std::mem::take(&mut group_members[best_j]);
        group_members[best_i].extend(j_members);
        let j_colors = std::mem::take(&mut group_colors[best_j]);
        group_colors[best_i] = group_colors[best_i].union(&j_colors).copied().collect();
        alive[best_j] = false;
    }

    let final_groups: Vec<usize> = alive.iter().enumerate()
        .filter(|&(_, &a)| a)
        .map(|(i, _)| i)
        .collect();
    let num_groups = final_groups.len();

    let mut tile_group_idx = vec![0usize; tile_count];
    for (gi, &group_id) in final_groups.iter().enumerate() {
        for &ti in &group_members[group_id] {
            tile_group_idx[ti] = gi;
        }
    }

    // Step 4: K-Means(k=4) per group → final palettes
    let mut palettes: Vec<Vec<[u8; 3]>> = Vec::with_capacity(num_groups);
    for &group_id in &final_groups {
        let mut group_pixels = Vec::new();
        for &ti in &group_members[group_id] {
            let tx = (ti as u32) % tiles_w;
            let ty = (ti as u32) / tiles_w;
            for py in 0..8u32 {
                for px in 0..8u32 {
                    group_pixels.push(pixels[((ty * 8 + py) * width + tx * 8 + px) as usize]);
                }
            }
        }
        palettes.push(quantize_region(&group_pixels, 4).0);
    }

    // Step 5: pixel mapping — with optional dither-aware palette reassignment
    let dither_opts = dither; // Option<(strength, method)>
    let mut output = vec![[0u8; 4]; pixels.len()];

    if let Some((dither_strength, dither_method)) = dither_opts {
        let pal_oklchs: Vec<Vec<OklchPixel>> = palettes
            .iter()
            .map(|pal| pal.iter().map(|c| OklchPixel::from_rgb(c[0], c[1], c[2])).collect())
            .collect();

        let mut reassigned = 0usize;
        for ti in 0..tile_count {
            let tx = (ti as u32) % tiles_w;
            let ty = (ti as u32) / tiles_w;

            let mut best_gi = 0;
            let mut best_err = f32::MAX;
            let mut best_choices = [0usize; 64];

            // Try each palette, pick the one with lowest dithered error
            for gi in 0..num_groups {
                let pal_oklch = &pal_oklchs[gi];
                let mut err = 0.0f32;
                let mut choices = [0usize; 64];

                for py in 0..8u32 {
                    for px in 0..8u32 {
                        let gx = (tx * 8 + px) as usize;
                        let gy = (ty * 8 + py) as usize;
                        let idx = gy * width as usize + gx;
                        let o = all_oklch[idx];
                        let edge_val = edge_map.map_or(0.0, |em| em[idx]);
                        let threshold = dither_threshold(dither_method, gx, gy);

                        let chosen = dither_pixel(o, pal_oklch, threshold, dither_strength, edge_val, edge_threshold);
                        let pi = (py * 8 + px) as usize;
                        choices[pi] = chosen;
                        err += o.distance_sq(pal_oklch[chosen]);
                    }
                }

                if err < best_err {
                    best_err = err;
                    best_gi = gi;
                    best_choices = choices;
                }
            }

            if tile_group_idx[ti] != best_gi {
                tile_group_idx[ti] = best_gi;
                reassigned += 1;
            }

            let pal = &palettes[best_gi];
            for py in 0..8u32 {
                for px in 0..8u32 {
                    let gx = (tx * 8 + px) as usize;
                    let gy = (ty * 8 + py) as usize;
                    let idx = gy * width as usize + gx;
                    let c = pal[best_choices[(py * 8 + px) as usize]];
                    output[idx] = [c[0], c[1], c[2], 255];
                }
            }
        }
        eprintln!("  {} tile(s) reassigned for dithering", reassigned);
    } else {
        for ty in 0..tiles_h {
            for tx in 0..tiles_w {
                let ti = (ty * tiles_w + tx) as usize;
                let gi = tile_group_idx[ti];
                let pal = &palettes[gi];
                let pal_oklch: Vec<OklchPixel> = pal
                    .iter()
                    .map(|c| OklchPixel::from_rgb(c[0], c[1], c[2]))
                    .collect();

                for py in 0..8u32 {
                    for px in 0..8u32 {
                        let gx = (tx * 8 + px) as usize;
                        let gy = (ty * 8 + py) as usize;
                        let idx = gy * width as usize + gx;
                        let o = OklchPixel::from_rgb(pixels[idx][0], pixels[idx][1], pixels[idx][2]);
                        let best = pal[nearest_idx(&o, &pal_oklch)];
                        output[idx] = [best[0], best[1], best[2], 255];
                    }
                }
            }
        }
    }

    let tile_pal: Vec<u8> = tile_group_idx.iter().map(|&g| g as u8).collect();
    eprintln!("Quantized: {}×{} tiles, {} palette(s)", tiles_w, tiles_h, num_groups);

    (output, palettes, tile_pal)
}
