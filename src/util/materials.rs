// MATERIALS - Implements various materials

#![allow(dead_code)]

use cgmath::*;
use std::f32::consts::PI;
use rand::Rng;

use super::tracing::*;

// Abstract material definition
pub trait Material {
    fn scatter(&self, hit: &RayHit, ray: &Ray) -> (Ray, Color, f32); // returns a new ray, its attenuation, and the probabiltiy it was chosen for a given material
    fn emission(&self) -> Color;
}


// LAMBERTIAN
#[derive(Clone, Copy)]
pub struct Lambertian {
    pub albedo: Vec3,
    pub emission: Vec3,
    // pub ray_sampler: ImportanceSampler,
}
impl Default for Lambertian {
    fn default() -> Lambertian {
        Lambertian { 
            albedo: vec3(1.0,1.0,1.0),
            emission: Vec3::zero(),
            // ray_sampler: sample_hemisphere as ImportanceSampler,
        }
    }

}
impl Material for Lambertian {
    fn scatter(&self, hit: &RayHit, _ray: &Ray) -> (Ray, Color, f32) {
        let (dir, pdf) = sample_hemisphere(hit);
        (
            Ray {
                origin: hit.hitpoint,
                direction: dir,
            },
            self.albedo / PI,
            pdf,
        )
    }
    fn emission(&self) -> Color {
        self.emission
    }
}

// METAL
pub struct Metal {
    pub albedo: Color,
    pub emission: Color,
    pub roughness: f32,
}
impl Material for Metal {
    fn scatter(&self, hit: &RayHit, ray: &Ray) -> (Ray, Color, f32) {
        (
            Ray {
                origin: hit.hitpoint,
                direction: reflect(&ray.direction, &hit.normal) + self.roughness*rand_sphere_vec(),
            },
            self.albedo,
            1.0
        )
    }
    fn emission(&self) -> Color {
        self.emission
    }
}

// DIELECTRIC
pub struct Dielectric {
    pub idx_of_refraction: f32,
    // pub color: Vec3, 
}
impl Material for Dielectric {
    fn scatter(&self, hit: &RayHit, ray: &Ray) -> (Ray, Color, f32) {
        // index of refraction ratio depends on whether we're entering or leaving the object
        let eta = if hit.frontface {1.0/self.idx_of_refraction} else {self.idx_of_refraction};
        let critical_angle = eta*f32::sqrt(1.0-f32::min(-ray.direction.dot(hit.normal), 1.0).powi(2)) > 1.0;
        let fresnel_factor = fresnel(&ray.direction, &hit.normal, self.idx_of_refraction);
        // if angle is less than critical, then refract with probability according to fresnel coefficient (proportion of reflected/transmitted light)
        let will_refract = !critical_angle && rand::thread_rng().gen_range(0.0..1.0) >= fresnel_factor;
        let new_dir = if will_refract {
            refract(&ray.direction, &hit.normal, eta)
        }
        else {
            reflect(&ray.direction, &hit.normal)
        };
        
        (
            Ray {
                origin: hit.hitpoint,
                direction: new_dir
            },
            //if hit.frontface && !will_refract { vec3(1.0,1.0,1.0) } else { clampvec(self.color*hit.distance, 0.0, 1.0) } ,
            vec3(1.0,1.0,1.0),
            1.0
        )
    }
    fn emission(&self) -> Color {
        Vec3::zero()
    }
}

// PHASE FUNCTIONS
pub struct Isotropic {
    // An isotropic phase function is one where light scatters in all directions with equal probability
    // (there's only one such function, so this one is just parameterized by an albedo)
    pub albedo: Color,
    pub emission: Color,
}
impl Material for Isotropic {
    fn scatter(&self, hit: &RayHit, _ray: &Ray) -> (Ray, Color, f32) {
        // by definition, the isotropic phase function is where light scatters in all directions with equal distribution
        (Ray {origin: hit.hitpoint, direction: rand_sphere_vec() }, self.albedo, 1.0)
    }
    fn emission(&self) -> Color {
        self.emission
    }
}


// SAMPLING FUNCTIONS
// uniformly samples a hemisphere given by normal n
pub fn sample_hemisphere(hit: &RayHit) -> (Vec3, f32) {
    // get random vector in hemisphere
    let mut dir = rand_sphere_vec();
    dir.y = dir.y.abs();
    // rotate relative to given normal
    let rotation = cgmath::Basis3::between_vectors(Vec3::unit_y(), hit.normal);
    (rotation.rotate_vector(dir), 1.0/(2.0*PI))
}

// based on http://three-eyed-games.com/2018/05/12/gpu-path-tracing-in-unity-part-2/
pub fn alpha_sample(hit: &RayHit) -> (Vec3, f32) {
    let alpha = 1.0;
    let mut rng = rand::thread_rng();
    // pick random point on sphere sitting on xz plane
    let cos_theta = f32::powf(rng.gen_range(0.0..1.0), 1.0/(alpha+1.0));
    let sin_theta = f32::sqrt(f32::max(0.0, 1.0 - cos_theta*cos_theta));
    let phi = 2.0*PI*rng.gen_range(0.0..1.0);
    let vec = vec3(f32::cos(phi)*sin_theta, f32::sin(phi)*sin_theta, cos_theta);
    
    // rotate relative to given normal
    let rotation = cgmath::Basis3::between_vectors(Vec3::unit_z(), hit.normal);
    (rotation.rotate_vector(vec), (alpha+1.0)*f32::powf(cos_theta, alpha) / (2.0*PI))
}

// based on raytracing in one weekend
pub fn rtow_sample(hit: &RayHit) -> (Vec3, f32) {
    let dir = rand_sphere_vec();
    (hit.hitpoint + hit.normal + dir, 1.0/(2.0*PI))
}
