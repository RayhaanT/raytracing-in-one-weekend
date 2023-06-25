use core::panic;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    let image_width = 256;
    let image_height = 256;

    let path = Path::new("image.ppm");
    let display = path.display();

    let mut file = match File::create(path) {
        Ok(f) => BufWriter::new(f),
        Err(why) => panic!("Could not create {}: {}", display, why),
    };

    let header = format!("P4\n{} {}\n255\n", image_width, image_height);
    file.write(header.as_bytes()).unwrap();
    for i in 0..image_height {
        for j in 0..image_width {
            let r: f64 = i as f64 / (image_height as f64 - 1.0);
            let g: f64 = j as f64 / (image_width as f64 - 1.0);
            let b = 0.25;

            let ir: u32 = (r * 255.99) as u32;
            let ig: u32 = (g * 255.99) as u32;
            let ib: u32 = (b * 255.99) as u32;

            file.write(format!("{} {} {}\n", ir, ig, ib).as_bytes())
                .unwrap();
        }
    }
}
