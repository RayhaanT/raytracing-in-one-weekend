mod camera;
mod color;
mod hittable;
mod material;
mod math;
mod ray;
mod sphere;
mod vec3;

use core::panic;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::rc::Rc;

use hittable::HitRecord;

use crate::camera::Camera;
use crate::hittable::HittableList;
use crate::material::*;
use crate::math::rand_unit;
use crate::ray::Ray;
use crate::sphere::Sphere;
use crate::vec3::*;

fn ray_color(r: &Ray, world: &HittableList, depth: u32) -> Color {
    let mut rec = HitRecord::blank();

    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    // 0.001 min Removes shadow acne. Don't want bounced rays colliding
    // with the same surface at t = 1e-8 from fp inaccuracies
    if world.hit(r, 0.001, f64::MAX, &mut rec) {
        let mut scattered = Ray::blank();
        let mut attenuation = Color::new(0.0, 0.0, 0.0);

        if rec.mat.scatter(r, &rec, &mut attenuation, &mut scattered) {
            attenuation * ray_color(&scattered, world, depth - 1)
        } else {
            Color::new(0.0, 0.0, 0.0)
        }
    } else {
        let unit_dir = vec3::normalized(r.dir);
        let t = 0.5 * (unit_dir.y + 1.0);

        // Linear blend from white to blue
        Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t
    }
}

fn main() {
    // Output file
    let path = Path::new("image.ppm");
    let display = path.display();

    // Image consts
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let image_height = (image_width as f64 / aspect_ratio) as u32;
    let samples_per_pixel = 100;
    let max_bounce_depth = 50;

    // Materials
    let material_ground = Rc::new(Lambertian::new(0.8, 0.8, 0.6));
    // let material_center = Rc::new(Lambertian::new(0.7, 0.3, 0.3));
    let material_center = Rc::new(Dielectric::new(1.5));
    let material_left = Rc::new(Metal::new(0.8, 0.8, 0.8, 0.3));
    let material_right = Rc::new(Metal::new(0.8, 0.6, 0.2, 0.9));
    // let material_right = Rc::new(Dielectric::new(1.5));

    // World
    let mut world = HittableList {
        objects: Vec::new(),
    };
    world.add(Rc::new(Sphere::new(
        Point3::new(0.0, 0.0, -1.0),
        0.5,
        material_center.clone(),
    )));
    world.add(Rc::new(Sphere::new(
        Point3::new(0.0, -1000.5, -1.0),
        1000.0,
        material_ground.clone(),
    )));
    world.add(Rc::new(Sphere::new(
        Point3::new(-1.0, 0.0, -1.0),
        0.5,
        material_left.clone(),
    )));
    world.add(Rc::new(Sphere::new(
        Point3::new(1.0, 0.0, -1.0),
        0.5,
        material_right.clone(),
    )));

    // Generate balls
    let radius = 0.2;
    for x in -10..10 {
        for z in -10..10 {
            let center = Point3::new(
                x as f64 + 0.9 * rand_unit(),
                -0.5 + radius,
                z as f64 + 0.9 * rand_unit(),
            );

            let material: Rc<dyn Material>;
            let mat_type = rand_unit();

            if mat_type < 0.7 {
                material = Rc::new(Lambertian {
                    albedo: Color::rand() * Color::rand(),
                });
            } else if mat_type < 0.9 {
                material = Rc::new(Metal {
                    albedo: Color::rand_range(0.5, 1.0),
                    fuzz: rand_unit(),
                });
            } else {
                material = Rc::new(Dielectric::new(rand_unit() + 1.0));
            }

            world.add(Rc::new(Sphere::new(center, radius, material.clone())));
        }
    }

    // Camera
    let camera_pos = Point3::new(3.0, 1.0, 2.0);
    let look_at = Point3::new(0.0, 0.0, -1.0);
    let world_up = Point3::new(0.0, 1.0, 0.0);
    let dist_to_focus = (camera_pos - look_at).length();
    let aperture = 0.1;

    let cam = Camera::new(
        camera_pos,
        look_at,
        world_up,
        30.0,
        aspect_ratio,
        aperture,
        dist_to_focus,
    );

    let mut file = match File::create(path) {
        Ok(f) => BufWriter::new(f),
        Err(why) => panic!("Could not create {}: {}", display, why),
    };

    let header = format!("P3\n{} {}\n255\n", image_width, image_height);
    file.write(header.as_bytes()).unwrap();

    for i in (0..image_height).rev() {
        eprint!("\rScanlines remaining: {} \n", i);
        io::stdout().flush().unwrap();

        // Cast a ray at each pixel in the image
        for j in 0..image_width {
            let mut pixel_color = Color::new(0.0, 0.0, 0.0);

            for _ in 0..samples_per_pixel {
                let u = (j as f64 + rand_unit()) / (image_width as f64 - 1.0);
                let v = (i as f64 + rand_unit()) / (image_height as f64 - 1.0);

                let ray = cam.get_ray(u, v);
                pixel_color += ray_color(&ray, &world, max_bounce_depth);
            }

            crate::color::write_color(&mut file, &pixel_color, samples_per_pixel);
        }
    }

    eprint!("\nDone.\n");
}
