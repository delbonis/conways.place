#![allow(unused)]

#[derive(Copy, Clone, Debug)]
pub struct Tile {
    pub live: bool,
    pub last_update: u64,
    pub data: u32
}

impl Tile {
    pub fn new_empty() -> Tile {
        Tile {
            live: false,
            last_update: 0,
            data: 0
        }
    }
    pub fn new_empty_live() -> Tile {
        Tile {
            live: true,
            last_update: 0,
            data: 0
        }
    }
}

#[derive(Clone)]
pub struct World {
    cells: Vec<Tile>,
    dimensions: (usize, usize),
    tick: u64
}

impl World {

    pub fn new((w, h): (usize, usize)) -> World {
        World {
            cells: vec![Tile::new_empty(); w * h],
            dimensions: (w, h),
            tick: 0
        }
    }

    pub fn cell_at(&self, pos: (usize, usize)) -> Option<&Tile> {
        match cartesean_to_index(self.dimensions, pos) {
            Some(i) => Some(&self.cells[i]),
            None => None
        }
    }

    pub fn cell_at_mut(&mut self, pos: (usize, usize)) -> Option<&mut Tile> {
        match cartesean_to_index(self.dimensions, pos) {
            Some(i) => Some(&mut self.cells[i]),
            None => None
        }
    }

    fn neighbors_to(&self, (x, y): (usize, usize)) -> u8 {
        let mut n = 0;
        for i in 0..2 { // 0..2 because usize
            for j in 0..2 { // same
                if i == 1 && j == 1 { // if it's on (x, y)
                    match self.cell_at((x + i - 1, y + j - 1)) {
                        Some(t) => if t.live { n += 1 },
                        None => {} // nothing counts as 0
                    }
                }
            }
        }
        n
    }

    pub fn step(&self) -> World {

        let mut w = World {
            cells: vec![Tile::new_empty(); self.dimensions.0 * self.dimensions.1],
            dimensions: self.dimensions,
            tick: self.tick + 1
        };

        for i in 0..self.dimensions.0 {
            for j in 0..self.dimensions.1 {
                let at = self.cell_at((i, j)).unwrap();
                let state = match (self.neighbors_to((i, j)), at.live) {
                    (n, true) if n > 3 => false,
                    (2, true) => true,
                    (3, true) => true,
                    (n, true) if n < 2 => false,
                    (3, false) => true,
                    (_, _) => false // else, just kill yourself I guess?
                };
                let wi = cartesean_to_index(self.dimensions, (i, j)).unwrap();
                w.cells[wi].live = state;
                if state != at.live {
                    w.cells[wi].last_update = w.tick
                }
            }
        }

        w

    }

}

#[inline]
fn cartesean_to_index((w, h): (usize, usize), (x, y): (usize, usize)) -> Option<usize> {
    if x >= w || y >= h {
        None
    } else {
        Some(w * y + x)
    }
}

#[inline]
fn index_to_cartesean((w, h): (usize, usize), i: usize) -> Option<(usize, usize)> {
    if i >= w * h {
        None
    } else {
        let x = i % w;
        let y = i / w;
        Some((x, y))
    }
}
