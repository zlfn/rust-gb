//! Command line front end: reads the image, runs the conversion, writes the files.

use std::path::{Path, PathBuf};
use std::{fs, process};

use gb_image_fx::{Config, Converted, DitherMethod, Image};

struct Cli {
    input: PathBuf,
    output_prefix: PathBuf,
    preview: bool,
    /// Write the tile map out. The conversion always builds one; this says
    /// whether it becomes a file.
    map: bool,
    config: Config,
}

const HELP: &str = "\
gb-image-fx — Convert images to Game Boy / Game Boy Color tile data

USAGE:
    gb-image-fx <input> [OPTIONS]

OPTIONS:
    -o <prefix>           Output file prefix (default: input file stem)
    --obj                 OBJ/sprite mode: transparent pixels (alpha < 128) become
                          color index 0, and opaque colors fill indices 1-3 (max 3)
    --dmg                 DMG mode: one palette for the whole image, turned
                          grayscale by --quantize. Writes no palette or attribute
                          file. Combine with --obj for sprites.
    --metasprite <WxH>    Emit tiles per sprite cell (e.g. 16x16): cells row-major,
                          each cell column-major (8x16 OBJ pairs)
    --map                 Also emit a tile map naming the tile in each cell. Without
                          it the tiles are written in layout order, one per cell.
    --dedup               Fold identical tiles together (needs --map)
    --flip                Also fold tiles that match when mirrored (needs --dedup)
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

fn parse_args() -> Cli {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        eprint!("{}", HELP);
        process::exit(0);
    }

    let map = args.contains("--map");
    let dedup = args.contains("--dedup");
    let flip = args.contains("--flip");
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
        Some("bayer") => DitherMethod::Bayer,
        Some("ordered") => DitherMethod::Ordered,
        _ => DitherMethod::Blue,
    };
    let dither = dither_weight.map(|w| (w, dither_method));
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

    let config = Config {
        obj,
        dmg,
        quantize,
        max_palettes,
        dither,
        gbc_correction,
        metasprite,
        dedup,
        flip,
    };
    // Folding tiles without writing the map down leaves nothing that says where
    // they went, so the output files would not be usable.
    if dedup && !map {
        eprintln!("error: {}", gb_image_fx::Error::DedupWithoutMap);
        process::exit(1);
    }
    if let Err(e) = config.validate() {
        eprintln!("error: {}", e);
        process::exit(1);
    }

    Cli { input, output_prefix, preview, map, config }
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

/// Read an image file into the RGBA pixels the conversion works on.
fn load(path: &Path) -> Image {
    let img = image::open(path).unwrap_or_else(|e| {
        eprintln!("error: cannot open '{}': {}", path.display(), e);
        process::exit(1);
    });
    let (w, h) = (img.width(), img.height());
    let pixels = img.to_rgba8().pixels().map(|p| [p[0], p[1], p[2], p[3]]).collect();
    Image::from_rgba(w, h, pixels)
}

fn write(path: &str, data: &[u8], note: &str) {
    fs::write(path, data).unwrap_or_else(|e| {
        eprintln!("error: cannot write '{}': {}", path, e);
        process::exit(1);
    });
    eprintln!("  {} ({} bytes{})", path, data.len(), note);
}

fn main() {
    let cli = parse_args();
    let img = load(&cli.input);
    let (src_w, src_h) = (img.width(), img.height());

    let converted: Converted = gb_image_fx::convert(img, &cli.config).unwrap_or_else(|e| {
        eprintln!("error: {}", e);
        process::exit(1);
    });

    let s = converted.stats();
    if cli.config.quantize.is_some() {
        eprintln!("Quantizing {}×{} → {}×{}", src_w, src_h, s.size.0, s.size.1);
    }
    if let Some(err) = s.quantize_error {
        eprintln!("  {} palette(s), error {:.0}", s.palettes, err);
    }
    eprintln!(
        "{}×{} image → {}×{} tiles ({} total)",
        s.size.0, s.size.1, s.grid.0, s.grid.1, s.total_tiles
    );
    eprintln!("{} palette(s) detected", s.palettes);
    eprintln!("{} unique tile(s) (from {} total)", s.unique_tiles, s.total_tiles);

    let prefix = cli.output_prefix.display();
    if cli.preview {
        let (w, h, rgba) = converted.preview();
        let path = format!("{}_preview.png", prefix);
        image::RgbaImage::from_raw(w, h, rgba)
            .unwrap()
            .save(&path)
            .unwrap_or_else(|e| {
                eprintln!("error: cannot write '{}': {}", path, e);
                process::exit(1);
            });
        eprintln!("  {} ({}×{})", path, w, h);
        return;
    }

    write(&format!("{}_tiles.bin", prefix), &converted.tiles(), "");
    if let Some(pal) = converted.palettes() {
        write(
            &format!("{}_palettes.bin", prefix),
            &pal,
            &format!(", {} palette(s)", s.palettes),
        );
    }
    if let Some(attr) = converted.attributes() {
        write(&format!("{}_attributes.bin", prefix), &attr, "");
    }
    if cli.map {
        let map = converted.map().unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            process::exit(1);
        });
        write(&format!("{}_map.bin", prefix), &map, "");
    }
}
