use std::fs::File;
use ytesrev::prelude::*;
use ytesrev::window::WSETTINGS_MAIN;

mod game;
mod map;
use crate::game::*;
use crate::map::*;

fn main() {
    let img = PngImage::load_from_path(File::open("map.png").unwrap()).unwrap();

    let map = Map::create_from_image(&img);
    let map_im = map.clone().into_image();

    println!("{} x {}", map_im.width, map_im.height);

    let game = Game::new(&map);
    let s = DrawableWrapper(GameScene {
        games: vec![game],
        im: map_im,
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

    fn draw(&self, canvas: &mut Canvas<Window>, position: &Position, settings: DrawSettings) {
        self.im.draw(canvas, position, settings);
        for game in &self.games {
            game.draw(canvas, position, settings);
        }
    }
}
