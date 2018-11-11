use std::u64;
use ytesrev::image::PngImage;

#[derive(Clone)]
pub struct Map {
    pub data: Vec<Tile>,
    pub width: usize,
    pub start: (usize, usize),
    pub end: (usize, usize),
}

impl Map {
    pub fn create_from_image(image: &PngImage) -> Map {
        let mut data = vec![Tile::Wall; image.width * image.height];

        let mut start: Option<(usize, usize)> = None;

        for x in 0..image.width {
            for y in 0..image.height {
                let idx = x + y * image.width;
                let b = image.data[4 * idx];
                let g = image.data[4 * idx + 1];
                let r = image.data[4 * idx + 2];

                let b = match (r, g, b) {
                    (255, 255, 255) => Tile::Ground(u64::MAX),
                    (255, 0, 0) => {
                        start = Some((x, y));
                        println!("Start at {:?}", start);

                        Tile::Ground(u64::MAX)
                    }
                    _ => Tile::Wall,
                };

                data[idx] = b;
            }
        }

        let start = start.expect("No starting point!");

        // Fill ground distances
        let mut dist: u64 = 0;
        let mut visiting = vec![start];
        let mut end = start;

        loop {
            let mut next = Vec::new();

            for pos in visiting.drain(..) {
                if pos.0 <= 1 || pos.1 <= 1 {
                    continue;
                }
                for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                    let x = pos.0 + *dx as usize;
                    let y = pos.1 + *dy as usize;
                    if let Some(Tile::Ground(ref mut d)) = data.get_mut(x + y * image.width) {
                        if *d > dist {
                            *d = dist;
                            next.push((x, y));
                            end = (x, y);
                        }
                    }
                }
            }
            if next.len() == 0 {
                break;
            }

            dist += 1;
            visiting = next;
        }

        Map {
            data,
            width: image.width,
            start,
            end,
        }
    }

    pub fn into_image(self) -> PngImage {
        let mut im_data = vec![255; 4 * self.data.len()];
        let mut max = 256;
        if let Some(Tile::Ground(n)) = self.data.get(self.end.0 + self.end.1 * self.width) {
            max = *n;
        }

        for (i, x) in self.data.into_iter().enumerate() {
            let col = match x {
                Tile::Wall => (0, 0, 0),
                Tile::Ground(n) => {
                    let g;
                    if n < 5 {
                        g = 255;
                    } else {
                        g = 0;
                    }
                    ((n as f64 / max as f64 * 255.0) as u8, g, 255)
                }
            };

            im_data[4 * i] = col.0;
            im_data[4 * i + 1] = col.1;
            im_data[4 * i + 2] = col.2;
            im_data[4 * i + 3] = 255;
        }

        let height = im_data.len() / self.width / 4;

        PngImage {
            data: im_data,
            width: self.width,
            height,
        }
    }

    pub fn get_height(&self) -> usize {
        self.data.len() / self.width
    }
}

#[derive(Clone, Copy)]
pub enum Tile {
    Ground(u64),
    Wall,
}
