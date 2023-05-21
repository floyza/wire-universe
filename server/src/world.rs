use std::collections::HashMap;

use wire_universe::CellState;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum CellStateInternal {
    Alive,
    Dead,
    Wire,
}

fn cell_state_expel(c: Option<CellStateInternal>) -> CellState {
    match c {
        Some(CellStateInternal::Alive) => CellState::Alive,
        Some(CellStateInternal::Dead) => CellState::Dead,
        Some(CellStateInternal::Wire) => CellState::Wire,
        None => CellState::Empty,
    }
}

fn cell_state_admit(c: CellState) -> Option<CellStateInternal> {
    match c {
        CellState::Alive => Some(CellStateInternal::Alive),
        CellState::Dead => Some(CellStateInternal::Dead),
        CellState::Wire => Some(CellStateInternal::Wire),
        CellState::Empty => None,
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug)]
pub struct World {
    tiles: HashMap<Point, CellStateInternal>,
}

impl World {
    pub fn new() -> World {
        World {
            tiles: HashMap::new(),
        }
    }

    pub fn set_tile(&mut self, pos: Point, s: CellState) {
        match cell_state_admit(s) {
            Some(s) => self.tiles.insert(pos, s),
            None => self.tiles.remove(&pos),
        };
    }

    pub fn step(&mut self) {
        let copy = self.clone();
        for (pos, contents) in copy.tiles.iter() {
            let new = match *contents {
                CellStateInternal::Alive => CellStateInternal::Dead,
                CellStateInternal::Dead => CellStateInternal::Wire,
                CellStateInternal::Wire => {
                    let mut living_neighbors = 0;
                    for i in [
                        Point {
                            x: pos.x - 1,
                            y: pos.y - 1,
                        },
                        Point {
                            x: pos.x,
                            y: pos.y - 1,
                        },
                        Point {
                            x: pos.x + 1,
                            y: pos.y - 1,
                        },
                        Point {
                            x: pos.x - 1,
                            y: pos.y,
                        },
                        Point {
                            x: pos.x + 1,
                            y: pos.y,
                        },
                        Point {
                            x: pos.x - 1,
                            y: pos.y + 1,
                        },
                        Point {
                            x: pos.x,
                            y: pos.y + 1,
                        },
                        Point {
                            x: pos.x + 1,
                            y: pos.y + 1,
                        },
                    ] {
                        if let Some(CellStateInternal::Alive) = copy.tiles.get(&i) {
                            living_neighbors += 1;
                        }
                    }
                    if living_neighbors == 1 || living_neighbors == 2 {
                        CellStateInternal::Alive
                    } else {
                        CellStateInternal::Wire
                    }
                }
            };
            self.tiles.insert(*pos, new);
        }
    }

    pub fn copy_slice(&self, x: i32, y: i32, w: i32, h: i32) -> Vec<Vec<CellState>> {
        let mut ret = Vec::new();
        for j in y..(y + h) {
            let mut row = Vec::new();
            for i in x..(x + w) {
                let cell = self.tiles.get(&Point { x: i, y: j }).cloned();
                row.push(cell_state_expel(cell));
            }
            ret.push(row);
        }
        return ret;
    }
}

pub fn sample_world() -> World {
    let mut world = World::new();
    world
        .tiles
        .insert(Point { x: 1, y: 0 }, CellStateInternal::Alive);
    world
        .tiles
        .insert(Point { x: 0, y: 1 }, CellStateInternal::Dead);
    world
        .tiles
        .insert(Point { x: 1, y: 2 }, CellStateInternal::Wire);
    world
        .tiles
        .insert(Point { x: 2, y: 1 }, CellStateInternal::Wire);
    return world;
}
