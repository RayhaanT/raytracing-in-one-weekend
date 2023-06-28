use crate::math::clamp;
use std::io::{BufWriter, Write};

use crate::vec3::Color;

pub fn write_color<T: Write>(out: &mut BufWriter<T>, color: &Color, samples_per_pixel: u32) {
    let ir: u32 = (color.x * 255.99) as u32;
    let ig: u32 = (color.y * 255.99) as u32;
    let ib: u32 = (color.z * 255.99) as u32;

    let scale = 1.0 / samples_per_pixel as f64;
    let r = color.x * scale;
    let g = color.y * scale;
    let b = color.z * scale;

    let ir: u32 = (clamp(r, 0.0, 0.999) * 256.0) as u32;
    let ig: u32 = (clamp(g, 0.0, 0.999) * 256.0) as u32;
    let ib: u32 = (clamp(b, 0.0, 0.999) * 256.0) as u32;

    out.write(format!("{} {} {}\n", ir, ig, ib).as_bytes())
        .unwrap();
}
