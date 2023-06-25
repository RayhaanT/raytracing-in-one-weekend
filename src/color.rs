use std::io::{BufWriter, Write};

use crate::vec3::Color;

pub fn write_color<T: Write>(out: &mut BufWriter<T>, color: &Color) {
    let ir: u32 = (color.x * 255.99) as u32;
    let ig: u32 = (color.y * 255.99) as u32;
    let ib: u32 = (color.z * 255.99) as u32;

    out.write(format!("{} {} {}\n", ir, ig, ib).as_bytes())
        .unwrap();
}
