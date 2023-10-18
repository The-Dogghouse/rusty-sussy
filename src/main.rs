use std::{fs, io};
use std::io::Write;
use image::{DynamicImage, GenericImage, GenericImageView};
use std::process::Command;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Input image
    #[arg(short, long)]
    input: PathBuf,

    // Input image
    #[arg(short, long)]
    output: PathBuf,

    // Number of crew-mates to use horizontally
    #[arg(value_parser = clap::value_parser!(u32).range(1..512),short, long, default_value_t = 21)]
    width: u32
}

fn load_twerk_frames() -> Vec<DynamicImage> {
    let mut result: Vec<DynamicImage> = Vec::with_capacity(6);

    for n in 0..6 {
        result.push(image::open(format!("resources/twerk_imgs/{n}.png")).unwrap());
    }

    result
}

fn darken(component: u8) -> u8 {
    ((component as u32 * 2) / 3) as u8
}

fn main() {
    let args = Args::parse();

    let twerk_frames = load_twerk_frames();
    let twerk_size_x= twerk_frames[0].width();
    let twerk_size_y = twerk_frames[0].height();

    let input_image = image::open(args.input).expect("Failed to open input");
    let input_aspect = input_image.height() as f64 / input_image.width() as f64;
    let twerk_aspect = twerk_size_x as f64 / twerk_size_y as f64;

    let output_width = args.width; // Output width in crew-mates
    let output_height = (output_width as f64 * input_aspect * twerk_aspect).floor() as u32;
    let output_px = (output_width * twerk_size_x, output_height * twerk_size_y);

    let input_image_scaled = input_image.
        resize_exact(output_width, output_height, image::imageops::Nearest)
        .into_rgba8();
    let mut temp_files:Vec<String> = Vec::with_capacity(twerk_frames.len());

    for frame_num in 0..twerk_frames.len() as i32 {
        let mut background = DynamicImage::new_rgba8(output_px.0, output_px.1);
        for y in 0..output_height as i32 {
            for x in 0..output_width as i32 {
                let input_pixel = input_image_scaled.get_pixel(x as u32, y as u32);
                let [in_r, in_g, in_b, in_a] = input_pixel.0;

                // Skip transparent pixels
                if in_a == 0u8 {
                    continue;
                }

                // Re-implementing Python's % operator behavior with negative numbers
                let twerk_frame: DynamicImage = twerk_frames[((x - y + frame_num).rem_euclid(twerk_frames.len() as i32)) as usize].clone();
                let mut twerk_result: DynamicImage = twerk_frame.clone(); // Thanks for this Rust...

                for twerk_frame_p in twerk_frame.pixels() {
                    let [frame_r, frame_g, frame_b, _] = twerk_frame_p.2.0;
                    if frame_r == 214u8 && frame_g == 224u8 && frame_b == 240u8 {
                        twerk_result.put_pixel(twerk_frame_p.0, twerk_frame_p.1, image::Rgba([in_r, in_g, in_b, 255u8]))
                    } else if frame_r == 131u8 && frame_g == 148u8 && frame_b == 191u8 {
                        twerk_result.put_pixel(twerk_frame_p.0, twerk_frame_p.1, image::Rgba([darken(in_r), darken(in_g), darken(in_b), 255u8]))
                    }
                }

                background.copy_from(&twerk_result, x as u32 * twerk_size_x, y as u32 * twerk_size_y).unwrap();
            }
        }
        let tmp_path = format!("resources/tmp/sus-{frame_num}.png");
        background.save(tmp_path.clone()).unwrap();
        temp_files.push(tmp_path);
    }

    let o = Command::new("gifski")
        .args(["-o", &args.output.into_os_string().into_string().unwrap()])
        .args(["--repeat", "0"])
        .arg("resources/tmp/sus-0.png")
        .arg("resources/tmp/sus-1.png")
        .arg("resources/tmp/sus-2.png")
        .arg("resources/tmp/sus-3.png")
        .arg("resources/tmp/sus-4.png")
        .output()
        .expect("Failed to run gifski");

    io::stdout().write_all(&o.stdout).unwrap();
    io::stdout().write_all(&o.stderr).unwrap();

    for file in temp_files {
        fs::remove_file(file.clone())
            .expect(&*format!("Failed to delete temp file {file}"));
    }
}