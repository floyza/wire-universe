use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum CellState {
    Alive,
    Dead,
    Empty,
    Wire,
}

pub const WORLD_WIDTH: usize = 500;
pub const WORLD_HEIGHT: usize = 500;

#[derive(Clone)]
pub struct World {
    pub tiles: Box<[CellState; WORLD_WIDTH * WORLD_HEIGHT]>,
}

impl World {
    pub fn new() -> World {
        World {
            tiles: Box::new([CellState::Empty; WORLD_WIDTH * WORLD_HEIGHT]),
        }
    }

    pub fn idx(x: usize, y: usize) -> usize {
        y * WORLD_WIDTH + x
    }

    pub fn step(&mut self) {
        let copy = self.clone();
        for i in 0..copy.tiles.len() {
            self.tiles[i] = copy.next_state(i);
        }
    }

    fn next_state(&self, idx: usize) -> CellState {
        match self.tiles[idx] {
            CellState::Alive => CellState::Dead,
            CellState::Dead => CellState::Wire,
            CellState::Empty => CellState::Empty,
            CellState::Wire => {
                let mut living_neighbors = 0;
                for i in [
                    idx as isize - 1,
                    idx as isize + 1,
                    idx as isize - WORLD_WIDTH as isize,
                    idx as isize + WORLD_WIDTH as isize,
                    idx as isize - WORLD_WIDTH as isize - 1,
                    idx as isize - WORLD_WIDTH as isize + 1,
                    idx as isize + WORLD_WIDTH as isize - 1,
                    idx as isize + WORLD_WIDTH as isize + 1,
                ] {
                    if i >= 0 {
                        if let Some(CellState::Alive) = self.tiles.get(i as usize) {
                            living_neighbors += 1;
                        }
                    }
                }
                if living_neighbors == 1 || living_neighbors == 2 {
                    CellState::Alive
                } else {
                    CellState::Wire
                }
            }
        }
    }
    pub fn copy_slice(&self, x: usize, y: usize, w: usize, h: usize) -> Vec<Vec<CellState>> {
        let mut ret = Vec::new();
        for j in y..(y + h) {
            let mut row = Vec::new();
            for i in x..(x + w) {
                row.push(self.tiles[World::idx(i, j)]);
            }
            ret.push(row);
        }
        return ret;
    }
}

pub fn sample_world() -> World {
    let mut world = World::new();
    world.tiles[World::idx(1, 0)] = CellState::Alive;
    world.tiles[World::idx(0, 1)] = CellState::Dead;
    world.tiles[World::idx(1, 2)] = CellState::Wire;
    world.tiles[World::idx(2, 1)] = CellState::Wire;
    return world;
}
