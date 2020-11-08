use rand::prelude::StdRng;
use rand::SeedableRng;
use spacegame::core::noise::perlin::{make_permutation, perlin2d, Perlin};
fn main() {
    let imgx = 2048;
    let imgy = 2048;

    let nb_blocks = [64, 64];
    let block_size = [32u32; 32];
    let image_size = [block_size[0] * nb_blocks[0], block_size[1] * nb_blocks[1]];

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(image_size[0], image_size[1]);

    let mut rnd = StdRng::from_entropy();
    let perlin = Perlin::new(&mut rnd);

    let mut max = 0.0f32;
    let mut min = 0.0f32;

    let values = {
        let w = nb_blocks[0];
        let h = nb_blocks[1];
        let mut values = vec![];
        for x in 0..w {
            for y in 0..h {
                let xf = x as f32 / w as f32;
                let yf = y as f32 / h as f32;

                values.push(perlin.octave_perlin(xf, yf, 2, 0.9));
            }
        }
        values
    };

    assert_eq!(nb_blocks[0] * nb_blocks[1], values.len() as u32);

    for x in 0..image_size[0] {
        for y in 0..image_size[1] {
            let mut pixel = imgbuf.get_pixel_mut(x, y);
            let block_x = x / nb_blocks[0];
            let block_y = y / nb_blocks[1];

            assert!(block_x < nb_blocks[0]);
            assert!(block_y < nb_blocks[1]);

            //let perlin = perlin.octave_perlin(xf, yf, 2, 0.9);
            let idx = (block_x + nb_blocks[0] * block_y) as usize;
            let pixel_value = values[idx];
            match pixel_value {
                0.0..=0.15 => *pixel = image::Rgb([255u8, 0, 0]),
                0.15..=0.35 => *pixel = image::Rgb([255u8, 133, 0]),
                0.35..=0.55 => *pixel = image::Rgb([0, 255u8, 0]),
                _ => *pixel = image::Rgb([0, 0, 255u8]),
            }
        }
    }

    // // Iterate over the coordinates and pixels of the image
    // for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
    //     let xf = x as f32 / 2048.0;
    //     let yf = y as f32 / 2048.0;
    //
    //     let block_x = x / nb_blocks[0];
    //     let block_y = x / nb_blocks[1];
    //
    //     assert!(block_x < nb_blocks[0]);
    //     assert!(block_y < nb_blocks[1]);
    //
    //     //let perlin = perlin.octave_perlin(xf, yf, 2, 0.9);
    //     let idx = (block_x + nb_blocks[0] * block_y) as usize;
    //     let pixel_value = values[idx];
    //
    //     match pixel_value {
    //         0.0..=0.15 => *pixel = image::Rgb([255u8, 0, 0]),
    //         0.15..=0.35 => *pixel = image::Rgb([255u8, 133, 0]),
    //         0.35..=0.55 => *pixel = image::Rgb([0, 255u8, 0]),
    //         _ => *pixel = image::Rgb([0, 0, 255u8]),
    //     }
    //
    //     // max = max.max(perlin);
    //     // min = min.min(perlin);
    //     // let pixel_value = (perlin * 255.0) as u8;
    //     //*pixel = image::Rgb([pixel_value, pixel_value, pixel_value]);
    // }

    // Save the image as “fractal.png”, the format is deduced from the path
    imgbuf.save("perlin.png").unwrap();

    println!("{} to {}", min, max);
}
