use serde::{Deserialize, Serialize};

pub mod proto;

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
#[repr(u8)]
pub enum CellState {
    Alive,
    Dead,
    Empty,
    Wire,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
