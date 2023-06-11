use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, WebSocket};
use wire_universe::{proto::FromClient, CellState};

use crate::util::console_log;

pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub struct World {
    pub x: i32,
    pub y: i32,
    pub tiles: Vec<Vec<CellState>>,
}

impl World {
    pub fn getCell(&self, x: i32, y: i32) -> Option<CellState> {
        let iy = y - self.y;
        let ix = x - self.x;
        if iy >= 0 && iy < self.tiles.len() as i32 {
            let iy = iy as usize;
            if ix >= 0 && ix < self.tiles[iy].len() as i32 {
                let ix = ix as usize;
                return Some(self.tiles[iy][ix]);
            }
        }
        None
    }
    pub fn setCell(&mut self, x: i32, y: i32, val: CellState) -> Option<()> {
        let iy = y - self.y;
        let ix = x - self.x;
        if iy >= 0 && iy < self.tiles.len() as i32 {
            let iy = iy as usize;
            if ix >= 0 && ix < self.tiles[iy].len() as i32 {
                let ix = ix as usize;
                self.tiles[iy][ix] = val;
                return Some(());
            }
        }
        None
    }
}

pub struct State {
    pub world: World,
    pub viewport: Viewport,
    pub brush: CellState,
    pub brush_pos: Option<(i32, i32)>,
    pub canvas: HtmlCanvasElement,
    pub brush_canvas: HtmlCanvasElement,
    pub canvases: HtmlElement,
    pub zoom: i32,
    pub socket: WebSocket,
    pub mousedown_state: Option<MousedownState>,
}

#[derive(Clone, Debug)]
pub enum MousedownState {
    Still { start_x: i32, start_y: i32 },
    Drag { prev_x: i32, prev_y: i32 },
}

#[derive(Clone, Copy, Debug)]
pub enum Command {
    /// x, y are tile coords
    TileHover {
        x: i32,
        y: i32,
    },
    TileHoverStop,
    /// x, y are tile coords
    TileClick {
        x: i32,
        y: i32,
    },
    /// x, y are pixel coords
    MouseDrag {
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
    },
    Zoom {
        amount: f32,
    },
}

impl State {
    fn set_brush(&self, s: CellState) -> Option<()> {
        let document = web_sys::window()?.document()?;
        let wire = document.get_element_by_id("paint-wire")?;
        let electron = document.get_element_by_id("paint-electron")?;
        let tail = document.get_element_by_id("paint-tail")?;
        let blank = document.get_element_by_id("paint-blank")?;
        wire.set_attribute("data-selected", "false").ok()?;
        electron.set_attribute("data-selected", "false").ok()?;
        tail.set_attribute("data-selected", "false").ok()?;
        blank.set_attribute("data-selected", "false").ok()?;
        match s {
            CellState::Alive => electron.set_attribute("data-selected", "true").ok()?,
            CellState::Dead => tail.set_attribute("data-selected", "true").ok()?,
            CellState::Empty => blank.set_attribute("data-selected", "true").ok()?,
            CellState::Wire => wire.set_attribute("data-selected", "true").ok()?,
        };
        self.draw_brush();
        Some(())
    }
    fn draw_brush(&self) {
        // todo!();
    }
    pub fn mouse_pos_to_pixel(&self, x: i32, y: i32) -> (i32, i32) {
        (
            x - self.canvases.offset_left() - self.brush_canvas.offset_left(),
            y - self.canvases.offset_top() - self.brush_canvas.offset_top(),
        )
    }
    pub fn pixel_to_tile(&self, x: i32, y: i32) -> (i32, i32) {
        (
            (x + self.viewport.x) / self.zoom,
            (y + self.viewport.y) / self.zoom,
        )
    }
    pub fn mouse_pos_to_tile(&self, x: i32, y: i32) -> (i32, i32) {
        let (x, y) = self.mouse_pos_to_pixel(x, y);
        self.pixel_to_tile(x, y)
    }
    pub fn process_command(&mut self, cmd: Command) -> Option<()> {
        console_log!("cmd get: {:?}", cmd);
        match cmd {
            Command::TileHover { x, y } => {
                self.brush_pos = Some((x, y));
                self.draw_brush();
            }
            Command::TileHoverStop => {
                self.brush_pos = None;
                self.draw_brush();
            }
            Command::TileClick { x, y } => {
                self.world.setCell(x, y, self.brush);
                paint_tile(&self.canvas, self.brush, x, y);
                let msg = FromClient::ModifyCell {
                    x,
                    y,
                    cell: self.brush,
                };
                let json = serde_json::to_string(&msg).unwrap();
                self.socket.send_with_str(&json).ok()?;
            }
            Command::MouseDrag {
                start_x,
                start_y,
                end_x,
                end_y,
            } => todo!(),
            Command::Zoom { amount } => todo!(),
        }
        Some(())
    }
}

fn paint_tile(canvas: &HtmlCanvasElement, tile: CellState, x: i32, y: i32) -> Option<()> {
    let ctx = canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;
    let color = match tile {
        CellState::Alive => "blue",
        CellState::Dead => "grey",
        CellState::Empty => "white",
        CellState::Wire => "orange",
    };
    ctx.set_fill_style(&JsValue::from_str(color));
    Some(())
}
