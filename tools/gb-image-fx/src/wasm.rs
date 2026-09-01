//! Browser entry point: the conversion over byte arrays, with no filesystem.

use wasm_bindgen::prelude::*;

use crate::{Config, DitherMethod, Error, Image};

/// What to make of the image. Build one, set what you need, and pass it to
/// [`convert`]; the defaults match the command line's.
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct Options {
    /// Sprite mode: transparent pixels become colour 0, opaque ones fill 1-3.
    pub obj: bool,
    /// One palette for the whole image, and no palette or attribute output.
    pub dmg: bool,
    /// Size to reduce a full-colour image to. Zero leaves it alone.
    pub quantize_width: u32,
    pub quantize_height: u32,
    /// Palette ceiling while quantizing.
    pub max_palettes: usize,
    /// Dither weight, or a negative number to leave dithering off.
    pub dither: f64,
    /// Dither pattern: `blue`, `bayer` or `ordered`.
    pub dither_method: String,
    /// Pre-compensate colours for the Game Boy Color's LCD.
    pub gbc_correction: bool,
    /// Sprite cell size in pixels, which reorders the tiles cell by cell.
    pub metasprite_width: u32,
    pub metasprite_height: u32,
    /// Fold identical tiles together. Needs `map` to record where they went.
    pub dedup: bool,
    /// Fold tiles that match once mirrored.
    pub flip: bool,
    /// Return a tile map naming the tile in each cell.
    pub map: bool,
    /// Render the image as the console would show it.
    pub preview: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            obj: false,
            dmg: false,
            quantize_width: 0,
            quantize_height: 0,
            max_palettes: 8,
            dither: -1.0,
            dither_method: "blue".to_string(),
            gbc_correction: false,
            metasprite_width: 0,
            metasprite_height: 0,
            dedup: false,
            flip: false,
            map: false,
            preview: true,
        }
    }
}

#[wasm_bindgen]
impl Options {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Options {
        Options::default()
    }
}

/// A finished conversion. `palettes` and `attributes` are empty for a DMG, which
/// reads neither, and `map` is empty unless it was asked for.
#[wasm_bindgen(getter_with_clone)]
pub struct Output {
    /// 2bpp tile data, 16 bytes per tile.
    pub tiles: Vec<u8>,
    /// RGB555 palettes, 8 bytes each.
    pub palettes: Vec<u8>,
    /// Per-cell attributes, one byte each.
    pub attributes: Vec<u8>,
    /// Per-cell tile indices, one byte each.
    pub map: Vec<u8>,
    /// RGBA preview, four bytes per pixel, ready for a canvas.
    pub preview: Vec<u8>,
    pub preview_width: u32,
    pub preview_height: u32,
    pub unique_tiles: u32,
    pub total_tiles: u32,
    pub palette_count: u32,
}

/// Convert RGBA pixels, four bytes per pixel, row-major.
#[wasm_bindgen]
pub fn convert(rgba: &[u8], width: u32, height: u32, options: &Options) -> Result<Output, JsError> {
    if rgba.len() != (width as usize) * (height as usize) * 4 {
        return Err(JsError::new("rgba length does not match the dimensions"));
    }
    // Folding tiles without a map leaves nothing that says where they went.
    if options.dedup && !options.map {
        return Err(JsError::new(&Error::DedupWithoutMap.to_string()));
    }

    let config = Config {
        obj: options.obj,
        dmg: options.dmg,
        quantize: (options.quantize_width > 0 && options.quantize_height > 0)
            .then_some((options.quantize_width, options.quantize_height)),
        max_palettes: options.max_palettes,
        dither: (options.dither >= 0.0).then(|| {
            let method = match options.dither_method.as_str() {
                "bayer" => DitherMethod::Bayer,
                "ordered" => DitherMethod::Ordered,
                _ => DitherMethod::Blue,
            };
            (options.dither, method)
        }),
        gbc_correction: options.gbc_correction,
        metasprite: (options.metasprite_width > 0 && options.metasprite_height > 0)
            .then_some((options.metasprite_width, options.metasprite_height)),
        dedup: options.dedup,
        flip: options.flip,
    };

    let pixels = rgba.chunks_exact(4).map(|c| [c[0], c[1], c[2], c[3]]).collect();
    let converted = crate::convert(Image::from_rgba(width, height, pixels), &config)
        .map_err(|e| JsError::new(&e.to_string()))?;

    // A map that cannot name every tile is an error, not an empty array.
    let map = match options.map {
        true => converted.map().map_err(|e| JsError::new(&e.to_string()))?,
        false => Vec::new(),
    };
    let (preview_width, preview_height, preview) = match options.preview {
        true => converted.preview(),
        false => (0, 0, Vec::new()),
    };

    let stats = converted.stats();
    Ok(Output {
        tiles: converted.tiles(),
        palettes: converted.palettes().unwrap_or_default(),
        attributes: converted.attributes().unwrap_or_default(),
        map,
        preview,
        preview_width,
        preview_height,
        unique_tiles: stats.unique_tiles as u32,
        total_tiles: stats.total_tiles as u32,
        palette_count: stats.palettes as u32,
    })
}
