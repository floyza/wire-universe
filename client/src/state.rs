use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, WebSocket};
use wire_universe::{proto::FromClient, CellState};

use crate::util::{console_log, document};

pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

const TILE_BUFFER: i32 = 5;

pub struct World {
    pub x: i32,
    pub y: i32,
    pub tiles: Vec<Vec<CellState>>,
}

impl World {
    pub fn get_cell(&self, x: i32, y: i32) -> Option<CellState> {
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
    pub fn set_cell(&mut self, x: i32, y: i32, val: CellState) -> Option<()> {
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
    pub fn set_brush(&self, s: CellState) -> Result<(), JsValue> {
        let document = document()?;
        let wire = document
            .get_element_by_id("paint-wire")
            .ok_or(JsValue::from_str("#paint-wire missing"))?;
        let electron = document
            .get_element_by_id("paint-electron")
            .ok_or(JsValue::from_str("#paint-electron missing"))?;
        let tail = document
            .get_element_by_id("paint-tail")
            .ok_or(JsValue::from_str("#paint-tail missing"))?;
        let blank = document
            .get_element_by_id("paint-blank")
            .ok_or(JsValue::from_str("#paint-blank missing"))?;
        wire.set_attribute("data-selected", "false")?;
        electron.set_attribute("data-selected", "false")?;
        tail.set_attribute("data-selected", "false")?;
        blank.set_attribute("data-selected", "false")?;
        match s {
            CellState::Alive => electron.set_attribute("data-selected", "true")?,
            CellState::Dead => tail.set_attribute("data-selected", "true")?,
            CellState::Empty => blank.set_attribute("data-selected", "true")?,
            CellState::Wire => wire.set_attribute("data-selected", "true")?,
        };
        self.draw_brush()?;
        Ok(())
    }
    pub fn render_tiles(&self) -> Result<(), JsValue> {
        let ctx = self
            .canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;
        ctx.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
        let vp = self.tile_viewport();
        for x in vp.x..(vp.x + vp.w) {
            for y in vp.y..(vp.y + vp.h) {
                if let Some(t) = self.world.get_cell(x, y) {
                    self.paint_tile(&self.canvas, t, x, y)?;
                }
            }
        }
        Ok(())
    }
    fn draw_brush(&self) -> Result<(), JsValue> {
        let ctx = self
            .brush_canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;
        ctx.clear_rect(
            0.0,
            0.0,
            self.brush_canvas.width() as f64,
            self.brush_canvas.height() as f64,
        );
        if let Some((x, y)) = self.brush_pos {
            self.paint_tile(&self.brush_canvas, self.brush, x, y)?;
        }
        Ok(())
    }
    fn paint_tile(
        &self,
        canvas: &HtmlCanvasElement,
        tile: CellState,
        x: i32,
        y: i32,
    ) -> Result<(), JsValue> {
        let ctx = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;
        let color = match tile {
            CellState::Alive => "blue",
            CellState::Dead => "grey",
            CellState::Empty => "white",
            CellState::Wire => "orange",
        };
        ctx.set_fill_style(&JsValue::from_str(color));
        ctx.fill_rect(
            (x * self.zoom - self.viewport.x) as f64,
            (y * self.zoom - self.viewport.y) as f64,
            self.zoom as f64,
            self.zoom as f64,
        );
        Ok(())
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
    pub fn tile_viewport(&self) -> Viewport {
        Viewport {
            x: (self.viewport.x / self.zoom) - TILE_BUFFER,
            y: (self.viewport.y / self.zoom) - TILE_BUFFER,
            w: (self.viewport.w / self.zoom) + 1 + TILE_BUFFER * 2,
            h: (self.viewport.h / self.zoom) + 1 + TILE_BUFFER * 2,
        }
    }
    pub fn send_viewport(&self) -> Result<(), JsValue> {
        let tvp = self.tile_viewport();
        let msg = FromClient::SetView {
            x: tvp.x,
            y: tvp.y,
            w: tvp.w,
            h: tvp.h,
        };
        self.socket
            .send_with_str(&serde_json::to_string(&msg).unwrap())?;
        Ok(())
    }
    pub fn process_command(&mut self, cmd: Command) -> Result<(), JsValue> {
        console_log!("cmd get: {:?}", cmd);
        match cmd {
            Command::TileHover { x, y } => {
                self.brush_pos = Some((x, y));
                self.draw_brush()?;
            }
            Command::TileHoverStop => {
                self.brush_pos = None;
                self.draw_brush()?;
            }
            Command::TileClick { x, y } => {
                self.world.set_cell(x, y, self.brush);
                self.paint_tile(&self.canvas, self.brush, x, y)?;
                let msg = FromClient::ModifyCell {
                    x,
                    y,
                    cell: self.brush,
                };
                let json = serde_json::to_string(&msg).unwrap();
                self.socket.send_with_str(&json)?;
            }
            Command::MouseDrag {
                start_x,
                start_y,
                end_x,
                end_y,
            } => {
                self.viewport.x += end_x - start_x;
                self.viewport.y += end_y - start_y;
                self.render_tiles()?;
                self.send_viewport()?;
            }
            Command::Zoom { amount } => console_log!("unimplemented zoom"),
        }
        Ok(())
    }
}
