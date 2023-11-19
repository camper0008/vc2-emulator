use std::{fs, io, path::PathBuf};

use gumdrop::Options;
use image::{io::Reader, DynamicImage, Pixel, Rgba};

#[derive(Options)]
struct MyOptions {
    #[options(help = "print help message")]
    help: bool,

    #[options(
        help = "get input from <file paths>(s)",
        required,
        free,
        parse(try_from_str = "read_file")
    )]
    files: Vec<FileInfo<DynamicImage>>,

    #[options(
        help = "vram address",
        default = "0x2034",
        parse(try_from_str = "parse_number")
    )]
    vram: u32,

    #[options(
        help = "screen width address",
        default = "0x2038",
        parse(try_from_str = "parse_number")
    )]
    screen_width: u32,
}

fn parse_number(number: &str) -> Result<u32, String> {
    if number.starts_with("0x") {
        u32::from_str_radix(&number[2..], 16).map_err(|e| e.to_string())
    } else if number.starts_with("0b") {
        u32::from_str_radix(&number[2..], 2).map_err(|e| e.to_string())
    } else {
        number.parse::<u32>().map_err(|e| e.to_string())
    }
}

struct FileInfo<Image> {
    image: Image,
    name: String,
}

fn read_file(path: &str) -> Result<FileInfo<DynamicImage>, String> {
    let image = Reader::open(path)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;
    let name = PathBuf::from(path);
    let name = name
        .file_stem()
        .ok_or_else(|| format!("invalid path '{path}': no filename"))?;
    let name = name
        .to_str()
        .ok_or_else(|| format!("invalid path '{path}': filename not utf-8 compliant"))?;
    let name = name.to_string().replace(" ", "_");
    Ok(FileInfo { name, image })
}

fn pixel_to_word(pixel: Rgba<u8>) -> String {
    let [r, g, b, _] = pixel.channels() else {
        unreachable!("rgba should have 4 channels");
    };
    format!("0x{r:02X}{g:02X}{b:02X}00")
}

fn image_to_instructions(vram: u32, screen_width: u32, image: Vec<(u32, u32, Rgba<u8>)>) -> String {
    image
        .into_iter()
        .map(|(x, y, pixel)| {
            let mov_y = format!("mov r1, {y}");
            let mul_screen = format!("mul r1, [{screen_width:#X}]");
            let add_vram = format!("add r1, [{vram:#X}]");
            let add_x = if x == 0 {
                String::new()
            } else {
                format!("add r1, {x}")
            };
            let add_position = format!("add r1, r0");
            let mov_pixel = format!("mov [r1], {}", pixel_to_word(pixel));
            [mov_y, mul_screen, add_vram, add_x, add_position, mov_pixel]
                .into_iter()
                .fold(String::new(), |acc, curr| {
                    if curr == String::new() {
                        acc
                    } else {
                        acc + &format!("\n    {curr}")
                    }
                })
        })
        .fold(String::new(), |acc, curr| acc + &curr)
}

fn main() -> io::Result<()> {
    let MyOptions {
        files,
        vram,
        screen_width,
        ..
    } = Options::parse_args_default_or_exit();

    files
        .into_iter()
        .map(|FileInfo { name, image }| {
            let image = image.to_rgba8();
            let (width, height) = image.dimensions();
            let image: Vec<_> = (0..width)
                .map(|x| {
                    (0..height)
                        .map(|y| (x, y, image.get_pixel(x, y).to_owned()))
                        .collect::<Vec<_>>()
                })
                .flatten()
                .filter(|(_, _, pixel)| {
                    let [_, _, _, alpha] = pixel.channels() else {
                        unreachable!("rgba should have 4 channels")
                    };
                    *alpha == 255
                })
                .collect::<Vec<_>>();
            FileInfo { name, image }
        })
        .map(|FileInfo { name, image }| FileInfo {
            image: format!(
                "{name}:\n{}",
                image_to_instructions(vram, screen_width, image)
            ),
            name,
        })
        .map(|FileInfo { name, image }| fs::write(format!("{name}.asm"), image))
        .collect::<io::Result<()>>()
}
