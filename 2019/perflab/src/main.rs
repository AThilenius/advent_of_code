// `rustup toolchain install nightly`
// `cargo +nightly run`

extern crate bmp;
extern crate tsc_timer;

use bmp::{Image, Pixel};
use tsc_timer::{Start, Stop};

use std::env;

pub struct Filter {
    divisor: i32,
    values: [[i32; 3]; 3],
}

#[allow(dead_code)]
impl Filter {
    fn gauss() -> Filter {
        Filter {
            divisor: 24,
            values: [[0, 4, 0], [4, 8, 4], [0, 4, 0]],
        }
    }
    fn hline() -> Filter {
        Filter {
            divisor: 1,
            values: [[-1, -2, -1], [0, 0, 0], [1, 2, 1]],
        }
    }
    fn vline() -> Filter {
        Filter {
            divisor: 1,
            values: [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]],
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let image_path = args.get(1).unwrap_or_else(|| {
        panic!("Please specify an input bmp file path.");
    });
    let input_img = bmp::open(image_path).unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });
    let filter = Filter::vline();
    let output_img = terrible_just_like_the_assignment(&input_img, &filter);
    output_img.save("output.bmp").unwrap();
    let _ = sane_way_to_do_it(&input_img, &filter);
    let _ = you_are_being_too_clever(&input_img, &filter);
}

fn terrible_just_like_the_assignment(raw_image: &Image, filter: &Filter) -> Image {
    // 8192x8192, 12byte per pixel image just like the assignment (805MB x 2)
    let mut input_img = vec![vec![vec![0_i32; 8192]; 8192]; 3];
    let mut output_img = vec![vec![vec![0_i32; 8192]; 8192]; 3];
    for (x, y) in raw_image.coordinates() {
        let Pixel { r, g, b } = raw_image.get_pixel(x, y);
        input_img[0][x as usize][y as usize] = r as i32;
        input_img[1][x as usize][y as usize] = g as i32;
        input_img[2][x as usize][y as usize] = b as i32;
    }
    // Begin timing, apply the filter in the same order as the assignment
    let start = Start::now();
    for col in 1..(raw_image.get_width() - 1) as usize {
        for row in 1..(raw_image.get_height() - 1) as usize {
            for plane in 0..3 {
                // Apply the filter at this row/col
                for j in 0..3 {
                    for i in 0..3 {
                        output_img[plane][row][col] +=
                            input_img[plane][row + i - 1][col + j - 1] * filter.values[i][j];
                    }
                }
                // Divide by the filter divisor
                output_img[plane][row][col] /= filter.divisor;
                // Clamp the values 0 to 255
                output_img[plane][row][col] = match output_img[plane][row][col] {
                    v if v < 0 => 0,
                    v if v > 255 => 255,
                    v => v,
                };
            }
        }
    }
    let diff = Stop::now() - start;
    let pixels = raw_image.get_width() as u64 * raw_image.get_height() as u64;
    println!("Terrible code Cycles Per Pixel: {}", diff.cycles() / pixels);
    // Convert back to a normal image
    let mut out_bmp = Image::new(raw_image.get_width(), raw_image.get_height());
    for (x, y) in out_bmp.coordinates() {
        out_bmp.set_pixel(
            x,
            y,
            Pixel::new(
                output_img[0][x as usize][y as usize] as u8,
                output_img[1][x as usize][y as usize] as u8,
                output_img[2][x as usize][y as usize] as u8,
            ),
        );
    }
    return out_bmp;
}

fn sane_way_to_do_it(raw_image: &Image, filter: &Filter) -> Image {
    let mut input_img = vec![vec![vec![0_u8; raw_image.get_height() as usize]; raw_image.get_width() as usize]; 3];
    let mut output_img = vec![vec![vec![0_u8; raw_image.get_height() as usize]; raw_image.get_width() as usize]; 3];
    for (x, y) in raw_image.coordinates() {
        let Pixel { r, g, b } = raw_image.get_pixel(x, y);
        input_img[0][x as usize][y as usize] = r;
        input_img[1][x as usize][y as usize] = g;
        input_img[2][x as usize][y as usize] = b;
    }
    // Begin timing, apply the filter in the correct order
    let start = Start::now();
    for plane in 0..3 {
        for row in 1..(raw_image.get_height() - 1) as usize {
            for col in 1..(raw_image.get_width() - 1) as usize {
                let mut value: i32 = 0;
                // Apply the filter at this row/col
                for j in 0..3 {
                    for i in 0..3 {
                        value += input_img[plane][row + i - 1][col + j - 1] as i32 * filter.values[i][j];
                    }
                }
                // Divide by the filter divisor
                value /= filter.divisor;
                // Clamp the values 0 to 255
                output_img[plane][row][col] = match value {
                    v if v < 0 => 0,
                    v if v > 255 => 255,
                    v => v as u8,
                };
            }
        }
    }
    let diff = Stop::now() - start;
    let pixels = raw_image.get_width() as u64 * raw_image.get_height() as u64;
    println!("Terrible code Cycles Per Pixel: {}", diff.cycles() / pixels);
    let mut out_bmp = Image::new(raw_image.get_width(), raw_image.get_height());
    return out_bmp;
}

fn you_are_being_too_clever(raw_image: &Image, filter: &Filter) -> Image {
    let mut input_img = vec![vec![vec![0_u8; raw_image.get_height() as usize]; raw_image.get_width() as usize]; 3];
    let mut output_img = vec![vec![vec![0_u8; raw_image.get_height() as usize]; raw_image.get_width() as usize]; 3];
    for (x, y) in raw_image.coordinates() {
        let Pixel { r, g, b } = raw_image.get_pixel(x, y);
        input_img[0][x as usize][y as usize] = r;
        input_img[1][x as usize][y as usize] = g;
        input_img[2][x as usize][y as usize] = b;
    }
    // Pre-compute a lookup table of all 256 values, for each of the 9 filter
    // values. These values include both the multiple and divide, but are
    // integral values bit-shifted 8 bits up.
    let mut lookup = vec![vec![vec![0_i32; 256]; 3]; 3];
    for j in 0..3 {
        for i in 0..3 {
            for pixel_value in 0..256 {
                let shifted_pixel_value = (pixel_value as i32) << 8;
                lookup[j][i][pixel_value] = shifted_pixel_value * filter.values[j][i] / filter.divisor;
            }
        }
    }
    // Begin timing, apply the filter in the correct order
    let start = Start::now();
    for plane in 0..3 {
        for row in 1..(raw_image.get_height() - 1) as usize {
            for col in 1..(raw_image.get_width() - 1) as usize {
                let mut value: i32 = 0;
                // Accumulate from the lookup
                for j in 0..3 {
                    for i in 0..3 {
                        let channel_value = input_img[plane][row + i - 1][col + j - 1];
                        value += lookup[i][j][channel_value as usize];
                    }
                }
                // Downshift by 8 (divide by 256)
                value = value >> 8;
                // Clamp the values 0 to 255
                output_img[plane][row][col] = match value {
                    v if v < 0 => 0,
                    v if v > 255 => 255,
                    v => v as u8,
                };
            }
        }
    }
    let diff = Stop::now() - start;
    let pixels = raw_image.get_width() as u64 * raw_image.get_height() as u64;
    println!("Terrible code Cycles Per Pixel: {}", diff.cycles() / pixels);
    let mut out_bmp = Image::new(raw_image.get_width(), raw_image.get_height());
    return out_bmp;
}
