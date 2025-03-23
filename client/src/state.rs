use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, WebSocket};
use wire_universe::{proto::FromClient, CellState, Point};

use crate::util::{console_log, document};

#[derive(Debug, Clone)]
pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

const TILE_BUFFER: f64 = 0.3;

#[derive(Debug, Clone)]
pub struct World {
    pub x: i32,
    pub y: i32,
    pub tiles: Vec<Vec<CellState>>,
}

impl World {
    // step the automaton, provided with with the new outside cells
    pub fn step(&mut self, data: Vec<CellState>) {
        let ot = self.tiles.clone();
        for y in 1..self.tiles.len() - 1 {
            for x in 1..self.tiles.len() - 1 {
                let pos = Point {
                    x: x as i32,
                    y: y as i32,
                };
                let next = match ot[y][x] {
                    CellState::Alive => CellState::Dead,
                    CellState::Dead => CellState::Wire,
                    CellState::Empty => CellState::Empty,
                    CellState::Wire => {
                        let mut n = 0;
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
                            if ot[i.y as usize][i.x as usize] == CellState::Alive {
                                n += 1;
                            }
                        }
                        if n == 1 || n == 2 {
                            CellState::Alive
                        } else {
                            CellState::Wire
                        }
                    }
                };
                self.tiles[y][x] = next;
            }
            assert_eq!(
                data.len(),
                self.tiles.len() * 2 + self.tiles[0].len() * 2 - 4,
                "wrongly sized data: data != expected"
            );
            let mut i = 0;
            let w = self.tiles[0].len();
            let h = self.tiles.len();
            for y in 0..h {
                self.tiles[y][0] = data[i];
                i += 1;
            }
            for x in 1..w {
                self.tiles[h - 1][x] = data[i];
                i += 1;
            }
            for y in (1..h).rev() {
                self.tiles[y][w - 1] = data[i];
                i += 1;
            }
            for x in (1..w - 1).rev() {
                self.tiles[0][x] = data[i];
                i += 1;
            }
        }
    }
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
    pub zoom_float: f64,
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
        amount: f64,
    },
}

impl State {
    pub fn set_brush(&mut self, s: CellState) -> Result<(), JsValue> {
        self.brush = s;
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
            self.paint_tile_outlined(&self.brush_canvas, self.brush, x, y)?;
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
    fn paint_tile_outlined(
        &self,
        canvas: &HtmlCanvasElement,
        tile: CellState,
        x: i32,
        y: i32,
    ) -> Result<(), JsValue> {
        self.paint_tile(canvas, tile, x, y)?;
        let ctx = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;
        ctx.set_stroke_style(&JsValue::from_str("black"));
        ctx.stroke_rect(
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
            (x + self.viewport.x).div_euclid(self.zoom),
            (y + self.viewport.y).div_euclid(self.zoom),
        )
    }
    pub fn mouse_pos_to_tile(&self, x: i32, y: i32) -> (i32, i32) {
        let (x, y) = self.mouse_pos_to_pixel(x, y);
        self.pixel_to_tile(x, y)
    }
    // get the center of the tile in pixels from the top left of the canvas
    pub fn tile_to_pixel(&self, x: i32, y: i32) -> (i32, i32) {
        (
            x * self.zoom - self.viewport.x,
            y * self.zoom - self.viewport.y,
        )
    }
    pub fn tile_viewport(&self) -> Viewport {
        let buffer_x = ((self.viewport.w / self.zoom) as f64 * TILE_BUFFER) as i32;
        let buffer_y = ((self.viewport.h / self.zoom) as f64 * TILE_BUFFER) as i32;
        // w and h are both always positive, so div_euclid == normal division
        Viewport {
            x: (self.viewport.x.div_euclid(self.zoom)) - buffer_x,
            y: (self.viewport.y.div_euclid(self.zoom)) - buffer_y,
            w: (self.viewport.w / self.zoom) + 1 + buffer_x * 2,
            h: (self.viewport.h / self.zoom) + 1 + buffer_y * 2,
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
            .send_with_u8_array(&rmp_serde::to_vec(&msg).unwrap())?;
        Ok(())
    }
    fn set_zoom(&mut self, zoom: i32) -> Result<(), JsValue> {
        let ratio = zoom as f64 / self.zoom as f64;
        let (tgx, tgy) = if let Some((tx, ty)) = self.brush_pos {
            let (px, py) = self.tile_to_pixel(tx, ty);
            (px as f64, py as f64)
        } else {
            (self.viewport.w as f64 / 2.0, self.viewport.h as f64 / 2.0)
        };
        self.viewport.x = ((self.viewport.x as f64 + tgx) * ratio - tgx).round() as i32;
        self.viewport.y = ((self.viewport.y as f64 + tgy) * ratio - tgy).round() as i32;
        self.zoom = zoom;

        self.render_tiles()?;
        self.send_viewport()?;
        self.draw_brush()?;
        Ok(())
    }
    pub fn process_command(&mut self, cmd: Command) -> Result<(), JsValue> {
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
                self.socket
                    .send_with_u8_array(&rmp_serde::to_vec(&msg).unwrap())?;
            }
            Command::MouseDrag {
                start_x,
                start_y,
                end_x,
                end_y,
            } => {
                self.viewport.x -= end_x - start_x;
                self.viewport.y -= end_y - start_y;
                self.render_tiles()?;
                self.send_viewport()?;
            }
            Command::Zoom { amount } => {
                self.zoom_float *= 1.05_f64.powf(amount);
                if self.zoom_float < 5. {
                    self.zoom_float = 5.;
                }
                if self.zoom_float > 80. {
                    self.zoom_float = 80.;
                }
                let new_zoom = self.zoom_float as i32;
                if new_zoom != self.zoom {
                    self.set_zoom(new_zoom)?;
                }
            }
        }
        Ok(())
    }
    pub fn sync_canvas_size(&self) {
        self.canvas.set_width(self.viewport.w as u32);
        self.canvas.set_height(self.viewport.h as u32);
        self.brush_canvas.set_width(self.viewport.w as u32);
        self.brush_canvas.set_height(self.viewport.h as u32);
    }
}
