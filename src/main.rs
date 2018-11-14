#![feature(bind_by_move_pattern_guards)]

use std::cell::Cell;
use std::fs::File;
use std::mem::replace;
use ytesrev::prelude::*;
use ytesrev::window::WSETTINGS_MAIN;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseUtil;

use bincode::{deserialize_from, serialize_into};

mod car_textures;
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

static mut MOUSE: Option<MouseUtil> = None;

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
                controller: Controller::NEAT(genome, 0),
                ..Game::new_human(&map)
            });
        }
    } else {
        for i in 0..POP_SIZE {
            let (genome, new_g_id) = Genome::init(NUM_INPUTS, 2);
            games.push(Game {
                controller: Controller::NEAT(genome, 0),
                ..Game::new_human(&map)
            });
            g_id = new_g_id;
        }
    }

    // games.clear();
    // games.push(Game::new_human(&map));

    let s = DrawableWrapper(GameScene {
        games: games,
        g_id,
        map: &map,
        im: map_im,
        speed_mult: 1,
        showing: None,
        place_mouse: Cell::new(false),
        last_fitness_improvment: 0.,
    });

    let mut wmng = WindowManager::init_window(
        s,
        WindowManagerSettings {
            windows: vec![("Game".into(), WSETTINGS_MAIN)],
            ..default_settings("Pong")
        },
    );

    unsafe {
        MOUSE = Some(wmng.context.mouse());
    }

    wmng.start();
}

struct GameScene<'a> {
    im: PngImage,
    games: Vec<Game<'a>>,
    map: &'a Map,

    g_id: usize,

    showing: Option<Vec<usize>>,

    speed_mult: u64,

    place_mouse: Cell<bool>,

    last_fitness_improvment: f64,
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
                if self.speed_mult != 0 {
                    self.speed_mult -= 1;
                }
                println!("{}x", self.speed_mult);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Right),
                ..
            } => {
                self.speed_mult += 1;
                println!("{}x", self.speed_mult);
            }
            Event::MouseMotion {
                mut xrel, mut yrel, ..
            } => {
                if xrel.abs() > yrel.abs() {
                    yrel = 0;
                } else {
                    xrel = 0;
                }

                self.place_mouse.set(true);

                for game in &mut self.games {
                    if !game.died {
                        if let Controller::Human = game.controller {
                            game.player_dir += xrel as f64 * 0.002;
                            game.player_speed -= yrel as f64 * 0.4;
                            if game.player_speed < 0. {
                                game.player_speed = 0.;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn update(&mut self, dt: f64) {
        if dt > 0.06 {
            return;
        }

        self.last_fitness_improvment += dt;

        for i in 0..self.speed_mult {
            for game in &mut self.games {
                game.update(dt);

                if game.improved {
                    self.last_fitness_improvment = 0.;
                    game.improved = false;
                }
                if let Controller::Human = game.controller {
                    self.last_fitness_improvment = 0.;
                }
            }
        }

        if self.last_fitness_improvment > 10. {
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

        for (i, game) in self.games.iter().enumerate() {
            match game.controller {
                Controller::NEAT(_, _) if i > SHOW => {
                    continue;
                }
                _ => {}
            }
            game.draw(canvas, position, settings);
        }

        if self.place_mouse.replace(false) {
            unsafe {
                let (w, h) = canvas.window().size();
                if let Some(mouse) = &MOUSE {
                    mouse.warp_mouse_in_window(canvas.window(), w as i32 / 2, h as i32 / 2);
                    mouse.show_cursor(false);
                    mouse.set_relative_mouse_mode(true);
                }
            }
        }
    }
}

impl GameScene<'_> {
    fn evolve(&mut self) {
        self.last_fitness_improvment = 0.;
        let has_human = self.games.iter().any(|x| {
            if let Controller::Human = x.controller {
                true
            } else {
                false
            }
        });


        let mut fitnesses = Vec::with_capacity(POP_SIZE);
        let mut species: Vec<Vec<(Genome, usize)>> = Vec::new();

        for (i, game) in self.games.drain(..).enumerate() {
            match game.controller {
                Controller::NEAT(genome, species_idx) => {
                    while species.len() <= species_idx {
                        species.push(Vec::new());
                    }
                    species[species_idx].push((genome, i));
                    fitnesses.push(game.best_score);
                }
                _ => {}
            }
        }

        let new_population = next_generation(species.clone(), fitnesses, &mut self.g_id, true);

        println!("Saving...");

        let mut file = File::create(SAVE_PATH).unwrap();
        serialize_into(file, &(new_population.clone(), self.g_id)).expect("Can't save");
        println!("Done");

        for (i, species) in class_species(new_population, species).into_iter().enumerate() {
            for (genome, _) in species {
                self.games.push(Game {
                    controller: Controller::NEAT(genome, i),
                    ..Game::new_human(self.map)
                });
            }
        }

        if has_human {
            self.games.push(Game::new_human(self.map));
        }
    }
}
