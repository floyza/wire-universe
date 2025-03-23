use std::{collections::HashMap, path::Path};

use anyhow::{anyhow, Context, Result};
use wire_universe::CellState;

#[derive(Copy, Clone, Debug, PartialEq)]
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
    pts: HashMap<Point, usize>,
    pts_r: HashMap<usize, Point>,
    sts: Vec<CellStateInternal>,
    nbors: Vec<Vec<usize>>,
}

impl World {
    pub fn new() -> World {
        World {
            pts: HashMap::new(),
            pts_r: HashMap::new(),
            sts: Vec::new(),
            nbors: Vec::new(),
        }
    }

    pub fn from_wi(path: &Path) -> Result<World> {
        let data =
            std::fs::read(path).context(format!("Failed to read wi file {}", path.display()))?;
        let string = std::str::from_utf8(&data).context(format!(
            "File {} contained non utf-8 characters",
            path.display()
        ))?;
        let lines: Vec<_> = string.lines().collect();
        let result = (|| -> Result<World> {
            let (bounds, data) = lines
                .split_first()
                .ok_or_else(|| anyhow!("Too few lines"))?;
            let (w, h) = match bounds.split_ascii_whitespace().collect::<Vec<_>>() {
                v if v.len() == 2 => (v[0].parse::<usize>()?, v[1].parse::<usize>()?),
                _ => Err(anyhow!("First line format not formatted '<w> <h>'"))?,
            };
            let mut world = World::new();
            for y in 0..h {
                for x in 0..w {
                    let val = data
                        .get(y)
                        .and_then(|row| row.as_bytes().get(x))
                        .ok_or_else(|| anyhow!("Dimensions incorrect"))?;
                    let tile = match val {
                        b'#' => Some(CellState::Wire),
                        b'~' => Some(CellState::Dead),
                        b'@' => Some(CellState::Alive),
                        _ => None,
                    };
                    if let Some(tile) = tile {
                        world.set_tile(
                            Point {
                                x: x as i32,
                                y: y as i32,
                            },
                            tile,
                        );
                    }
                }
            }
            Ok(world)
        })();
        let world = result.context(format!("Failed to parse {}", path.display()))?;
        Ok(world)
    }

    pub fn set_tile(&mut self, pos: Point, s: CellState) {
        // TODO needs testing
        match cell_state_admit(s) {
            Some(s) => {
                if let Some(&i) = self.pts.get(&pos) {
                    self.sts[i] = s;
                } else {
                    let mut neighbors = Vec::new();
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
                        if let Some(&i) = self.pts.get(&i) {
                            neighbors.push(i);
                            self.nbors[i].push(self.sts.len());
                        }
                    }
                    self.sts.push(s);
                    self.nbors.push(neighbors);
                    self.pts.insert(pos, self.sts.len() - 1);
                    self.pts_r.insert(self.sts.len() - 1, pos);
                }
            }
            None => {
                if let Some(i) = self.pts.remove(&pos) {
                    assert!(
                        self.pts_r.remove(&i).is_some(),
                        "pts_r missing value in pts"
                    );
                    for ni in self.nbors[i].clone() {
                        self.nbors[ni].retain(|&x| x != i);
                    }
                    self.sts.swap_remove(i);
                    self.nbors.swap_remove(i);
                    // update neighbors for moved cell, which is now at `i`
                    let old_i = self.sts.len();
                    // we don't need to update anything if we removed the last cell
                    if i != old_i {
                        let moved_pt = self.pts_r[&old_i];
                        self.pts_r.remove(&old_i).unwrap();
                        self.pts_r.insert(i, moved_pt);
                        self.pts.insert(moved_pt, i); // modify existent value
                        for &ni in &self.nbors[i].clone() {
                            for nni in &mut self.nbors[ni] {
                                if *nni == old_i {
                                    *nni = i;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        };
    }

    pub fn step(&mut self) {
        let mut adj = vec![0; self.sts.len()];
        for (i, st) in self.sts.iter().enumerate() {
            if *st == CellStateInternal::Alive {
                for &n in &self.nbors[i] {
                    adj[n] += 1;
                }
            }
        }
        for (i, &n) in adj.iter().enumerate() {
            self.sts[i] = match self.sts[i] {
                CellStateInternal::Alive => CellStateInternal::Dead,
                CellStateInternal::Dead => CellStateInternal::Wire,
                CellStateInternal::Wire => {
                    if n == 1 || n == 2 {
                        CellStateInternal::Alive
                    } else {
                        CellStateInternal::Wire
                    }
                }
            }
        }
    }

    pub fn copy_slice(&self, x: i32, y: i32, w: i32, h: i32) -> Vec<Vec<CellState>> {
        let mut ret = Vec::new();
        for j in y..(y + h) {
            let mut row = Vec::new();
            for i in x..(x + w) {
                let cell = self.pts.get(&Point { x: i, y: j }).map(|&i| self.sts[i]);
                row.push(cell_state_expel(cell));
            }
            ret.push(row);
        }
        return ret;
    }
}

pub fn sample_world() -> World {
    let mut world = World::new();
    world.set_tile(Point { x: 1, y: 0 }, CellState::Alive);
    world.set_tile(Point { x: 0, y: 1 }, CellState::Dead);
    world.set_tile(Point { x: 1, y: 2 }, CellState::Wire);
    world.set_tile(Point { x: 2, y: 1 }, CellState::Wire);
    return world;
}
