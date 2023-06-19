use std::{cell::RefCell, rc::Rc};

use js_sys::JsString;
use state::{Command, MousedownState, State};
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, WebSocket};
use wire_universe::{proto::FromClient, CellState};

use crate::{
    state::{Viewport, World},
    util::console_log,
};

mod state;
mod util;

fn init_websockets() -> Result<WebSocket, JsValue> {
    let ws = WebSocket::new("ws://localhost:3000/ws")?;
    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        if let Ok(txt) = e.data().dyn_into::<JsString>() {
            // console_log!("message event, received Text: {:?}", txt);
        } else {
            // console_log!("message event, received Unknown: {:?}", e.data());
        }
    });
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    let cloned_ws = ws.clone();
    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        let msg = serde_json::to_string(&FromClient::StartStream).unwrap();
        cloned_ws.send_with_str(&msg).unwrap();
    });
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    Ok(ws)
}

fn init_brushes() -> Option<()> {
    let document = web_sys::window().unwrap().document().unwrap();
    let callback = Closure::<dyn FnMut()>::new(|| {
        console_log!("pressed button");
        // set_brush(CellState::Wire);
    });
    document
        .get_element_by_id("paint-wire")?
        .dyn_into::<web_sys::HtmlButtonElement>()
        .ok()?
        .set_onclick(Some(callback.as_ref().unchecked_ref()));
    callback.forget();

    Some(())
}

fn init_input_callbacks(st: Rc<RefCell<State>>) {
    let brush_canvas = &st.borrow_mut().brush_canvas;

    {
        let st = st.clone();
        let callback = Closure::<dyn FnMut()>::new(move || {
            st.borrow_mut().process_command(Command::TileHoverStop);
        });
        brush_canvas.set_onmouseleave(Some(callback.as_ref().unchecked_ref()));
        callback.forget();
    }
    {
        let st = st.clone();
        let callback = Closure::<dyn FnMut(_)>::new(move |ev: web_sys::MouseEvent| {
            let mut st = st.borrow_mut();
            st.process_command(Command::TileHoverStop);
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
                st.process_command(Command::TileClick { x: tx, y: ty });
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
                            });
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
                        });
                    }
                }
            }
        });
        brush_canvas.set_onmousemove(Some(callback.as_ref().unchecked_ref()));
        callback.forget();
    }
}

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    console_log!("Starting wasm");
    let socket = init_websockets()?;
    let document = web_sys::window().unwrap().document().unwrap();
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
    init_brushes().unwrap();
    init_input_callbacks(st);
    return Ok(());
}
