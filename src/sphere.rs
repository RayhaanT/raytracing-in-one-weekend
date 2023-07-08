use std::sync::Arc;

use crate::hittable::*;
use crate::material::Material;
use crate::vec3::*;

pub struct Sphere {
    center: Point3,
    radius: f64,
    mat: Arc<dyn Material>,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, mat: Arc<dyn Material>) -> Sphere {
        Sphere {
            center,
            radius,
            mat,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &crate::ray::Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        // Solving for parameters t such that r(t) is on the sphere,
        // i.e. r(t) has distance radius^2 from center
        // i.e. (r(t) - center) \cdot (r(t) - center) = radius^2
        // Below comes from above equation to solve for t w quadratic formula
        // Replacing b with h where b = 2h allows symbolic simplification
        let oc = r.origin - self.center;
        let a = r.dir.length_sq();
        let h = dot(&oc, &r.dir);
        let c = oc.length_sq() - self.radius * self.radius;
        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            return false;
        }

        // Need to find t in valid range (t_min, t_max)
        let sqrtd = discriminant.sqrt();
        let mut root = (-h - sqrtd) / a;
        if root < t_min || root > t_max {
            root = (-h + sqrtd) / a;
            if root < t_min || root > t_max {
                return false;
            }
        }

        let p = r.at(root);
        let out_norm = (p - self.center) / self.radius;
        *rec = HitRecord::new(p, root, r, &out_norm, self.mat.clone());

        true
    }
}
