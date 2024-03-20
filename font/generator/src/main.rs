use stb_truetype_rust::{stbtt_BakeFontBitmap, stbtt_bakedchar as BakedChar};
use std::io::Write;

const FONT_HEIGHT: f32 = 18.0;
const W: i32 = 128;
const H: i32 = 128;
const NUM_CHARS: usize = 96;
const FIRST_CHAR: i32 = 32;

fn main() {
    let mut args = std::env::args();
    let _ = args.next().unwrap();
    let font_name = args.next().unwrap();
    let output_name = args.next();

    let font = std::fs::read(font_name.as_str()).unwrap();

    let mut pixels = vec![0; (W * H) as usize];
    let mut char_data = Vec::new();
    for _ in 0..NUM_CHARS {
        char_data.push(BakedChar {
            x0: 0,
            y0: 0,
            x1: 0,
            y1: 0,
            xoff: 0.0,
            yoff: 0.0,
            xadvance: 0.0,
        });
    }

    let num_rows = unsafe {
        stbtt_BakeFontBitmap(
            font.as_ptr(),
            0,
            FONT_HEIGHT,
            pixels.as_mut_ptr(),
            W,
            H,
            FIRST_CHAR,
            NUM_CHARS as i32,
            char_data.as_mut_ptr(),
        )
    };

    assert!(num_rows != 0, "Failed to generate the atlas");
    assert!(num_rows > 0, "The glyphs don't fit in the atlas");
    let num_rows = num_rows + 8 - (num_rows % 8);

    if let Some(output_name) = &output_name {
        if output_name.ends_with(".png") {
            dump_png(&pixels, W, H, output_name.as_str());
        } else if output_name.ends_with(".rs") {
            let mut output = std::fs::File::create(&output_name).unwrap();
            generate_code(&pixels, W as i32, num_rows, &char_data, &font_name, &mut output).unwrap();
        }
    } else {
        generate_code(
            &pixels,
            W as i32,
            num_rows,
            &char_data,
            &font_name,
            &mut std::io::stdout(),
        )
        .unwrap();
    }
}

fn dump_png(pixels: &[u8], w: i32, h: i32, file_name: &str) {
    let mut rgba_pixels = Vec::with_capacity((w * h * 4) as usize);
    for p in pixels {
        rgba_pixels.push(*p);
        rgba_pixels.push(*p);
        rgba_pixels.push(*p);
        rgba_pixels.push(255);
    }

    let file = std::fs::File::create(file_name).unwrap();
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&rgba_pixels).unwrap();
}

fn generate_code(
    pixels: &[u8],
    w: i32,
    h: i32,
    char_data: &[BakedChar],
    font_path: &str,
    output: &mut dyn Write,
) -> std::io::Result<()> {
    let pixels = &pixels[..(w * h) as usize];
    let font_name = font_path.rsplit("/").next().unwrap();

    writeln!(output, "/// An embedded bitmap ascii font for debugging purposes.")?;
    writeln!(output, "/// Generated from font {font_name}.")?;
    writeln!(output, "")?;
    writeln!(output, "pub const FIRST_CHAR: u32 = {FIRST_CHAR};")?;
    writeln!(output, "pub const ATLAS_WIDTH: u32 = {w};")?;
    writeln!(output, "pub const ATLAS_HEIGHT: u32 = {h};")?;
    writeln!(output, "pub const FONT_HEIGHT: u32 = {FONT_HEIGHT};")?;
    writeln!(output, "")?;
    writeln!(output, "pub struct GlyphInfo {{")?;
    writeln!(output, "    pub uv0: (u16, u16),")?;
    writeln!(output, "    pub uv1: (u16, u16),")?;
    writeln!(output, "    pub offset: (i16, i16),")?;
    writeln!(output, "    pub x_advance: f32,")?;
    writeln!(output, "}}")?;
    writeln!(output, "")?;
    writeln!(output, "pub const GLYPH_INFO: &[GlyphInfo] = &[")?;
    for c in char_data {
        writeln!(
            output,
            "    GlyphInfo {{ uv0: ({}, {}), uv1: ({}, {}), offset: ({}, {}), x_advance: {} }},",
            c.x0, c.y0, c.y0, c.y1, c.xoff, c.yoff, c.xadvance
        )?;
    }
    writeln!(output, "];")?;
    writeln!(output, "")?;
    writeln!(output, "pub const GLYPH_ATLAS: &[u8] = &[")?;
    for px in pixels.chunks(16) {
        write!(output, "   ")?;
        for p in px {
            write!(output, " {hexa:>4},", hexa = format!("0x{:X}", p))?;
        }
        writeln!(output, "")?;
    }
    writeln!(output, "];")?;
    writeln!(output, "")?;

    Ok(())
}
