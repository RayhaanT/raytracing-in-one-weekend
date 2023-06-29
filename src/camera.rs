use crate::{math::deg_to_rad, ray::Ray, vec3::*};

pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
    x: Vec3,
    y: Vec3,
    z: Vec3,
    lens_radius: f64,
}

impl Camera {
    // vfov = v field of view in degrees
    pub fn new(
        pos: Point3,
        look_at: Point3,
        up: Vec3,
        vfov: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
    ) -> Camera {
        let vfovr = deg_to_rad(vfov);
        let h = (vfovr / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = viewport_height * aspect_ratio;

        let z = normalized(pos - look_at);
        let x = normalized(cross(&up, &z));
        let y = cross(&z, &x);

        let origin = pos;
        let horizontal = x * viewport_width * focus_dist;
        let vertical = y * viewport_height * focus_dist;

        Camera {
            origin,
            horizontal,
            vertical,
            lower_left_corner: origin - (horizontal / 2.0) - (vertical / 2.0) - z * focus_dist,
            x,
            y,
            z,
            lens_radius: aperture / 2.0,
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        // depth of field scattering
        let rd = Vec3::rand_in_disk() * self.lens_radius;
        let offset = self.x * rd.x + self.y * rd.y;

        // Ray from camera to a point on the viewport surface
        Ray {
            origin: self.origin + offset,
            dir: self.lower_left_corner + self.horizontal * u + self.vertical * v
                - self.origin
                - offset,
        }
    }
}
