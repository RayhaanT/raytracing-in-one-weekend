use crate::{math::deg_to_rad, ray::Ray, vec3::*};

pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    // vfov = v field of view in degrees
    pub fn new(pos: Point3, look_at: Point3, up: Vec3, vfov: f64, aspect_ratio: f64) -> Camera {
        let vfovr = deg_to_rad(vfov);
        let h = (vfovr / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = viewport_height * aspect_ratio;

        let z = normalized(pos - look_at);
        let x = normalized(cross(&up, &z));
        let y = cross(&z, &x);

        let origin = pos;
        let horizontal = x * viewport_width;
        let vertical = y * viewport_height;

        Camera {
            origin,
            horizontal,
            vertical,
            lower_left_corner: origin - (horizontal / 2.0) - (vertical / 2.0) - z,
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        // Ray from camera to a point on the viewport surface
        Ray {
            origin: self.origin,
            dir: self.lower_left_corner + self.horizontal * u + self.vertical * v - self.origin,
        }
    }
}
