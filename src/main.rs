#![allow(unused)]

use std::{io::{self}, f64::consts::PI};
use std::sync::Arc;

mod camera;
mod hittable;
mod utils;
mod primitives;
mod materials;
mod bvh;
mod textures;
mod external;

use primitives::*;
use materials::*;
use utils::random_double;
use camera::Camera;
use hittable::*;
use bvh::*;
use textures::*;

fn main() {
    // WORLD
    let mut world = HittableList::new();

    // --- Base scattering functions ---
    let lambert_white = Lambertian::new(Color::new(1.0, 1.0, 1.0));
    let lambert_red   = Lambertian::new(Color::new(0.80, 0.05, 0.05));
    let lambert_green = Lambertian::new(Color::new(0.12, 0.45, 0.15));
    let lambert_blue  = Lambertian::new(Color::new(0.10, 0.20, 0.80));
    let lambert_yellow= Lambertian::new(Color::new(0.95, 0.85, 0.25));
    let lambert_cyan  = Lambertian::new(Color::new(0.20, 0.85, 0.85));

    let specular = Specular::new();
    let refrac15 = Refractive::new(1.5);

    // --- Material helpers ---
    // Pure matte
    let matte = |diff: Arc<dyn ScatteringFunction>| {
        Arc::new(Material::new(diff, specular.clone(), refrac15.clone(), None, 1.0, 0.0, 0.0, 0.0))
    };
    // Polished metal (high specular with a bit of diffuse)
    let metal = |tint: Color| {
        Arc::new(Material::new(
            Lambertian::new(tint).clone(),
            specular.clone(),
            refrac15.clone(),
            None,
            0.05, 0.90, 0.0, 0.05,
        ))
    };
    // Ideal mirror (all specular)
    let mirror = Arc::new(Material::new(
        lambert_white.clone(), specular.clone(), refrac15.clone(), None,
        0.0, 1.0, 0.0, 0.0
    ));
    // Glass
    let glass = Arc::new(Material::new(
        lambert_white.clone(), specular.clone(), refrac15.clone(), None,
        0.0, 0.05, 0.90, 0.05
    ));
    // Emissive light (area light)
    let light = Arc::new(Material::new(
        lambert_white.clone(), specular.clone(), refrac15.clone(),
        Some(Color::new(17.0, 17.0, 17.0)),
        1.0, 0.0, 0.0, 0.0
    ));

    // --- Cornell Box ---
    // Left wall
    world.add(Arc::new(Quad::new(
        Point3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        matte(lambert_green.clone()),
    )));
    // Right wall
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        matte(lambert_red.clone()),
    )));
    // Back wall
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        matte(lambert_blue.clone()),
    )));
    // Floor
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        matte(lambert_white.clone()),
    )));
    // Ceiling
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 555.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        matte(lambert_white.clone()),
    )));

    // Ceiling light
    world.add(Arc::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        light.clone(),
    )));
    // Side light
    world.add(Arc::new(Quad::new(
        Point3::new(40.0, 300.0, 550.0),
        Vec3::new(100.0, 0.0, 0.0),
        Vec3::new(0.0, 100.0, 0.0),
        Arc::new(Material::new(
            lambert_white.clone(), specular.clone(), refrac15.clone(),
            Some(Color::new(0.5, 0.5, 1.0)),
            1.0, 0.0, 0.0, 0.3
        )),
    )));

    // --- Main objects ---

    // --- Earth textured sphere in the center ---
    let earth_texture = Arc::new(ImageTexture::new("earthSurface.jpg"));
    let earth_material = Arc::new(Material::new(
        Lambertian::new_from_texture(earth_texture),
        specular.clone(),
        refrac15.clone(),
        None,
        1.0, 0.0, 0.0, 0.0
    ));
    world.add(Arc::new(Sphere::new(
        Point3::new(278.0, 50.0, 278.0), // Center of the scene
        50.0, // Smaller radius than main objects
        earth_material,
    )));

    // Tall translucent box
    let tall_box_mat = Arc::new(Material::new(
        Lambertian::new(Color::new(0.7, 0.2, 0.7)), // tint
        specular.clone(),
        refrac15.clone(),
        None,
        0.15, 0.15, 0.65, 0.05,
    ));
    let tall_box = Arc::new(Quad::new_box(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 330.0, 165.0),
        tall_box_mat,
    ));
    let tall_box = Arc::new(RotationY::new(tall_box, 18.0));
    let tall_box = Arc::new(Translation::new(tall_box, Vec3::new(260.0, 0.0, 295.0)));
    world.add(tall_box);

    // Short metallic box
    let short_box_mat = metal(Color::new(0.80, 0.80, 0.85));
    let short_box = Arc::new(Quad::new_box(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 165.0, 165.0),
        short_box_mat,
    ));
    let short_box = Arc::new(RotationY::new(short_box, -15.0));
    let short_box = Arc::new(Translation::new(short_box, Vec3::new(80.0, 0.0, 200.0)));
    world.add(short_box);

    // Glass sphere
    world.add(Arc::new(Sphere::new(
        Point3::new(420.0, 90.0, 140.0), 90.0,
        glass.clone(),
    )));
    // Mirror sphere
    world.add(Arc::new(Sphere::new(
        Point3::new(380.0, 60.0, 420.0), 60.0,
        mirror.clone(),
    )));

    // --- Field of small random spheres on the floor ---
    // Distribution in a central strip, avoiding collisions with main objects
    let mut rng_spheres = Vec::<Arc<dyn Hittable>>::new();
    for gx in 0..10 {
        for gz in 0..8 {
            let cx = 40.0 + gx as f64 * 45.0 + 15.0 * random_double();
            let cz = 60.0 + gz as f64 * 55.0 + 15.0 * random_double();
            let r  = 8.0 + 6.0 * random_double();
            let cy = r; // resting on the floor
            let center = Point3::new(cx, cy, cz);

            // Avoid areas close to the big spheres
            let avoid = [
                Point3::new(420.0, 90.0, 140.0), // glass
                Point3::new(380.0, 60.0, 420.0), // mirror
                Point3::new(160.0, 50.0, 420.0), // matte
            ];
            let mut skip = false;
            for a in avoid.iter() {
                if (center - *a).length() < 80.0 { skip = true; break; }
            }
            if skip { continue; }

            // Random material
            let choose = random_double();
            let mat: Arc<Material> = if choose < 0.6 {
                // Random color matte
                let col = Color::new(
                    0.2 + 0.8 * random_double(),
                    0.2 + 0.8 * random_double(),
                    0.2 + 0.8 * random_double(),
                );
                matte(Lambertian::new(col))
            } else if choose < 0.85 {
                // Tinted metal
                let col = Color::new(
                    0.6 + 0.4 * random_double(),
                    0.6 + 0.4 * random_double(),
                    0.6 + 0.4 * random_double(),
                );
                metal(col)
            } else {
                // Glass marbles
                glass.clone()
            };

            world.add(Arc::new(Sphere::new(center, r, mat)) as Arc<dyn Hittable + Send + Sync>);
        }
    }

    // --- Camera ---
    let aspect_ratio = 1.0;
    let image_width = 1920;
    let vfov = 40.0;
    let lookfrom = Point3::new(278.0, 278.0, -800.0);
    let lookat   = Point3::new(278.0, 278.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);
    let defocus_angle = 0.0;
    let focus_dist = (lookfrom - lookat).length();

    let cam = Camera::new(
        aspect_ratio,
        image_width,
        vfov,
        lookfrom,
        lookat,
        vup,
        defocus_angle,
        focus_dist,
    );

    // --- BVH and render ---
    let bvh_node: Arc<BVHNode> = Arc::new(BVHNode::new(world.objects.clone()));
    let mut new_world = HittableList::new();
    new_world.add(bvh_node);

    cam.render(&new_world, "output.ppm");
}