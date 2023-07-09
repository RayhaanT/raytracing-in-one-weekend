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

#[derive(Clone, Copy)]
struct Image {
    height: u32,
    width: u32,
    samples: u32,
    bounce_depth: u32,
}

struct Tile {
    buffer: Vec<Vec<Color>>,
    start_x: usize,
    start_y: usize,
}

impl Tile {
    pub fn new(height: u32, width: u32, index: u32, image: Image) -> Tile {
        let tiles_wide = (image.width as f32 / width as f32).ceil() as u32;
        let start_y = ((index / tiles_wide) * height) as usize;
        let start_x = ((index % tiles_wide) * width) as usize;

        let h = if start_y as u32 + height < image.height {
            height
        } else {
            image.height - start_y as u32
        };
        let w = if start_x as u32 + width < image.width {
            width
        } else {
            image.width - start_x as u32
        };

        Tile {
            start_x,
            start_y,
            buffer: vec![vec![Color::new(0.0, 0.0, 0.0); w as usize]; h as usize],
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
    let image = Image {
        width: image_width,
        height: (image_width as f64 / aspect_ratio) as u32,
        samples: 100,
        bounce_depth: 50,
    };

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

    let header = format!("P3\n{} {}\n255\n", image.width, image.height);
    file.write(header.as_bytes()).unwrap();

    let iworld = Arc::new(world);
    let (tx, rx) = mpsc::channel::<Tile>();

    let mut requested = 0;
    let tile_width = 10;
    let tile_height = 10;
    let total_tiles = (image.width as f32 / tile_width as f32).ceil() as u32
        * (image.height as f32 / tile_height as f32).ceil() as u32;
    let mut join_handles = Vec::new();

    // Spawn the original thread group
    for _ in (0..threads).rev() {
        // Need to clone Arcs because of lifetimes
        let cam_temp = cam.clone();
        let world_temp = iworld.clone();
        let tx_temp = tx.clone();

        join_handles.push(thread::spawn(move || {
            render_tile(
                Tile::new(tile_height, tile_width, requested, image),
                cam_temp,
                world_temp,
                image,
                tx_temp,
            )
        }));

        requested += 1;
    }

    // Receive scan lines and write into overall buffer
    let mut image_buffer: Vec<Vec<Color>> =
        vec![vec![Color::new(0.0, 0.0, 0.0); image.width as usize]; image.height as usize];

    // As each line is received, launch a new thread for the next line
    // Spawn a thread for each scan line instead of blocking the whole
    // image out into a stripe for each thread because stripes evaluate at
    // different speeds depending on the scene, so some threads exit early
    // This ensures 100% core utilization at the cost of more thread spawns/Arc clones
    while requested < total_tiles {
        let cam_temp = cam.clone();
        let world_temp = iworld.clone();
        let tx_temp = tx.clone();

        let received = rx.recv().unwrap();
        thread::spawn(move || {
            render_tile(
                Tile::new(tile_height, tile_width, requested, image),
                cam_temp,
                world_temp,
                image,
                tx_temp,
            )
        });

        for (y, row) in received.buffer.iter().enumerate() {
            for (x, c) in row.iter().enumerate() {
                image_buffer[received.start_y + y][received.start_x + x] = *c;
            }
        }

        requested += 1;
    }

    // Receive the final thread group since there will still be 1 live batch
    for _ in 0..threads {
        let received = rx.recv().unwrap();
        for (y, row) in received.buffer.iter().enumerate() {
            for (x, c) in row.iter().enumerate() {
                image_buffer[received.start_y + y][received.start_x + x] = *c;
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

fn render_tile(
    mut tile: Tile,
    cam: Arc<Camera>,
    world: Arc<HittableList>,
    image: Image,
    tx: Sender<Tile>,
) {
    let h = tile.buffer.len();
    let w = tile.buffer.first().unwrap().len();

    eprint!(
        "Rendering tile : [{}, {}] - [{}, {}] \n",
        tile.start_x,
        tile.start_y,
        tile.start_x + w,
        tile.start_y + h
    );

    for i in 0..h {
        let line = tile.start_y + i;
        io::stdout().flush().unwrap();

        // Cast a ray at each pixel in the image
        for j in 0..w {
            let col = tile.start_x + j;
            let mut pixel_color = Color::new(0.0, 0.0, 0.0);

            for _ in 0..image.samples {
                let u = (col as f64 + rand_unit()) / (image.width as f64 - 1.0);
                let v = (line as f64 + rand_unit()) / (image.height as f64 - 1.0);

                let ray = cam.get_ray(u, v);
                pixel_color += ray_color(&ray, &world, image.bounce_depth);
            }

            tile.buffer[i][j] = crate::color::process_color(&pixel_color, image.samples);
        }
    }

    tx.send(tile).unwrap();
}
