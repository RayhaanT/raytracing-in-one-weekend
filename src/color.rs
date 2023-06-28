use crate::math::clamp;
use std::io::{BufWriter, Write};

use crate::vec3::Color;

pub fn write_color<T: Write>(out: &mut BufWriter<T>, color: &Color, samples_per_pixel: u32) {
    let scale = 1.0 / samples_per_pixel as f64;
    // sqrt is for gamma correction with gamma = 2
    let r = (color.x * scale).sqrt();
    let g = (color.y * scale).sqrt();
    let b = (color.z * scale).sqrt();

    let ir: u32 = (clamp(r, 0.0, 0.999) * 256.0) as u32;
    let ig: u32 = (clamp(g, 0.0, 0.999) * 256.0) as u32;
    let ib: u32 = (clamp(b, 0.0, 0.999) * 256.0) as u32;

    out.write(format!("{} {} {}\n", ir, ig, ib).as_bytes())
        .unwrap();
}
