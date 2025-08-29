use std::cmp::max;
use std::fs::File;
use std::io::{self, Write, stdout};
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};

use crate::primitives::vec3::{Point3, Vec3};
use crate::primitives::ray::Ray;
use crate::primitives::color::{Color, write_color};
use crate::primitives::interval::Interval;
use crate::hittable::{HitRecord, Hittable};
use crate::utils::{degrees_to_radians, random_double, random_double_range, INFINITY};
use crate::vec3::*;


pub struct Camera {
    image_width: i32,
    image_height: i32,
    center: Point3,

    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,

    samples_per_pixel: i32,
    pixel_samples_scale: f64,
    depth: i32,

    defocus_u: Vec3,
    defocus_v: Vec3,
    defocus_angle: f64,
}

impl Camera {
    pub fn new(aspect_ratio: f64, image_width: i32, vfov: f64, lookfrom: Point3, lookat: Point3, vup: Vec3, defocus_angle: f64, focus_dist: f64) -> Self {

        // Calculate the image height, and ensure it's at least 1
        let image_height = max((image_width as f64 / aspect_ratio) as i32, 1);

        //
        let center = lookfrom;

        // Variation angle of rays through each pixel
        let defocus_angle: f64 = defocus_angle;

        // Distance from camera lookfrom point to plane of perfect focus
        let focus_dist: f64 = focus_dist;

        // Viewport dimensions
        let theta: f64 = degrees_to_radians(vfov);
        let h:f64 = f64::tan(theta/2.0);
        let viewport_height: f64 = 2.0 * h * focus_dist;
        let viewport_width: f64 = viewport_height * aspect_ratio;

        // Calculate u, v and w (The unit basis vectors for the camera)
        let w: Vec3 = lookfrom - lookat;
        let w = w.unit_vector();

        let u: Vec3 = cross(&vup, &w);
        let u = unit_vector(&u);

        let v: Vec3 = cross(&w, &u);

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        // Vector across viewport horizontal edge
        let viewport_u: Vec3 = viewport_width * u;
        let viewport_v: Vec3 = viewport_height * -v;

        // The horizontal and vertical vectors ("distance") from pixel to pixel.
        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        // Calculate the location of the upper left pixel moving (focal_length) units towards the camera
        // then moving half of the viewport width to the left and half of the viewport height upwards.
        let viewport_upper_left: Point3 = center
            - focus_dist * w
            - viewport_u / 2.0
            - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        // Calculatre the Camera defocus disk basis vectors
        let defocus_radius: f64 = focus_dist * f64::tan(degrees_to_radians(defocus_angle / 2.0));
        let defocus_u = u * defocus_radius;
        let defocus_v = v * defocus_radius;

        // Samples per pixel used for antialiasing
        let samples_per_pixel: i32 = 30000;
        let pixel_samples_scale: f64 = 1.0 / samples_per_pixel as f64;

        // How many bounces is a given ray allowd to do
        let depth: i32 = 12;

        Self {
            image_width,
            image_height,
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            samples_per_pixel,
            pixel_samples_scale,
            depth,
            defocus_u,
            defocus_v,
            defocus_angle
        }
    }


    pub fn render(&self, world: &dyn Hittable, filename: &str) -> io::Result<()> {
        let mut file = File::create(filename)?;
        writeln!(file, "P3\n{} {}\n255", self.image_width, self.image_height)?;

        let bar = ProgressBar::new((self.image_height * self.image_width) as u64);
        bar.set_style(ProgressStyle::default_bar()
            .template("{msg} [{elapsed_precise}] [{wide_bar:.cyan}] {pos}/{len} ({eta})")
            .progress_chars("=> "));

        let pixels: Vec<Vec<(i32, Color)>> = (0..self.image_height)
            .into_par_iter()
            .map(|j| {
                let mut row_pixels = Vec::with_capacity(self.image_width as usize);
                for i in 0..self.image_width {
                    let mut pixel_color: Vec3 = Vec3::new(0.0, 0.0, 0.0);
                    for _sample in 0..self.samples_per_pixel {
                        let r: Ray = self.get_ray(i, j);
                        pixel_color = pixel_color + ray_color(&r, world, self.depth);
                    }
                    let final_color = self.pixel_samples_scale * pixel_color;
                    row_pixels.push((i, final_color));
                }
                bar.inc(self.image_width as u64);
                row_pixels
            })
            .collect();

        bar.finish_with_message("Rendering complete");

        for row in pixels {
            for (_i, color) in row {
                write_color(&mut file, &color)?;
            }
        }

        stdout().flush().unwrap();
        Ok(())
    }
    
    fn get_ray(&self, i: i32, j: i32) -> Ray {
        let offset: Vec3 = sample_square();
        //println!("{:?}", offset);
        let pixel_sample: Vec3 = self.pixel00_loc 
            + ((i as f64 + offset.x) * self.pixel_delta_u) 
            + ((j as f64 + offset.y) * self.pixel_delta_v);
        
        let ray_origin: Point3;
        if self.defocus_angle <= 0.0 {
            ray_origin = self.center;
        } else {
            ray_origin = self.defocus_disk_sample();
        }
        let ray_direction: Vec3 = pixel_sample - ray_origin;
    
        Ray::new(ray_origin, ray_direction)
    }

    // Returns a random point in the camera defocus disk.
    fn defocus_disk_sample(&self) -> Vec3 {
        let p: Point3 = Vec3::random_in_unit_disk();
        self.center + (p.x * self.defocus_u) + (p.y * self.defocus_v)
    }

}

fn sample_square() -> Vec3 {
    Vec3::new(random_double() - 0.5, random_double() - 0.5, 0.0)
}


pub fn ray_color(r: &Ray, world: &dyn Hittable, depth: i32) -> Color {
    if depth <= 0 {
        // If we have exceeded the ray bounce limit, no more light is scattered.
        return Color::new(0.0, 0.0, 0.0);
    }

    let hit_record: Option<HitRecord> = world.hit(r, &mut Interval::new(0.001, INFINITY));

    if hit_record.is_none() {
        return Color::new(0.0, 0.0, 0.0); // Background color
    }

    let hit_record: HitRecord = hit_record.unwrap();

    let random_behavior: f64 = random_double_range(0.0, 1.0);

    let mut scattered_ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0));
    let mut attenuation = Color::new(0.0, 0.0, 0.0);

    let material = &hit_record.mat;

    let scattered: Color = if random_behavior < material.kd {
        // Diffuse reflection
        if material.diffuse.scatter(r, &hit_record, &mut attenuation, &mut scattered_ray) {
            attenuation * ray_color(&scattered_ray, world, depth - 1)
        } else {
            Color::new(0.0, 0.0, 0.0)
        }
    } else if random_behavior < material.kd + material.ks {
        // Specular reflection
        if material.specular.scatter(r, &hit_record, &mut attenuation, &mut scattered_ray) {
            attenuation * ray_color(&scattered_ray, world, depth - 1)
        } else {
            Color::new(0.0, 0.0, 0.0)
        }
    } else if random_behavior < material.kd + material.ks + material.kt {
        // Refraction
        if material.refractive.scatter(r, &hit_record, &mut attenuation, &mut scattered_ray) {
            attenuation * ray_color(&scattered_ray, world, depth - 1)
        } else {
            Color::new(0.0, 0.0, 0.0)
        }
    } else {
        // Absorption or no scattering
        Color::new(0.0, 0.0, 0.0)
    };

    // Add emission if any
    if let Some(emit_color) = material.emit {
        scattered + emit_color
    } else {
        scattered
    }
}


