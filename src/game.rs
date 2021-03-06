use std::fs::File;

use std::f64::consts::PI;

use sdl2::pixels::PixelFormatEnum;
use sdl2::render::BlendMode;
use ytesrev::prelude::*;
use ytesrev::utils::line_aa;

use crate::car_textures::*;
use crate::map::{Map, Tile};
use crate::neat::Genome;
use crate::NUM_INPUTS;

pub struct Game<'a> {
    pub map: &'a Map,
    pub player_pos: (f64, f64),
    pub player_dir: f64, // In radians, 0 = right
    pub player_speed: f64,
    pub died: bool,
    pub best_score: f64,

    pub controller: Controller,
    pub time: f64,

    pub improved: bool,
}

pub enum Controller {
    NEAT(Genome, usize),
    Human,
}

impl<'a> Game<'a> {
    pub fn new_human(map: &'a Map) -> Game<'a> {
        Game {
            map,
            player_pos: (map.start.0 as f64, map.start.1 as f64),
            player_dir: 0.,
            player_speed: 0.,
            died: false,
            best_score: 0.,
            time: 0.,
            controller: Controller::Human,
            improved: false,
        }
    }

    pub fn cast_ray(&self, from: (f64, f64), angle: f64) -> (f64, f64) {
        let mut at = from;
        loop {
            match self
                .map
                .data
                .get(at.0 as usize + at.1 as usize * self.map.width)
            {
                Some(Tile::Wall) | None => {
                    return at;
                }
                _ => {}
            }

            at.0 += angle.cos();
            at.1 += angle.sin();
        }
    }
}

impl Drawable for Game<'_> {
    fn update(&mut self, dt: f64) {
        if self.died {
            return;
        }
        self.time += dt;
        self.player_pos.0 += self.player_speed * self.player_dir.cos() * dt;
        self.player_pos.1 += self.player_speed * self.player_dir.sin() * dt;

        if self.player_pos.0 < 0.
            || self.player_pos.1 < 0.
            || self.player_pos.0 > self.map.width as f64
            || self.player_pos.1 > self.map.get_height() as f64
        {
            self.died = true;
        }

        match self
            .map
            .data
            .get(self.player_pos.0 as usize + self.player_pos.1 as usize * self.map.width)
        {
            Some(Tile::Wall) | None => {
                self.died = true;
            }
            Some(Tile::Ground(x)) => {
                let score = *x as f64 / (self.time + 10.);
                if score > self.best_score {
                    self.best_score = score;
                    self.improved = true;
                }
            }
            _ => {}
        }

        if let Controller::NEAT(genome, _) = &self.controller {
            let mut inputs: [f64; NUM_INPUTS] = [0.; NUM_INPUTS];

            for i in 1..NUM_INPUTS {
                let d_angle = (i as f64 / (NUM_INPUTS - 1) as f64 - 0.5) * PI;
                let angle = self.player_dir + d_angle;
                let ray = self.cast_ray(self.player_pos, angle);

                let dx = ray.0 - self.player_pos.0;
                let dy = ray.1 - self.player_pos.1;
                let dist = (dx * dx + dy * dy).sqrt();
                inputs[i] = dist;
            }
            inputs[0] = self.player_speed;

            let res = genome.evaluate(&inputs);
            self.player_speed += (res[0].atanh().max(-40.).min(40.)) * dt;
            self.player_dir += res[1] * dt * 10.;
        }
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

        match self.controller {
            Controller::NEAT(_, s) => {
                load_species(s);

                if let Ok(ref mut textures) = SPECIES_TEXTURES.lock() {
                    let (car, crash) = &textures[s];
                    if self.died {
                        self.draw_texture(canvas, position, &crash);
                    } else {
                        self.draw_texture(canvas, position, &car);
                    }
                }
            }
            Controller::Human => self.draw_texture(canvas, position, &*CAR_TEXTURE_PLAYER),
        }

        for i in 0..NUM_INPUTS - 1 {
            let d_angle = (i as f64 / (NUM_INPUTS - 2) as f64 - 0.5) * PI;
            let angle = self.player_dir + d_angle;
            let ray = self.cast_ray(self.player_pos, angle);

            line_aa(
                canvas,
                (
                    self.player_pos.0 + r.x() as f64,
                    self.player_pos.1 + r.y() as f64,
                ),
                (ray.0 + r.x() as f64, ray.1 + r.y() as f64),
            );
        }
    }
}

impl Game<'_> {
    fn draw_texture(
        &self,
        canvas: &mut Canvas<Window>,
        position: &Position,
        car_texture: &PngImage,
    ) {
        let r = position.into_rect_with_size(self.map.width as u32, self.map.get_height() as u32);

        // Draw car
        let creator = canvas.texture_creator();
        let mut texture = creator
            .create_texture_target(
                Some(PixelFormatEnum::ABGR8888),
                car_texture.width as u32,
                car_texture.height as u32,
            )
            .expect("Can't make texture");

        texture.set_blend_mode(BlendMode::Blend);
        texture.update(None, car_texture.data.as_slice(), 4 * car_texture.width);

        let at = Point::new(
            self.player_pos.0 as i32 + r.x(),
            self.player_pos.1 as i32 + r.y(),
        );
        canvas
            .copy_ex(
                &texture,
                None,
                Some(Rect::from_center(
                    at,
                    car_texture.width as u32,
                    car_texture.height as u32,
                )),
                self.player_dir / PI * 180.,
                None,
                false,
                false,
            )
            .expect("Can't make texture");
    }
}
