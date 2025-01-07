use clap::Parser;
use image::{ImageReader, DynamicImage, ImageBuffer};
use ndarray::{Array2, Array3};
use ndarray::parallel::prelude::*;
use std::path::Path;

/// Command-line arguments for the utility.
#[derive(Parser, Debug)]
#[command(about = "Make a Virtual H&E Image from Fluorescent Microscopy Images")]
struct Args {
    /// Path to the nucleus (hematoxylin) channel image (e.g., nucleus.tif).
    nucleus: String,
    /// Path to the eosin channel image (e.g., autof.tif).
    eosin: String,
    /// Path to save the output RGB image (e.g., output.tiff).
    output: String,
    /// K arbitrary factor to adjust color profile of H&E.
    #[arg(short, default_value="2.5")]
    k: f32
}

/// Apply histogram scaling to the image so that 1 pixel per 100,000 saturates at max intensity.
fn apply_histogram_scaling(mut image: Array2<f32>, percentile: f32) -> Array2<f32> {
    let mut sorted: Vec<f32> = image.par_iter().copied().collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let threshold_index = ((percentile / 100.0) * (sorted.len() as f32)) as usize;
    let max_intensity = sorted[threshold_index.min(sorted.len() - 1)];
    image.par_mapv_inplace(|v| (v / max_intensity).min(1.0));
    image
}


/// Generate the virtual H&E RGB image using the given nucleus and eosin channels.
fn generate_virtual_he(
    nucleus: Array2<f32>,
    eosin: Array2<f32>,
    output_path: &Path,
    k: f32
) -> Result<(), Box<dyn std::error::Error>> {
    // Parameters from the document
    let beta_values = [
        // Hematoxylin: (red, green, blue)
        [0.860, 1.000, 0.300],
        // Eosin: (red, green, blue)
        [0.050, 1.000, 0.544],
    ];

    // Compute RGB channels in parallel
    let mut rgb = Array3::<f32>::zeros((nucleus.nrows(), nucleus.ncols(), 3));

    rgb.axis_iter_mut(ndarray::Axis(2)).into_par_iter().enumerate().for_each(|(channel, mut plane)| {
        for ((i, j), elem) in plane.indexed_iter_mut() {
            *elem = match channel {
                0 => (-beta_values[0][0] * nucleus[[i, j]] * k).exp() * (-beta_values[1][0] * eosin[[i, j]] * k).exp(),
                1 => (-beta_values[0][1] * nucleus[[i, j]] * k).exp() * (-beta_values[1][1] * eosin[[i, j]] * k).exp(),
                2 => (-beta_values[0][2] * nucleus[[i, j]] * k).exp() * (-beta_values[1][2] * eosin[[i, j]] * k).exp(),
                _ => unreachable!(),
            };
        }
    });

    // Normalize the RGB values to [0, 255] for uint8
    let rgb_uint8 = rgb.mapv(|v| (v * 255.0).min(255.0) as u8);

    // Save the RGB image
    let mut output_image = ImageBuffer::new(rgb.shape()[1] as u32, rgb.shape()[0] as u32);
    for (x, y, pixel) in output_image.enumerate_pixels_mut() {
        let r = rgb_uint8[[y as usize, x as usize, 0]];
        let g = rgb_uint8[[y as usize, x as usize, 1]];
        let b = rgb_uint8[[y as usize, x as usize, 2]];
        *pixel = image::Rgb([r, g, b]);
    }
    output_image.save(output_path)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let args = Args::parse();

    // Initialize image reader
    let mut nucleus_image = ImageReader::open(&args.nucleus)?;
    let mut eosin_image = ImageReader::open(&args.eosin)?;

    // Remove file size and memory limits to enable processing of large images
    nucleus_image.no_limits();
    eosin_image.no_limits();

    // Decode the image
    let nucleus_image = nucleus_image.decode()?;
    let eosin_image = eosin_image.decode()?;

    // Read images into ndarray
    let nucleus = match nucleus_image {
        DynamicImage::ImageLuma16(image) => Array2::<f32>::from_shape_vec(
            (image.height() as usize, image.width() as usize),
            image.pixels().map(|p| p[0] as f32 / 65535.0).collect(),
        )?,
        DynamicImage::ImageLuma8(image) => Array2::<f32>::from_shape_vec(
            (image.height() as usize, image.width() as usize),
            image.pixels().map(|p| p[0] as f32 / 255.0).collect(),
        )?,
        _ => panic!("Nucleus image must be grayscale!"),
    };

    let eosin = match eosin_image {
        DynamicImage::ImageLuma16(image) => Array2::<f32>::from_shape_vec(
            (image.height() as usize, image.width() as usize),
            image.pixels().map(|p| p[0] as f32 / 65535.0).collect(),
        )?,
        DynamicImage::ImageLuma8(image) => Array2::<f32>::from_shape_vec(
            (image.height() as usize, image.width() as usize),
            image.pixels().map(|p| p[0] as f32 / 255.0).collect(),
        )?,
        _ => panic!("Eosin image must be grayscale!"),
    };

    // Apply histogram scaling
    let nucleus_scaled = apply_histogram_scaling(nucleus, 99.999);
    let eosin_scaled = apply_histogram_scaling(eosin, 99.999);

    // Generate virtual H&E image
    generate_virtual_he(nucleus_scaled, eosin_scaled, Path::new(&args.output),args.k)?;
    println!("Virtual H&E image saved to: {}", args.output);

    Ok(())
}
