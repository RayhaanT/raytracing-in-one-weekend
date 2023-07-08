use crate::math::clamp;
use std::io::{BufWriter, Write};

use crate::vec3::Color;

pub fn write_color<T: Write>(out: &mut BufWriter<T>, c: &Color) {
    out.write(format!("{} {} {}\n", c.x as u32, c.y as u32, c.z as u32).as_bytes())
        .unwrap();
}

pub fn process_color(color: &Color, samples_per_pixel: u32) -> Color {
    let scale = 1.0 / samples_per_pixel as f64;
    // sqrt is for gamma correction with gamma = 2
    let r = (color.x * scale).sqrt();
    let g = (color.y * scale).sqrt();
    let b = (color.z * scale).sqrt();

    let ir = clamp(r, 0.0, 0.999) * 256.0;
    let ig = clamp(g, 0.0, 0.999) * 256.0;
    let ib = clamp(b, 0.0, 0.999) * 256.0;

    Color::new(ir, ig, ib)
}
