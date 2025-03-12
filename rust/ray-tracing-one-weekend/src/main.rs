use rand::Rng as _;
use rand::SeedableRng as _;
use ray_tracing_one_weekend::*;
use rayon::prelude::*;
use std::sync::Arc;

fn ray_color<T>(r: &Ray, world: &T, depth: i32) -> Color
where
    T: Hittable,
{
    if depth <= 0 {
        return Color::new(0.0, 0.0, 0.0);
    }
    if let Some(rec) = world.hit(r, 0.001, f64::INFINITY) {
        if let Some((attenuation, scattered)) = rec.material.scatter(r, &rec) {
            return attenuation * ray_color(&scattered, world, depth - 1);
        } else {
            return Color::new(0.0, 0.0, 0.0);
        }
    }
    let unit_direction = r.direction().unit_vector();
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
}

std::thread_local! {
    static RNG: std::cell::RefCell<rand_xorshift::XorShiftRng> = std::cell::RefCell::new(rand_xorshift::XorShiftRng::from_seed(rand::rng().random()));
}

fn random_scene<R>(rng: &mut R) -> HittableList
where
    R: rand::Rng,
{
    let mut world = HittableList::default();

    let ground_material = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5), &RNG));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    )));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rng.random_range(0.0..1.0);
            let x = a as f64 + 0.9 * rng.random_range(0.0..1.0);
            let z = b as f64 + 0.9 * rng.random_range(0.0..1.0);
            let center = Point3::new(x, 0.2, z);

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    // diffuse
                    let albedo = Color::random(rng, 0.0, 1.0);
                    let sphere_material = Arc::new(Lambertian::new(albedo, &RNG));
                    world.add(Arc::new(Sphere::new(center, 0.2, sphere_material)));
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo = Color::random(rng, 0.5, 1.0);
                    let fuzz = rng.random_range(0.0..0.5);
                    let sphere_material = Arc::new(Metal::new(albedo, fuzz, &RNG));
                    world.add(Arc::new(Sphere::new(center, 0.2, sphere_material)));
                } else {
                    // glass
                    let sphere_material = Arc::new(Dielectric::new(1.5, &RNG));
                    world.add(Arc::new(Sphere::new(center, 0.2, sphere_material)));
                }
            }
        }
    }

    let material1 = Arc::new(Dielectric::new(1.5, &RNG));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        material1,
    )));

    let material2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1), &RNG));
    world.add(Arc::new(Sphere::new(
        Point3::new(-4.0, 1.0, 1.0),
        1.0,
        material2,
    )));

    let material3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0, &RNG));
    world.add(Arc::new(Sphere::new(
        Point3::new(4.0, 1.0, -1.0),
        1.0,
        material3,
    )));

    world
}

fn main() {
    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 1200;
    let image_height = (image_width as f64 / aspect_ratio) as i32;
    let samples_per_pixel = 100;
    let max_depth = 50;

    // World
    let mut rng = rand::rng();
    let world = std::sync::Arc::new(random_scene(&mut rng));

    // Camera
    let lookfrom = Point3::new(14.0, 6.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);
    let dist_to_focus = 12.0;
    let aperture = 0.1;

    // Render

    println!("P3");
    println!("{} {}", image_width, image_height);
    println!("255");

    let ji: Vec<_> = (0..image_height)
        .flat_map(|j| (0..image_width).map(move |i| (j, i)))
        .collect();
    let image = dashmap::DashMap::with_capacity((image_height * image_width) as usize);
    ji.into_par_iter().for_each_init(
        || {
            let mut rng = rand::rng();
            let cam = Camera::new(
                lookfrom,
                lookat,
                vup,
                20.0,
                aspect_ratio,
                aperture,
                dist_to_focus,
                rand_xorshift::XorShiftRng::from_seed(rng.random()),
            );
            (rng, cam)
        },
        |(rng, cam), (j, i)| {
            let mut pixel_color = Color::default();
            for _ in 0..samples_per_pixel {
                let u = (i as f64 + rng.random_range(0.0..1.0)) / (image_width - 1) as f64;
                let v = (j as f64 + rng.random_range(0.0..1.0)) / (image_height - 1) as f64;
                let r = cam.get_ray(u, v);
                pixel_color += ray_color(&r, world.as_ref(), max_depth);
            }
            image.insert((j, i), pixel_color);
        },
    );
    let image = image.into_read_only();
    for j in (0..image_height).rev() {
        for i in 0..image_width {
            write_color(image.get(&(j, i)).unwrap(), samples_per_pixel);
        }
    }
    eprintln!("Done.");
}
