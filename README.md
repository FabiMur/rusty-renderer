# rusty-renderer

**rusty-renderer** is a ray tracing renderer written in Rust, inspired by the [Ray Tracing in One Weekend](https://raytracing.github.io/) book series by Peter Shirley.

## Features

- Physically-based rendering with support for diffuse, specular, and refractive materials
- BVH acceleration structure for efficient ray-object intersection
- Area lights and emissive materials
- Texturing support (image textures, checker, solid color)
- Cornell Box scene setup
- Camera with depth of field and antialiasing
- Multi-threaded rendering using Rayon
- Easily extensible architecture for adding new primitives and materials

## Scene Setup

The `main.rs` file is responsible for setting up the scene, including materials, camera, lights, and objects. All configuration and object placement is done here before rendering starts.

## Inspiration

This project is based on the concepts and code from the "Ray Tracing in One Weekend" book series, reimagined and extended in Rust for learning purposes.

## Getting Started

Clone the repository and run:

```sh
cargo run --release
```

The rendered image will be saved as `output.ppm`.

## Folder Structure

- `src/` - Source code organized by modules (camera, materials, primitives, textures, etc.)

## License

MIT