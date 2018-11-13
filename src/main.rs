#![feature(bind_by_move_pattern_guards)]

use std::fs::File;
use std::mem::replace;
use ytesrev::prelude::*;
use ytesrev::window::WSETTINGS_MAIN;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use bincode::{deserialize_from, serialize_into};

mod game;
mod map;
mod neat;
use crate::game::*;
use crate::map::*;
use crate::neat::*;

pub const POP_SIZE: usize = 100;
pub const NUM_INPUTS: usize = 6;
pub const SHOW: usize = 20;

pub const SAVE_PATH: &str = "save.bc";

fn main() {
    let img = PngImage::load_from_path(File::open("map.png").unwrap()).unwrap();

    let map = Map::create_from_image(&img);
    let map_im = map.clone().into_image();

    println!("{} x {}", map_im.width, map_im.height);

    let mut games = Vec::with_capacity(POP_SIZE);
    let mut g_id = 0;

    if let Ok(f) = File::open(SAVE_PATH) {
        println!("Reading save!");
        let (genomes, g_id_): (Vec<Genome>, usize) = deserialize_from(f).expect("Can't read");
        g_id = g_id_;

        for genome in genomes {
            games.push(Game {
                controller: Controller::NEAT(genome),
                ..Game::new_human(&map)
            });
        }
    } else {
        for i in 0..POP_SIZE {
            let (genome, new_g_id) = Genome::init(NUM_INPUTS, 2);
            games.push(Game {
                controller: Controller::NEAT(genome),
                ..Game::new_human(&map)
            });
            g_id = new_g_id;
        }
    }

    let s = DrawableWrapper(GameScene {
        games: games,
        g_id,
        map: &map,
        im: map_im,
        accel: 0.,
        rot_vel: 0.,
        old_species: vec![],
        time: 0.,
        speed_mult: 1.,
        showing: None,
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

    showing: Option<Vec<usize>>,

    time: f64,
    speed_mult: f64,
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
                self.speed_mult /= 2.;
                println!("{}x", self.speed_mult);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Right),
                ..
            } => {
                self.rot_vel = 2.;
                self.speed_mult *= 2.;
                println!("{}x", self.speed_mult);
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
        self.time += dt;

        for game in &mut self.games {
            game.update(dt * self.speed_mult);

            game.player_dir += self.rot_vel * dt * self.speed_mult;
            game.player_speed += self.accel * dt * self.speed_mult;
        }

        if self.time > 40. {
            self.evolve();
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

        // Find best

        for game in &self.games[0..SHOW] {
            game.draw(canvas, position, settings);
        }
    }
}

impl GameScene<'_> {
    fn evolve(&mut self) {
        self.time = 0.;

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

        let new_population = next_generation(species, fitnesses, &mut self.g_id, true);

        println!("Saving...");

        let mut file = File::create(SAVE_PATH).unwrap();
        serialize_into(file, &(new_population.clone(), self.g_id)).expect("Can't save");
        println!("Done");

        for genome in new_population {
            self.games.push(Game {
                controller: Controller::NEAT(genome),
                ..Game::new_human(self.map)
            });
        }
    }
}
