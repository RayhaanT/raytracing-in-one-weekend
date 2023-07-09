mod camera;
mod color;
mod hittable;
mod material;
mod math;
mod ray;
mod sphere;
mod vec3;

use core::panic;
use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;

use hittable::HitRecord;

use crate::camera::Camera;
use crate::hittable::HittableList;
use crate::material::*;
use crate::math::rand_unit;
use crate::ray::Ray;
use crate::sphere::Sphere;
use crate::vec3::*;

struct Scanline {
    buffer: Vec<Vec<Color>>,
    index: usize,
}

impl Scanline {
    pub fn new(width: usize, batch_size: usize, index: usize) -> Scanline {
        Scanline {
            index,
            buffer: vec![vec![Color::new(0.0, 0.0, 0.0); width]; batch_size],
        }
    }
}

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
    // Threading
    let args: Vec<String> = env::args().collect();
    let threads;
    if args.len() > 1 {
        threads = args[1].strip_prefix("-j").unwrap().parse().unwrap();
    } else {
        threads = thread::available_parallelism().unwrap().get();
    }
    eprintln!("Running on {} threads", threads);

    // Output file
    let path = Path::new("image.ppm");
    let display = path.display();

    // Image consts
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 1920;
    let image_height = (image_width as f64 / aspect_ratio) as usize;
    let samples_per_pixel = 100;
    let max_bounce_depth = 50;

    // Materials
    let material_ground = Arc::new(Lambertian::new(0.8, 0.8, 0.6));
    // let material_center = Rc::new(Lambertian::new(0.7, 0.3, 0.3));
    let material_center = Arc::new(Dielectric::new(1.5));
    let material_left = Arc::new(Metal::new(0.8, 0.8, 0.8, 0.3));
    let material_right = Arc::new(Metal::new(0.8, 0.6, 0.2, 0.9));
    // let material_right = Rc::new(Dielectric::new(1.5));

    // World
    let mut world = HittableList {
        objects: Vec::new(),
    };
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 0.0, -1.0),
        0.5,
        material_center.clone(),
    )));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.5, -1.0),
        1000.0,
        material_ground.clone(),
    )));
    world.add(Arc::new(Sphere::new(
        Point3::new(-1.0, 0.0, -1.0),
        0.5,
        material_left.clone(),
    )));
    world.add(Arc::new(Sphere::new(
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

            let material: Arc<dyn Material>;
            let mat_type = rand_unit();

            if mat_type < 0.7 {
                material = Arc::new(Lambertian {
                    albedo: Color::rand() * Color::rand(),
                });
            } else if mat_type < 0.9 {
                material = Arc::new(Metal {
                    albedo: Color::rand_range(0.5, 1.0),
                    fuzz: rand_unit(),
                });
            } else {
                material = Arc::new(Dielectric::new(rand_unit() + 1.0));
            }

            world.add(Arc::new(Sphere::new(center, radius, material.clone())));
        }
    }

    // Camera
    let camera_pos = Point3::new(3.0, 1.0, 2.0);
    let look_at = Point3::new(0.0, 0.0, -1.0);
    let world_up = Point3::new(0.0, 1.0, 0.0);
    let dist_to_focus = (camera_pos - look_at).length();
    let aperture = 0.1;

    let cam = Arc::new(Camera::new(
        camera_pos,
        look_at,
        world_up,
        30.0,
        aspect_ratio,
        aperture,
        dist_to_focus,
    ));
    let mut file = match File::create(path) {
        Ok(f) => BufWriter::new(f),
        Err(why) => panic!("Could not create {}: {}", display, why),
    };

    let header = format!("P3\n{} {}\n255\n", image_width, image_height);
    file.write(header.as_bytes()).unwrap();

    let iworld = Arc::new(world);
    let (tx, rx) = mpsc::channel::<Scanline>();

    let mut requested = 0;
    let batch_size = 10;
    let mut join_handles = Vec::new();

    // Spawn the original thread group
    for _ in (0..threads).rev() {
        // Need to clone Arcs because of lifetimes
        let cam_temp = cam.clone();
        let world_temp = iworld.clone();
        let tx_temp = tx.clone();

        join_handles.push(thread::spawn(move || {
            render_line(
                requested,
                batch_size,
                cam_temp,
                world_temp,
                image_height,
                image_width,
                samples_per_pixel,
                max_bounce_depth,
                tx_temp,
            )
        }));

        requested += batch_size;
    }

    // Receive scan lines and write into overall buffer
    let mut image_buffer: Vec<Vec<Color>> =
        vec![vec![Color::new(0.0, 0.0, 0.0); image_width]; image_height];

    // As each line is received, launch a new thread for the next line
    // Spawn a thread for each scan line instead of blocking the whole
    // image out into a stripe for each thread because stripes evaluate at
    // different speeds depending on the scene, so some threads exit early
    // This ensures 100% core utilization at the cost of more thread spawns/Arc clones
    while requested < image_height {
        let cam_temp = cam.clone();
        let world_temp = iworld.clone();
        let tx_temp = tx.clone();

        let batch = if requested + batch_size < image_height {
            batch_size
        } else {
            image_height - requested
        };

        let received = rx.recv().unwrap();
        thread::spawn(move || {
            render_line(
                requested,
                batch,
                cam_temp,
                world_temp,
                image_height,
                image_width,
                samples_per_pixel,
                max_bounce_depth,
                tx_temp,
            )
        });

        for (y, line) in received.buffer.iter().enumerate() {
            for (x, c) in line.iter().enumerate() {
                image_buffer[received.index + y][x] = *c;
            }
        }

        requested += batch;
    }

    // Receive the final thread group since there will still be 1 live batch
    for _ in 0..threads {
        let received = rx.recv().unwrap();
        for (y, line) in received.buffer.iter().enumerate() {
            for (x, c) in line.iter().enumerate() {
                image_buffer[received.index + y][x] = *c;
            }
        }
    }

    drop(tx);
    assert!(rx.try_iter().next().is_none());

    // Write into ppm file
    for row in image_buffer.iter().rev() {
        for pixel in row {
            crate::color::write_color(&mut file, &pixel);
        }
    }

    eprint!("\nDone.\n");
}

fn render_line(
    start: usize,
    batch_size: usize,
    cam: Arc<Camera>,
    world: Arc<HittableList>,
    image_height: usize,
    image_width: usize,
    samples: u32,
    bounce_depth: u32,
    tx: Sender<Scanline>,
) {
    let mut scanline = Scanline::new(image_width, batch_size, start);

    for i in 0..batch_size {
        let line = start + i;
        eprint!("Rendering scanline : {} \n", line);
        io::stdout().flush().unwrap();

        // Cast a ray at each pixel in the image
        for j in 0..image_width {
            let mut pixel_color = Color::new(0.0, 0.0, 0.0);

            for _ in 0..samples {
                let u = (j as f64 + rand_unit()) / (image_width as f64 - 1.0);
                let v = (line as f64 + rand_unit()) / (image_height as f64 - 1.0);

                let ray = cam.get_ray(u, v);
                pixel_color += ray_color(&ray, &world, bounce_depth);
            }

            scanline.buffer[i][j] = crate::color::process_color(&pixel_color, samples);
        }
    }

    tx.send(scanline).unwrap();
}
