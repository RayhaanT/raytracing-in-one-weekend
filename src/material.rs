use crate::{hittable::HitRecord, ray::Ray, vec3::*};

pub trait Material {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool;
}

pub struct Lambertian {
    pub albedo: Color,
}

impl Material for Lambertian {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        // Lambertian scattering for diffuse materials
        let mut scatter_direction = rec.normal + Vec3::rand_unit();

        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        *scattered = Ray::new(rec.p, scatter_direction);
        *attenuation = self.albedo;
        true
    }
}

impl Lambertian {
    pub fn new(r: f64, g: f64, b: f64) -> Lambertian {
        Lambertian {
            albedo: Color::new(r, g, b),
        }
    }
}

pub struct Metal {
    pub albedo: Color,
}

impl Material for Metal {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        let reflected = normalized(r_in.dir).reflect(rec.normal);
        *scattered = Ray::new(rec.p, reflected);
        *attenuation = self.albedo;

        // Don't scatter if inside
        dot(&scattered.dir, &rec.normal) > 0.0
    }
}

impl Metal {
    pub fn new(r: f64, g: f64, b: f64) -> Metal {
        Metal {
            albedo: Color::new(r, g, b),
        }
    }
}
