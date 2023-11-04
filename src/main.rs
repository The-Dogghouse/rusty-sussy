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

    // Output image
    #[arg(short, long)]
    output: PathBuf,

    // Number of crew-mates to use horizontally
    #[arg(value_parser = clap::value_parser ! (u32).range(1..512), short, long, default_value_t = 21)]
    width: u32,
}

fn darken(component: u8) -> u8 {
    ((component as u32 * 2) / 3) as u8
}

fn main() {
    let twerk_frames = [
        image::load_from_memory(include_bytes!("../resources/twerk_imgs/0.png")).unwrap(),
        image::load_from_memory(include_bytes!("../resources/twerk_imgs/1.png")).unwrap(),
        image::load_from_memory(include_bytes!("../resources/twerk_imgs/2.png")).unwrap(),
        image::load_from_memory(include_bytes!("../resources/twerk_imgs/3.png")).unwrap(),
        image::load_from_memory(include_bytes!("../resources/twerk_imgs/4.png")).unwrap(),
        image::load_from_memory(include_bytes!("../resources/twerk_imgs/5.png")).unwrap()
    ];

    let args = Args::parse();

    let twerk_size_x = twerk_frames[0].width();
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
    let mut temp_files: Vec<String> = Vec::with_capacity(twerk_frames.len());

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

                for twerk_frame_p in twerk_frame.pixels() {
                    let result_x = twerk_frame_p.0 + x as u32 * twerk_size_x;
                    let result_y = twerk_frame_p.1 + y as u32 * twerk_size_y;
                    let [frame_r, frame_g, frame_b, _] = twerk_frame_p.2.0;
                    if frame_r == 214u8 && frame_g == 224u8 && frame_b == 240u8 {
                        background.put_pixel(result_x, result_y, image::Rgba([in_r, in_g, in_b, 255u8]))
                    } else if frame_r == 131u8 && frame_g == 148u8 && frame_b == 191u8 {
                        background.put_pixel(result_x, result_y, image::Rgba([darken(in_r), darken(in_g), darken(in_b), 255u8]))
                    }
                }
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
        .output();

    // For some reason, we get an error code from `Command`,
    // if we're not in a full shell env, but the command still works
    // so, do this dance so we don't crash if we get a benign error
    if o.is_ok() {
        let r = o.unwrap();
        io::stdout().write_all(&r.stdout).unwrap();
        io::stdout().write_all(&r.stderr).unwrap();
    } else {
        eprintln!("Error running gifski: '{}'", o.err().unwrap());
    }

    for file in temp_files {
        fs::remove_file(file.clone())
            .expect(&*format!("Failed to delete temp file {file}"));
    }
}
