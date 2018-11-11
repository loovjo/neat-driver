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

    let s = DrawableWrapper(GameScene {
        games: vec![Game { map: &map }],
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
        vec![]
    }
    fn content_mut(&mut self) -> Vec<&mut Drawable> {
        vec![]
    }
    fn step(&mut self) {}
    fn state(&self) -> State {
        State::Working
    }

    fn draw(&self, canvas: &mut Canvas<Window>, position: &Position, settings: DrawSettings) {
        self.im.draw(canvas, position, settings);
    }
}
