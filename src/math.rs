use rand::{rngs::ThreadRng, Rng};
use std::f64::consts::PI;

pub fn deg_to_rad(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

pub fn clamp(x: f64, min: f64, max: f64) -> f64 {
    if x < min {
        min
    } else if max < x {
        max
    } else {
        x
    }
}

pub fn rand_unit(r: &mut ThreadRng) -> f64 {
    r.gen_range(0.0..1.0)
}
