use crate::CellState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum FromServer {
    Refresh {
        x: i32,
        y: i32,
        /// indexed by y then x
        tiles: Vec<Vec<CellState>>,
    },
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum FromClient {
    ModifyCell { x: i32, y: i32, cell: CellState },
    SetView { x: i32, y: i32, w: i32, h: i32 },
    StartStream,
}
