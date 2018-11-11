use std::fs::File;

use lazy_static::lazy_static;

use sdl2::pixels::PixelFormatEnum;
use sdl2::render::BlendMode;
use ytesrev::prelude::*;

use crate::map::Map;

lazy_static! {
    static ref CAR_TEXTURE: PngImage =
        PngImage::load_from_path(File::open("car.png").unwrap()).unwrap();
}

pub struct Game<'a> {
    pub map: &'a Map,
    pub player_pos: (f64, f64),
    pub player_dir: f64, // In radians, 0 = right
    pub player_speed: f64,
}

impl<'a> Game<'a> {
    pub fn new(map: &'a Map) -> Game<'a> {
        Game {
            map,
            player_pos: (map.start.0 as f64, map.start.1 as f64),
            player_dir: 0.,
            player_speed: 0.,
        }
    }
}

impl Drawable for Game<'_> {
    fn update(&mut self, dt: f64) {
        self.player_pos.0 += self.player_speed * self.player_dir.cos() * dt;
        self.player_pos.1 += self.player_speed * self.player_dir.sin() * dt;
    }

    fn step(&mut self) {}

    fn content(&self) -> Vec<&Drawable> {
        vec![]
    }
    fn content_mut(&mut self) -> Vec<&mut Drawable> {
        vec![]
    }

    fn state(&self) -> State {
        State::Working
    }

    fn draw(&self, canvas: &mut Canvas<Window>, position: &Position, settings: DrawSettings) {
        let r = position.into_rect_with_size(self.map.width as u32, self.map.get_height() as u32);

        // Draw car
        let creator = canvas.texture_creator();
        let mut texture = creator
            .create_texture_target(
                Some(PixelFormatEnum::ABGR8888),
                CAR_TEXTURE.width as u32,
                CAR_TEXTURE.height as u32,
            )
            .expect("Can't make texture");

        texture.set_blend_mode(BlendMode::Blend);
        texture
            .update(None, CAR_TEXTURE.data.as_slice(), 4 * CAR_TEXTURE.width)
            .expect("Can't make texture");

        let at = Point::new(self.player_pos.0 as i32 + r.x(), self.player_pos.1 as i32 + r.y());
        canvas
            .copy_ex(
                &texture,
                None,
                Some(Rect::from_center(at, CAR_TEXTURE.width as u32, CAR_TEXTURE.height as u32)),
                self.player_dir,
                None,
                false,
                false,
            )
            .expect("Can't make texture");
    }
}
