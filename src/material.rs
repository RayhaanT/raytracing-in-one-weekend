use crate::{hittable::HitRecord, math::rand_unit, ray::Ray, vec3::*};

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
        _r_in: &Ray,
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
    pub fuzz: f64,
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
        *scattered = Ray::new(rec.p, reflected + Vec3::rand_in_sphere() * self.fuzz);
        *attenuation = self.albedo;

        // Don't scatter if inside
        dot(&scattered.dir, &rec.normal) > 0.0
    }
}

impl Metal {
    pub fn new(r: f64, g: f64, b: f64, fuzz: f64) -> Metal {
        Metal {
            albedo: Color::new(r, g, b),
            fuzz,
        }
    }
}

pub struct Dielectric {
    ir: f64,
}

impl Material for Dielectric {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        // Air's index of refraction is 1. Ratio depends on if ray is
        // hitting the object from inside (mat -> air) or outside (air -> mat)
        let ir_ratio = if rec.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let unit_dir = normalized(r_in.dir);

        // Check for total internal reflection
        let cos_theta = dot(&-unit_dir, &rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let direction;

        if ir_ratio * sin_theta > 1.0 || Dielectric::reflectance(cos_theta, ir_ratio) > rand_unit()
        {
            // Internal reflection
            direction = unit_dir.reflect(rec.normal);
        } else {
            // Refraction
            direction = Vec3::refract(&unit_dir, &rec.normal, ir_ratio);
        }

        *scattered = Ray::new(rec.p, direction);
        *attenuation = Color::new(1.0, 1.0, 1.0);
        true
    }
}

impl Dielectric {
    pub fn new(ir: f64) -> Dielectric {
        Dielectric { ir }
    }

    fn reflectance(cos: f64, ir_ratio: f64) -> f64 {
        // Schlick's reflectance approximation
        let r0 = (1.0 - ir_ratio) / (1.0 + ir_ratio);
        let r0s = r0 * r0;
        r0s + (1.0 - r0s) * (1.0 - cos).powi(5)
    }
}
