#![feature(bind_by_move_pattern_guards)]

use std::mem::replace;
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
use crate::neat::*;

pub const POP_SIZE: usize = 20;
pub const NUM_INPUTS: usize = 5;

fn main() {
    let img = PngImage::load_from_path(File::open("map.png").unwrap()).unwrap();

    let map = Map::create_from_image(&img);
    let map_im = map.clone().into_image();

    println!("{} x {}", map_im.width, map_im.height);

    let mut games = Vec::with_capacity(POP_SIZE);
    let mut g_id = 0;

    for i in 0..POP_SIZE {
        let (genome, new_g_id) = Genome::init(NUM_INPUTS, 2);
        games.push(Game {
            controller: Controller::NEAT(genome),
            ..Game::new_human(&map)
        });
        g_id = new_g_id;
    }

    let s = DrawableWrapper(GameScene {
        games: games,
        g_id,
        map: &map,
        im: map_im,
        accel: 0.,
        rot_vel: 0.,
        old_species: vec![],
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
    map: &'a Map,

    g_id: usize,

    accel: f64,
    rot_vel: f64,

    old_species: Vec<Vec<(Genome, usize)>>,
}

impl<'a> Drawable for GameScene<'a> {
    fn content(&self) -> Vec<&Drawable> {
        self.games.iter().map(|x| x as &Drawable).collect()
    }
    fn content_mut(&mut self) -> Vec<&mut Drawable> {
        self.games.iter_mut().map(|x| x as &mut Drawable).collect()
    }
    fn step(&mut self) {
        println!("Evolving");
        self.evolve();
    }

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

        for game in &self.games {
            if !game.died {
                return;
            }
        }

        self.evolve();
    }

    fn draw(&self, canvas: &mut Canvas<Window>, position: &Position, settings: DrawSettings) {
        self.im.draw(canvas, position, settings);
        for game in &self.games {
            game.draw(canvas, position, settings);
        }
    }
}

impl GameScene<'_> {
    fn evolve(&mut self) {
        let mut population = Vec::with_capacity(POP_SIZE);
        let mut fitnesses = Vec::with_capacity(POP_SIZE);

        for game in self.games.drain(..) {
            match game.controller {
                Controller::NEAT(genome) => {
                    population.push(genome);
                    fitnesses.push(game.best_score as f64);
                }
                _ => {}
            }
        }

        let old_species = replace(&mut self.old_species, Vec::new());

        let species = class_species(population, old_species);
        self.old_species = species.clone();

        let new_population = next_generation(species, fitnesses, &mut self.g_id,true);

        for genome in new_population {
            self.games.push(Game {
                controller: Controller::NEAT(genome),
                ..Game::new_human(self.map)
            });
        }
    }
}
