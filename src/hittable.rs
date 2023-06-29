use crate::material::{Lambertian, Material};
use crate::ray::Ray;
use crate::vec3::*;
use std::rc::Rc;

#[derive(Clone)]
pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
    pub mat: Rc<dyn Material>,
}

impl HitRecord {
    fn get_face_normal(r: &Ray, outward_normal: &Vec3, front_face: &mut bool) -> Vec3 {
        // If the norm faces the same way as the norm, we're inside
        *front_face = dot(&r.dir, outward_normal) < 0.0;
        if *front_face {
            *outward_normal
        } else {
            -*outward_normal
        }
    }

    pub fn new(
        p: Point3,
        t: f64,
        r: &Ray,
        outward_normal: &Vec3,
        mat: Rc<dyn Material>,
    ) -> HitRecord {
        let mut front_face = false;
        let normal = HitRecord::get_face_normal(r, outward_normal, &mut front_face);

        HitRecord {
            p,
            t,
            front_face,
            normal,
            mat,
        }
    }

    pub fn blank() -> HitRecord {
        HitRecord {
            p: Point3::new(0.0, 0.0, 0.0),
            t: 0.0,
            front_face: false,
            normal: Vec3::new(0.0, 0.0, 0.0),
            mat: Rc::new(Lambertian {
                albedo: Color::new(0.0, 0.0, 0.0),
            }),
        }
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool;
}

pub struct HittableList {
    pub objects: Vec<Rc<dyn Hittable>>,
}

impl HittableList {
    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add(&mut self, new_obj: Rc<dyn Hittable>) {
        self.objects.push(new_obj);
    }

    pub fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        let mut temp_rec = rec.clone();
        let mut hit_anything = false;

        self.objects.iter().fold(t_max, |nearest, hittable| {
            if hittable.hit(r, t_min, nearest, &mut temp_rec) {
                hit_anything = true;
                *rec = temp_rec.clone();
                temp_rec.t
            } else {
                nearest
            }
        });

        hit_anything
    }
}
