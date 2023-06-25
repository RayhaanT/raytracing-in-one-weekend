use crate::vec3::{Point3, Vec3};

pub struct Ray {
    pub origin: Point3,
    pub dir: Vec3,
}

impl Ray {
    fn at(&self, t: f64) -> Point3 {
        self.origin + self.dir * t
    }
}
