mod color;
mod ray;
mod vec3;

use core::panic;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use crate::ray::Ray;
use crate::vec3::*;

fn hit_sphere(center: &Point3, radius: f64, r: &Ray) -> f64 {
    // Solving for parameters t such that r(t) is on the sphere,
    // i.e. r(t) has distance radius^2 from center
    // i.e. (r(t) - center) \cdot (r(t) - center) = radius^2
    // Below comes from above equation to solve for t w quadratic formula
    // Replacing b with h where b = 2h allows symbolic simplification
    let oc = r.origin - *center;
    let a = r.dir.length_sq();
    let h = dot(&oc, &r.dir);
    let c = oc.length_sq() - radius * radius;
    let discriminant = h * h - a * c;

    if discriminant < 0.0 {
        -1.0
    } else {
        (-h - discriminant.sqrt()) / a
    }
}

fn ray_color(r: &Ray) -> Color {
    let t = hit_sphere(&Point3::new(0.0, 0.0, -1.0), 0.5, r);
    if t > 0.0 {
        // Delta from sphere center to hit surface point
        let norm = normalized(&(r.at(t) - Vec3::new(0.0, 0.0, -1.0)));
        // Convert norm to color
        Color::new(norm.x + 1.0, norm.y + 1.0, norm.z + 1.0) * 0.5
    } else {
        let unit_dir = vec3::normalized(&r.dir);
        let t = 0.5 * (unit_dir.y + 1.0);

        // Linear blend from white to blue
        Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t
    }
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

    let origin = Point3::new(0.0, 0.0, 0.0);
    let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height, 0.0);
    let depth = Vec3::new(0.0, 0.0, focal_length);
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
