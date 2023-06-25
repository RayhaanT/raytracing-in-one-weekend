mod color;
mod vec3;

use core::panic;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use crate::vec3::Color;

fn main() {
    let image_width = 256;
    let image_height = 256;

    let path = Path::new("image.ppm");
    let display = path.display();

    let mut file = match File::create(path) {
        Ok(f) => BufWriter::new(f),
        Err(why) => panic!("Could not create {}: {}", display, why),
    };

    let header = format!("P3\n{} {}\n255\n", image_width, image_height);
    file.write(header.as_bytes()).unwrap();

    for i in (0..image_height).rev() {
        eprint!("\rScanlines remaining: {} \n", i);
        io::stdout().flush().unwrap();

        for j in 0..image_width {
            let pixel_color = Color {
                x: i as f64 / (image_height as f64 - 1.0),
                y: j as f64 / (image_width as f64 - 1.0),
                z: 0.25,
            };

            crate::color::write_color(&mut file, &pixel_color);
        }
    }

    eprint!("\nDone.\n");
}
