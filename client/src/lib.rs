use std::{cell::RefCell, rc::Rc};

use state::{Command, MousedownState, State};
use util::{document, window};
use wasm_bindgen::prelude::*;

use web_sys::{MessageEvent, WebSocket};
use wire_universe::{
    proto::{FromClient, FromServer},
    CellState,
};

use crate::{
    keyboard::install_keyhandler,
    state::{Viewport, World},
    util::console_log,
};

mod keyboard;
mod state;
mod util;

pub fn init_websocket(st: Rc<RefCell<State>>) {
    let ws = st.borrow().socket.clone();
    let onmessage_callback = Closure::<dyn FnMut(_)>::new({
        let st = st.clone();
        move |e: MessageEvent| {
            if let Some(txt) = e.data().as_string() {
                if let Ok(val) = serde_json::from_str::<FromServer>(&txt) {
                    match val {
                        FromServer::Refresh { x, y, tiles } => {
                            let st = &mut st.borrow_mut();
                            let world = &mut st.world;
                            world.tiles = tiles;
                            world.x = x;
                            world.y = y;
                            st.render_tiles().unwrap();
                        }
                    }
                }
            } else {
                console_log!("non-string message: {:?}", e.data());
            }
        }
    });
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    let cloned_ws = ws.clone();
    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        st.borrow().send_viewport().unwrap();
        let msg = serde_json::to_string(&FromClient::StartStream).unwrap();
        cloned_ws.send_with_str(&msg).unwrap();
    });
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();
}

fn init_brush(st: Rc<RefCell<State>>, elem_name: &str, cell: CellState) -> Result<(), JsValue> {
    let document = document()?;

    let callback = Closure::<dyn FnMut()>::new(move || {
        st.borrow_mut().set_brush(cell).unwrap();
    });
    document
        .get_element_by_id(elem_name)
        .ok_or(JsValue::from_str(&(format!("#{} missing", elem_name))))?
        .dyn_into::<web_sys::HtmlButtonElement>()?
        .set_onclick(Some(callback.as_ref().unchecked_ref()));
    callback.forget();

    Ok(())
}

fn init_brushes(st: Rc<RefCell<State>>) -> Result<(), JsValue> {
    init_brush(st.clone(), "paint-wire", CellState::Wire)?;
    init_brush(st.clone(), "paint-electron", CellState::Alive)?;
    init_brush(st.clone(), "paint-tail", CellState::Dead)?;
    init_brush(st.clone(), "paint-blank", CellState::Empty)?;
    Ok(())
}

fn init_input_callbacks(st: Rc<RefCell<State>>) {
    let brush_canvas = &st.borrow_mut().brush_canvas;

    {
        let st = st.clone();
        let callback = Closure::<dyn FnMut()>::new(move || {
            let mut st = st.borrow_mut();
            st.process_command(Command::TileHoverStop).unwrap();
            st.mousedown_state = None;
        });
        brush_canvas.set_onmouseleave(Some(callback.as_ref().unchecked_ref()));
        callback.forget();
    }
    {
        let st = st.clone();
        let callback = Closure::<dyn FnMut(_)>::new(move |ev: web_sys::MouseEvent| {
            let mut st = st.borrow_mut();
            let (px, py) = st.mouse_pos_to_pixel(ev.page_x(), ev.page_y());
            st.mousedown_state = Some(MousedownState::Still {
                start_x: px,
                start_y: py,
            });
        });
        brush_canvas.set_onmousedown(Some(callback.as_ref().unchecked_ref()));
        callback.forget();
    }
    {
        let st = st.clone();
        let callback = Closure::<dyn FnMut()>::new(move || {
            let mut st = st.borrow_mut();
            if let Some(MousedownState::Still { start_x, start_y }) = st.mousedown_state {
                let (tx, ty) = st.pixel_to_tile(start_x, start_y);
                st.process_command(Command::TileClick { x: tx, y: ty })
                    .unwrap();
            }
            st.mousedown_state = None;
        });
        brush_canvas.set_onmouseup(Some(callback.as_ref().unchecked_ref()));
        callback.forget();
    }
    {
        let st = st.clone();
        let callback = Closure::<dyn FnMut(_)>::new(move |ev: web_sys::MouseEvent| {
            let mut st = st.borrow_mut();
            let (px, py) = st.mouse_pos_to_pixel(ev.page_x(), ev.page_y());
            if let Some(ms) = &mut st.mousedown_state {
                match ms {
                    &mut MousedownState::Still { start_x, start_y } => {
                        if (((px - start_x).pow(2) + (py - start_y).pow(2)) as f32).sqrt() >= 5. {
                            *ms = MousedownState::Drag {
                                prev_x: px,
                                prev_y: py,
                            };
                            st.process_command(Command::MouseDrag {
                                start_x,
                                start_y,
                                end_x: px,
                                end_y: py,
                            })
                            .unwrap();
                        }
                    }
                    MousedownState::Drag { prev_x, prev_y } => {
                        let x = *prev_x;
                        let y = *prev_y;
                        *prev_x = px;
                        *prev_y = py;
                        st.process_command(Command::MouseDrag {
                            start_x: x,
                            start_y: y,
                            end_x: px,
                            end_y: py,
                        })
                        .unwrap();
                    }
                }
            }
            let (tx, ty) = st.mouse_pos_to_tile(ev.page_x(), ev.page_y());
            st.process_command(Command::TileHover { x: tx, y: ty })
                .unwrap();
        });
        brush_canvas.set_onmousemove(Some(callback.as_ref().unchecked_ref()));
        callback.forget();
    }
}

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    console_log!("Starting wasm");
    let socket = WebSocket::new("ws://localhost:3000/ws")?;
    let document = document()?;
    let canvas = document
        .get_element_by_id("world-canvas")
        .expect("missing #world-canvas")
        .dyn_into()?;
    let brush_canvas = document
        .get_element_by_id("brush-canvas")
        .expect("missing #brush-canvas")
        .dyn_into()?;
    let canvases = document
        .get_element_by_id("canvases")
        .expect("missing #canvases")
        .dyn_into()?;
    let world = World {
        x: 0,
        y: 0,
        tiles: vec![],
    };
    let st = State {
        world,
        viewport: Viewport {
            x: 0,
            y: 0,
            w: 600,
            h: 800,
        },
        brush: CellState::Wire,
        brush_pos: None,
        canvas,
        brush_canvas,
        canvases,
        zoom: 20,
        socket,
        mousedown_state: None,
    };
    let st = Rc::new(RefCell::new(st));
    init_websocket(st.clone());
    init_brushes(st.clone())?;
    install_keyhandler(st.clone())?;
    init_input_callbacks(st);
    return Ok(());
}
