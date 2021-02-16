#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    e: [f64; 3],
}

impl Default for Vec3 {
    fn default() -> Self {
        Self { e: [0.0, 0.0, 0.0] }
    }
}

impl Vec3 {
    pub fn new(e0: f64, e1: f64, e2: f64) -> Self {
        Self { e: [e0, e1, e2] }
    }

    pub fn random<R>(rng: &mut R, min: f64, max: f64) -> Self
    where
        R: rand::Rng,
    {
        Self {
            e: [
                rng.gen_range(min..max),
                rng.gen_range(min..max),
                rng.gen_range(min..max),
            ],
        }
    }

    pub fn x(&self) -> f64 {
        self.e[0]
    }
    pub fn y(&self) -> f64 {
        self.e[1]
    }
    pub fn z(&self) -> f64 {
        self.e[2]
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }
    pub fn length_squared(&self) -> f64 {
        self.dot(&self)
    }

    pub fn dot(&self, rhs: &Self) -> f64 {
        self.e[0] * rhs.e[0] + self.e[1] * rhs.e[1] + self.e[2] * rhs.e[2]
    }
    pub fn cross(&self, rhs: &Self) -> Self {
        Self {
            e: [
                self.e[1] * rhs.e[2] - self.e[2] * rhs.e[1],
                self.e[2] * rhs.e[0] - self.e[0] * rhs.e[2],
                self.e[0] * rhs.e[1] - self.e[1] * rhs.e[0],
            ],
        }
    }

    pub fn unit_vector(self) -> Self {
        let l = self.length();
        self / l
    }

    pub fn near_zero(&self) -> bool {
        const S: f64 = 1e-8;
        self.e[0].abs() < S && self.e[1].abs() < S && self.e[2].abs() < S
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            e: [-self.e[0], -self.e[1], -self.e[2]],
        }
    }
}

impl std::ops::Index<usize> for Vec3 {
    type Output = f64;
    fn index(&self, index: usize) -> &Self::Output {
        &self.e[index]
    }
}

impl std::ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Self) -> Self {
        Self {
            e: [
                self.e[0] + rhs.e[0],
                self.e[1] + rhs.e[1],
                self.e[2] + rhs.e[2],
            ],
        }
    }
}

impl std::ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.e[0] += rhs.e[0];
        self.e[1] += rhs.e[1];
        self.e[2] += rhs.e[2];
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            e: [
                self.e[0] - rhs.e[0],
                self.e[1] - rhs.e[1],
                self.e[2] - rhs.e[2],
            ],
        }
    }
}

impl std::ops::Mul for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Self) -> Self {
        Self {
            e: [
                self.e[0] * rhs.e[0],
                self.e[1] * rhs.e[1],
                self.e[2] * rhs.e[2],
            ],
        }
    }
}
impl std::ops::Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f64) -> Self {
        Self {
            e: [self.e[0] * rhs, self.e[1] * rhs, self.e[2] * rhs],
        }
    }
}
impl std::ops::Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        rhs * self
    }
}

impl std::ops::MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.e[0] *= rhs;
        self.e[1] *= rhs;
        self.e[2] *= rhs;
    }
}

impl std::ops::Div<f64> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f64) -> Self {
        self * (1.0 / rhs)
    }
}

impl std::ops::DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, rhs: f64) {
        *self *= 1.0 / rhs;
    }
}

pub type Point3 = Vec3;
pub type Color = Vec3;

pub fn write_color(pixel_color: &Color, samples_per_pixel: i32) {
    let r = pixel_color.x();
    let g = pixel_color.y();
    let b = pixel_color.z();

    // Divide the color by the number of samples and ganma-corrected for ganma=2.0
    let scale = 1.0 / samples_per_pixel as f64;
    let r = (scale * r).sqrt();
    let g = (scale * g).sqrt();
    let b = (scale * b).sqrt();

    println!(
        "{} {} {}",
        (256.0 * num::clamp(r, 0.0, 0.999)) as i32,
        (256.0 * num::clamp(g, 0.0, 0.999)) as i32,
        (256.0 * num::clamp(b, 0.0, 0.999)) as i32
    );
}

#[derive(Debug)]
pub struct Ray {
    orig: Point3,
    dir: Vec3,
}

impl Ray {
    pub fn new(orig: Point3, dir: Vec3) -> Self {
        Self { orig, dir }
    }

    pub fn origin(&self) -> &Point3 {
        &self.orig
    }
    pub fn direction(&self) -> &Vec3 {
        &self.dir
    }

    pub fn at(&self, t: f64) -> Point3 {
        self.orig + t * self.dir
    }
}

pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f64,
    pub material: std::sync::Arc<dyn Material + Send + Sync>,
    pub front_face: bool,
}

pub trait Hittable {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

pub struct Sphere {
    center: Point3,
    radius: f64,
    material: std::sync::Arc<dyn Material + Send + Sync>,
}
impl Sphere {
    pub fn new(
        center: Point3,
        radius: f64,
        material: std::sync::Arc<dyn Material + Send + Sync>,
    ) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}
impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = *r.origin() - self.center;
        let a = r.direction().length_squared();
        let half_b = oc.dot(r.direction());
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }
        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let p = r.at(root);
        let outward_normal = (p - self.center) / self.radius;
        let front_face = r.direction().dot(&outward_normal) < 0.0;
        Some(HitRecord {
            p,
            normal: if front_face {
                outward_normal
            } else {
                -outward_normal
            },
            t: root,
            material: self.material.clone(),
            front_face,
        })
    }
}

