use crate::CellState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum FromServer {
    FullRefresh {
        x: i32,
        y: i32,
        /// indexed by y then x
        tiles: Vec<Vec<CellState>>,
    },
    PartialRefresh {
        /// the outside perimeter tiles, starting at the top left and going counter-clockwise
        tiles: Vec<CellState>,
    },
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum FromClient {
    ModifyCell { x: i32, y: i32, cell: CellState },
    SetView { x: i32, y: i32, w: i32, h: i32 },
    StartStream,
}
