use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::process;
use std::str::FromStr;
use std::vec::Vec;

// Exit node lists here, https://check.torproject.org/exit-addresses
// Sample entry: 
//
// ExitNode DFA97DED4CE79FF6F31DAF917C2810CCE8729E9D
// Published 2022-12-28 14:06:34
// LastStatus 2022-12-29 04:00:00
// ExitAddress 185.244.xxx.xxx 2022-12-29 04:07:27


fn main() {
    let mut out = String::new();
    let mut files = Vec::new();
    let mut seen = HashMap::new();
    let exit = Regex::new(r"^ExitAddress ([^ ]*) ").unwrap();

    for (i, arg) in std::env::args().enumerate() {
        if i == 0 {
            continue;
        } else if arg == "-out" {
            out = std::env::args().nth(i + 1).unwrap();
        } else {
            files.push(arg);
        }
    }

    if files.is_empty() || out.is_empty() {
        println!("Usage: torexit -out <png> <file> [<file>...]");
        process::exit(1);
    }

    for (i, name) in files.iter().enumerate() {
        if let Err(e) = parse(i, name, &mut seen, &exit) {
            println!("Failed to parse file {}: {}", name, e);
            process::exit(1);
        }
    }

    if seen.is_empty() {
        println!("No exit addresses found in any of the data files");
        process::exit(1);
    }

    let mut keys: Vec<String> = seen.keys().map(|s| s.to_string()).collect();
    keys.sort_by(|a, b| {
        if seen[a][0] == seen[b][0] {
            seen[a].len().cmp(&seen[b].len())
        } else {
            seen[a][0].cmp(&seen[b][0])
        }
    });

    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut cs = Vec::new();

    for (i, k) in keys.iter().enumerate() {
        xs.push(i as u32);
        ys.push(seen[k].len() as u32);
        cs.push(color::Rgb([
            (seen[k][0] * 255 / files.len()) as u8,
            0,
            (255 - seen[k][0] * 255 / files.len()) as u8,
        ]));
    }

    let path = Path::new(&out);
    let file = File::create(&path).unwrap();
    let w = buf_image::ImageBuffer::new(xs.len() as u32, *ys.iter().max().unwrap());
    let mut img = image::ImageRgb8(w);

    for (x, y, c) in xs.iter().zip(ys.iter()).zip(cs.iter()) {
        img.put_pixel(*x, *y, *c);
    }

   
    let mut png_encoder = png::Encoder::new(file, xs.len() as u32, *ys.iter().max().unwrap());
    png_encoder.set_color(png::ColorType::RGB);
    png_encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = png_encoder.write_header().unwrap();
    png_writer.write_image_data(&img.into_raw()).unwrap();


}