pub struct HittableList {
    objects: Vec<std::sync::Arc<dyn Hittable + Send + Sync>>,
}
impl HittableList {
    pub fn add(&mut self, object: std::sync::Arc<dyn Hittable + Send + Sync>) {
        self.objects.push(object);
    }
}
impl Default for HittableList {
    fn default() -> Self {
        Self {
            objects: Default::default(),
        }
    }
}
impl Hittable for HittableList {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut ret = None;
        let mut closest_so_far = t_max;
        for object in self.objects.iter() {
            if let Some(rec) = object.hit(r, t_min, closest_so_far) {
                closest_so_far = rec.t;
                ret = Some(rec);
            }
        }
        ret
    }
}

pub struct Camera<R> {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    lens_radius: f64,
    rng: R,
}
impl<R> Camera<R> {
    pub fn new(
        lookfrom: Point3,
        lookat: Point3,
        vup: Vec3,
        vfov: f64, /* vertical field-of-view in degrees */
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
        rng: R,
    ) -> Self {
        let theta = degrees_to_radians(vfov);
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let w = (lookfrom - lookat).unit_vector();
        let u = vup.cross(&w).unit_vector();
        let v = w.cross(&u);

        let origin = lookfrom;
        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;
        let lens_radius = aperture / 2.0;
        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            lens_radius,
            rng,
        }
    }
}
impl<R> Camera<R>
where
    R: rand::Rng,
{
    pub fn get_ray(&mut self, s: f64, t: f64) -> Ray {
        let rd = self.lens_radius * random_in_unit_disk(&mut self.rng);
        let offset = self.u * rd.x() + self.v * rd.y();
        Ray::new(
            self.origin + offset,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
        )
    }
}

fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

fn random_in_unit_disk<R>(rng: &mut R) -> Vec3
where
    R: rand::Rng,
{
    loop {
        let p = Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0);
        if p.length_squared() < 1.0 {
            return p;
        }
    }
}

pub trait Material {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>;
}

pub struct Lambertian<R: 'static> {
    albedo: Color,
    tls_rng: &'static std::thread::LocalKey<std::cell::RefCell<R>>,
}
impl<R> Lambertian<R> {
    pub fn new(
        albedo: Color,
        tls_rng: &'static std::thread::LocalKey<std::cell::RefCell<R>>,
    ) -> Self {
        Self { albedo, tls_rng }
    }
}
impl<R> Material for Lambertian<R>
where
    R: rand::Rng,
{
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = rec.normal
            + self
                .tls_rng
                .with(|rng| random_in_unit_sphere(&mut *rng.borrow_mut()))
                .unit_vector();
        // Catch degenerated scatter direction
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }
        Some((self.albedo, Ray::new(rec.p, scatter_direction)))
    }
}
fn random_in_unit_sphere<R>(rng: &mut R) -> Vec3
where
    R: rand::Rng,
{
    loop {
        let p = Vec3::random(rng, -1.0, 1.0);
        if p.length_squared() < 1.0 {
            return p;
        }
    }
}

pub struct Metal<R: 'static> {
    albedo: Color,
    fuzz: f64,
    tls_rng: &'static std::thread::LocalKey<std::cell::RefCell<R>>,
}
impl<R> Metal<R> {
    pub fn new(
        albedo: Color,
        fuzz: f64,
        tls_rng: &'static std::thread::LocalKey<std::cell::RefCell<R>>,
    ) -> Self {
        Self {
            albedo,
            fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
            tls_rng,
        }
    }
}
impl<R> Material for Metal<R>
where
    R: rand::Rng,
{
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let reflected = reflect(&r_in.direction().unit_vector(), &rec.normal);
        let scattered = Ray::new(
            rec.p,
            reflected
                + self.fuzz
                    * self
                        .tls_rng
                        .with(|rng| random_in_unit_sphere(&mut *rng.borrow_mut())),
        );
        if scattered.direction().dot(&rec.normal) > 0.0 {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}

fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
    *v - 2.0 * v.dot(n) * *n
}

pub struct Dielectric<R: 'static> {
    index_of_refraction: f64,
    tls_rng: &'static std::thread::LocalKey<std::cell::RefCell<R>>,
}
impl<R> Dielectric<R> {
    pub fn new(
        index_of_refraction: f64,
        tls_rng: &'static std::thread::LocalKey<std::cell::RefCell<R>>,
    ) -> Self {
        Self {
            index_of_refraction,
            tls_rng,
        }
    }
}
impl<R> Material for Dielectric<R>
where
    R: rand::Rng,
{
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let attenuation = Color::new(1.0, 1.0, 1.0);
        let refraction_ratio = if rec.front_face {
            1.0 / self.index_of_refraction
        } else {
            self.index_of_refraction
        };

        let unit_direction = r_in.direction().unit_vector();
        let cos_theta = (-unit_direction).dot(&rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let direction = if cannot_refract
            || reflectance(cos_theta, refraction_ratio)
                > self
                    .tls_rng
                    .with(|rng| rng.borrow_mut().gen_range(0.0..1.0))
        {
            reflect(&unit_direction, &rec.normal)
        } else {
            refract(&unit_direction, &rec.normal, refraction_ratio)
        };

        let scattered = Ray::new(rec.p, direction);
        Some((attenuation, scattered))
    }
}

fn refract(uv: &Vec3, n: &Vec3, etai_over_etat: f64) -> Vec3 {
    let cos_theta = (-*uv).dot(n).min(1.0);
    let r_out_perp = etai_over_etat * (*uv + cos_theta * *n);
    let r_out_parallel = -((1.0 - r_out_perp.length_squared()).abs().sqrt()) * *n;
    r_out_perp + r_out_parallel
}

fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
    // Use Schlick's approximation for reflectance
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}
