use image::imageops::{grayscale, resize, FilterType};

use image::{GenericImageView, ImageBuffer};
use image::{Luma, Pixel};

const IMG_WIDTH: u32 = 16;
const IMG_HEIGHT: u32 = 9;
pub const IMG_SIZE: usize = (IMG_WIDTH * IMG_HEIGHT) as usize;

pub fn get_dhash<I: GenericImageView + 'static>(img: &I) -> [bool; IMG_SIZE] {
    let buffered_image = to_grey_signature_image(img);

    let mut bits = [false; IMG_SIZE];

    let mut cur_idx = 0;

    for y in 0..IMG_HEIGHT {
        for x in 0..IMG_WIDTH {
            let left_pixel = buffered_image.get_pixel(x, y);
            let right_pixel = buffered_image.get_pixel(x + 1, y);

            bits[cur_idx] = left_pixel[0] > right_pixel[0];

            cur_idx += 1;
        }
    }

    return bits;
}

pub fn to_grey_signature_image<I: GenericImageView + 'static>(
    img: &I,
) -> ImageBuffer<
    Luma<<<I as GenericImageView>::Pixel as Pixel>::Subpixel>,
    std::vec::Vec<<<I as GenericImageView>::Pixel as Pixel>::Subpixel>,
> {
    let grey_image = grayscale(img);

    let signature_image = resize(&grey_image, IMG_WIDTH + 1, IMG_HEIGHT, FilterType::Triangle);

    return signature_image;
}
