#![feature(bind_by_move_pattern_guards)]

use std::fs::File;
use ytesrev::prelude::*;
use ytesrev::window::WSETTINGS_MAIN;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod game;
mod map;
mod neat;
use crate::game::*;
use crate::map::*;

fn main() {
    neat::test();
}

#[allow(unused)]
fn main_() {
    let img = PngImage::load_from_path(File::open("map.png").unwrap()).unwrap();

    let map = Map::create_from_image(&img);
    let map_im = map.clone().into_image();

    println!("{} x {}", map_im.width, map_im.height);

    let game = Game::new(&map);
    let s = DrawableWrapper(GameScene {
        games: vec![game],
        im: map_im,
        accel: 0.,
        rot_vel: 0.,
    });

    let mut wmng = WindowManager::init_window(
        s,
        WindowManagerSettings {
            windows: vec![("Game".into(), WSETTINGS_MAIN)],
            ..default_settings("Pong")
        },
    );
    wmng.start();
}

struct GameScene<'a> {
    im: PngImage,
    games: Vec<Game<'a>>,

    accel: f64,
    rot_vel: f64,
}

impl<'a> Drawable for GameScene<'a> {
    fn content(&self) -> Vec<&Drawable> {
        self.games.iter().map(|x| x as &Drawable).collect()
    }
    fn content_mut(&mut self) -> Vec<&mut Drawable> {
        self.games.iter_mut().map(|x| x as &mut Drawable).collect()
    }
    fn step(&mut self) {}
    fn state(&self) -> State {
        State::Working
    }

    fn event(&mut self, event: Event) {
        match event {
            Event::KeyDown {
                keycode: Some(Keycode::Left),
                ..
            } => {
                self.rot_vel = -2.;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Right),
                ..
            } => {
                self.rot_vel = 2.;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Up),
                ..
            } => {
                self.accel = 20.;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Down),
                ..
            } => {
                self.accel = -20.;
            }
            Event::KeyUp { .. } => {
                self.accel = 0.;
                self.rot_vel = 0.;
            }
            _ => {}
        }
    }

    fn update(&mut self, dt: f64) {
        for game in &mut self.games {
            game.update(dt);

            game.player_dir += self.rot_vel * dt;
            game.player_speed += self.accel * dt;
        }
    }

    fn draw(&self, canvas: &mut Canvas<Window>, position: &Position, settings: DrawSettings) {
        self.im.draw(canvas, position, settings);
        for game in &self.games {
            game.draw(canvas, position, settings);
        }
    }
}
