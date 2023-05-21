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
