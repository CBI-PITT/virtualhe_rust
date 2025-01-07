use clap::Parser;
use image::{ImageReader, DynamicImage,ImageBuffer};
use ndarray::{Array2, Array3};
use std::path::Path;

/// Command-line arguments for the utility.
#[derive(Parser)]
#[derive(Debug)]
struct Args {
    /// Path to the nucleus (hematoxylin) channel image (e.g., nucleus.tif).
    nucleus: String,
    /// Path to the eosin channel image (e.g., autof.tif).
    eosin: String,
    /// Path to save the output RGB image (e.g., output.tiff).
    output: String,
}

/// Apply histogram scaling to the image so that 1 pixel per 100,000 saturates at max intensity.
fn apply_histogram_scaling(image: &Array2<f32>, percentile: f32) -> Array2<f32> {
    let flattened: Vec<f32> = image.iter().copied().collect();
    let mut sorted = flattened.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let threshold_index = ((percentile / 100.0) * (sorted.len() as f32)) as usize;
    let max_intensity = sorted[threshold_index.min(sorted.len() - 1)];
    image.mapv(|v| (v / max_intensity).min(1.0))
}

/// Generate the virtual H&E RGB image using the given nucleus and eosin channels.
fn generate_virtual_he(
    nucleus: Array2<f32>,
    eosin: Array2<f32>,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parameters from the document
    let beta_values = [
        // Hematoxylin: (red, green, blue)
        [0.860, 1.000, 0.300],
        // Eosin: (red, green, blue)
        [0.050, 1.000, 0.544],
    ];
    let k = 2.5;

    // Compute RGB channels
    let mut rgb = Array3::<f32>::zeros((nucleus.nrows(), nucleus.ncols(), 3));
    //for (channel, beta) in beta_values.iter().enumerate() {
        for (i, j) in (0..nucleus.nrows()).flat_map(|i| (0..nucleus.ncols()).map(move |j| (i, j))) {
            rgb[[i, j, 0]] = (-beta_values[0][0] * nucleus[[i, j]] * k).exp() * (-beta_values[1][0] * eosin[[i, j]] * k).exp();
            rgb[[i, j, 1]] = (-beta_values[0][1] * nucleus[[i, j]] * k).exp() * (-beta_values[1][1] * eosin[[i, j]] * k).exp();
            rgb[[i, j, 2]] = (-beta_values[0][2] * nucleus[[i, j]] * k).exp() * (-beta_values[1][2] * eosin[[i, j]] * k).exp();
        }
    //}


// Normalize the RGB values to [0, 255] for uint16
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

    // Load nucleus and eosin images as grayscale float arrays
    let nucleus_image = ImageReader::open(&args.nucleus)?.decode()?;
    let eosin_image = ImageReader::open(&args.eosin)?.decode()?;

    let nucleus = match nucleus_image {
        DynamicImage::ImageLuma16(image) => {
            println!("Reading {}", &args.nucleus);
            Array2::<f32>::from_shape_vec(
                (image.height() as usize, image.width() as usize),
                image.pixels().map(|p| p[0] as f32 / 65535.0).collect(),
            )?
        }
        _ => panic!("Nucleus image must be grayscale!"),
    };

    let eosin = match eosin_image {
        DynamicImage::ImageLuma16(image) => {
            println!("Reading {}", &args.eosin);
            Array2::<f32>::from_shape_vec(
                (image.height() as usize, image.width() as usize),
                image.pixels().map(|p| p[0] as f32 / 65535.0).collect(),
            )?
        }
        _ => panic!("Eosin image must be grayscale!"),
    };

    // Apply histogram scaling
    println!{"Normalizing channels"}
    let nucleus_scaled = apply_histogram_scaling(&nucleus, 99.999);
    let eosin_scaled = apply_histogram_scaling(&eosin, 99.999);

    // Generate virtual H&E image
    println!{"Calculating and Saving vH&E"}
    generate_virtual_he(nucleus_scaled, eosin_scaled, Path::new(&args.output))?;
    println!("Virtual H&E image saved to: {}", args.output);

    Ok(())
}
