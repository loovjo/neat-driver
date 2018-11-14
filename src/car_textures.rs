use rand::{thread_rng, Rng};

use std::fs::File;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;

use palette::{Alpha, Hsva, Hue, LinSrgba, Pixel};

use ytesrev::prelude::*;

lazy_static! {
    pub static ref CAR_TEXTURE_AI: PngImage =
        PngImage::load_from_path(File::open("car.png").unwrap()).unwrap();
    pub static ref CAR_BROKEN_TEXTURE: PngImage =
        PngImage::load_from_path(File::open("car_broken.png").unwrap()).unwrap();
    pub static ref CAR_TEXTURE_PLAYER: PngImage = PngImage::load_from_path_transform(
        File::open("car2.png").unwrap(),
        |col| Color::RGBA(col.r, col.b, col.g, col.a)
    )
    .unwrap();
    pub static ref SPECIES_TEXTURES: Arc<Mutex<Vec<(PngImage, PngImage)>>> =
        Arc::new(Mutex::new(Vec::new()));
}

pub fn get_texture_from_species(species: usize) -> (PngImage, PngImage) {
    let mut rng = thread_rng();
    if let Ok(ref mut textures) = SPECIES_TEXTURES.lock() {
        while textures.len() <= species {
            println!("Making {}", species);
            let shift = (rng.gen_range(0., 1.), rng.gen_range(0.5, 1.));

            textures.push((
                shift_hue(&*CAR_TEXTURE_AI, shift.0, shift.1),
                shift_hue(&*CAR_BROKEN_TEXTURE, shift.0, shift.1),
            ));
        }

        textures[species].clone()
    } else {
        unreachable!("o no")
    }
}

pub fn shift_hue(im: &PngImage, hue_shift: f32, sat_shift: f32) -> PngImage {
    let orig_data = im
        .data
        .as_slice()
        .iter()
        .map(|x| *x as f32 / 256.)
        .collect::<Vec<_>>();

    let rgbs: &[LinSrgba] = LinSrgba::from_raw_slice(&orig_data);

    let converted: Vec<LinSrgba> = rgbs
        .iter()
        .map(|x| {
            let c = x.color.into();
            Alpha {
                color: c,
                alpha: x.alpha,
            }
        })
        .map(|mut x: Hsva| {
            x.color = x.color.shift_hue(hue_shift * 360.);
            x
        })
        .map(|x| x.into())
        .collect::<Vec<_>>();

    let data: Vec<f32> = LinSrgba::into_raw_slice(converted.as_slice()).to_vec();

    PngImage {
        width: im.width,
        height: im.height,
        data: data.into_iter().map(|x| (x * 256.) as u8).collect(),
    }
}
