#![allow(unused)]

use std::fmt::{self, Display, Formatter};

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
    pub fn new_with_state(living: bool) -> Tile {
        Tile {
            live: if living { true } else { false },
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

    pub fn from_bools((w, h): (usize, usize), tiles: Vec<bool>) -> Result<World, String> {

        // Quickly error out of the length of the tiles doesn't make sense.
        if tiles.len() != w * h {
            return Err(String::from("len doesn't match dims"));
        }

        Ok(World {
            cells: tiles.into_iter()
                .map(Tile::new_with_state)
                .collect(),
            dimensions: (w, h),
            tick: 0
        })

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

impl Display for World {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let world_box: String = vec!['='; self.dimensions.0].iter().collect();
        f.write_str(world_box.as_ref());
        f.write_str("\n");
        for l in 0..self.dimensions.1 { // for each line
            let mut s = String::new();
            for c in 0..self.dimensions.0 {
                match self.cell_at((c, l)) {
                    Some(c) => s.push(if c.live { '#' } else { ' ' }),
                    None => return Ok(()) // TODO Make this not suck.
                }
            }
            s.push('\n');
            f.write_str(s.as_str());
        }
        f.write_str(world_box.as_ref());
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_i2c_1() {
        assert_eq!(Some((5, 0)), index_to_cartesean((20, 1), 5))
    }

    #[test]
    fn test_c2i_1() {
        assert_eq!(Some(7), cartesean_to_index((3, 5), (1, 2)))
    }

    #[test]
    fn debug_world_deser() {

        // This is a glider.  Run with --nocapture.
        let tiles = vec![
            false, false, false, false, false,
            false, false,  true, false, false,
            false, false, false,  true, false,
            false,  true,  true,  true, false,
            false, false, false, false, false
        ];

        println!("{}", World::from_bools((5, 5), tiles).unwrap());

    }
}
