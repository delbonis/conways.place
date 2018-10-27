#![allow(unused)]

use std::fmt::{self, Display, Formatter};
use std::iter::*;
use std::ops::Range;
use std::time;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub live: bool,
    pub last_update: u64,
    pub data: u32
}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.live == other.live && self.data == other.data
    }
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
            live: living,
            last_update: 0,
            data: 0
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    cells: Vec<Tile>,
    dimensions: (usize, usize),
    tick: u64
}

const NEIGHBOR_OFFSETS: [(isize, isize); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1)
];

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

    #[inline]
    fn neighbors_to(&self, (x, y): (usize, usize)) -> Vec<&Tile> {
        NEIGHBOR_OFFSETS.into_iter()
            .map(|(dx, dy)| self.cell_at(((x as isize + dx) as usize, (y as isize + dy) as usize)))
            .filter(Option::is_some)
            .map(Option::unwrap)
            .collect()
    }

    pub fn set_tile_liveness(&mut self, pos: (usize, usize), live: bool) {
        let i = cartesean_to_index(self.dimensions, pos);
        if i.is_some() {
            self.cells[i.unwrap()].live = live;
        }
    }

    pub fn step(&self) -> World {

        // First compute the state of each of the tiles.
        let next_tiles: Vec<Tile> = (0..self.cells.len()).into_iter()
            // The unwrap is fine because we never pass in an OOB thing.  Same for the index.
            .map(|i| (index_to_cartesean(self.dimensions, i).unwrap(), &self.cells[i]))
            .map(|(p, s)| compute_tile_next_step(
                self.neighbors_to(p).into_iter()
                    .filter(|t| t.live)
                    .collect(),
                s,
                self.tick + 1))
            .collect();

        // Then actually assemble the new world.
        World {
            cells: next_tiles,
            dimensions: self.dimensions,
            tick: self.tick + 1
        }

    }

    pub fn cells(&self) -> &Vec<Tile> {
        &self.cells
    }

    pub fn dims(&self) -> (usize, usize) {
        self.dimensions
    }

}

#[inline]
fn compute_tile_next_step(adjacents: Vec<&Tile>, subject: &Tile, tick_for: u64) -> Tile {
    let will_live = adjacents.len() == 3 || (subject.live && adjacents.len() == 2);
    Tile {
        live: will_live,
        last_update: if will_live != subject.live { tick_for } else { subject.last_update },
        data: if will_live {
            if subject.live {
                subject.data // if it's already alive, just use the same color
            } else {
                adjacents[tick_for as usize % adjacents.len()].data // if it's becomming alive, then pick a new one
            }
        } else { subject.data }
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
pub fn index_to_cartesean((w, h): (usize, usize), i: usize) -> Option<(usize, usize)> {
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

    extern crate test;

    use self::test::Bencher;
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

    #[bench]
    fn bench_step(b: &mut Bencher) {

        let pat = [
            false,
            true,
            false,
            true,
            true,
            false,
            true,
            false,
            true,
            false,
            true,
            true,
            false
        ];

        let wb = (0..1000000).into_iter()
            .map(|i| pat[i % pat.len()])
            .collect();

        let mut w = World::from_bools((1000, 1000), wb).unwrap();
        b.iter(|| { w = w.step() });

    }

}
