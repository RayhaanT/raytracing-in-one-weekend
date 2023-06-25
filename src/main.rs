mod color;
mod ray;
mod vec3;

use core::panic;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use crate::ray::Ray;
use crate::vec3::{Color, Point3, Vec3};

fn ray_color(r: &Ray) -> Color {
    let unit_dir = vec3::normalized(&r.dir);
    let t = 0.5 * (unit_dir.y + 1.0);

    // Linear blend from white to blue
    Color {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    } * (1.0 - t)
        + Color {
            x: 0.5,
            y: 0.7,
            z: 1.0,
        } * t
}

fn main() {
    // Output file
    let path = Path::new("image.ppm");
    let display = path.display();

    // Image consts
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let image_height = (image_width as f64 / aspect_ratio) as u32;

    // Camera consts
    let viewport_height = 2.0;
    let viewport_width = aspect_ratio * viewport_height;
    let focal_length = 1.0;

    let origin = Point3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let horizontal = Vec3 {
        x: viewport_width,
        y: 0.0,
        z: 0.0,
    };
    let vertical = Vec3 {
        x: 0.0,
        y: viewport_height,
        z: 0.0,
    };
    let depth = Vec3 {
        x: 0.0,
        y: 0.0,
        z: focal_length,
    };
    let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - depth;

    let mut file = match File::create(path) {
        Ok(f) => BufWriter::new(f),
        Err(why) => panic!("Could not create {}: {}", display, why),
    };

    let header = format!("P3\n{} {}\n255\n", image_width, image_height);
    file.write(header.as_bytes()).unwrap();

    for i in (0..image_height).rev() {
        eprint!("\rScanlines remaining: {} \n", i);
        io::stdout().flush().unwrap();

        // Cast a ray at each pixel in the image
        for j in 0..image_width {
            let u = (j as f64) / (image_width as f64 - 1.0);
            let v = (i as f64) / (image_height as f64 - 1.0);

            let r = Ray {
                origin,
                dir: lower_left_corner + horizontal * u + vertical * v - origin,
            };
            let pixel_color = ray_color(&r);

            crate::color::write_color(&mut file, &pixel_color);
        }
    }

    eprint!("\nDone.\n");
}
