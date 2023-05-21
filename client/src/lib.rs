use js_sys::JsString;
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, WebSocket};
use wire_universe::proto::FromClient;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn init_websockets() -> Result<(), JsValue> {
    let ws = WebSocket::new("ws://localhost:3000/ws")?;
    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        if let Ok(txt) = e.data().dyn_into::<JsString>() {
            console_log!("message event, received Text: {:?}", txt);
        } else {
            console_log!("message event, received Unknown: {:?}", e.data());
        }
    });
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    let cloned_ws = ws.clone();
    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        console_log!("socket opened");
        let msg = serde_json::to_string(&FromClient::StartStream).unwrap();
        cloned_ws.send_with_str(&msg).unwrap();
    });
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    Ok(())
}

fn set_brush(s: &str) {}

fn init_brushes() -> Option<()> {
    let document = web_sys::window().unwrap().document().unwrap();
    let callback = Closure::<dyn FnMut()>::new(|| {
        set_brush("Wire");
    });
    document
        .get_element_by_id("paint-wire")?
        .dyn_into::<web_sys::HtmlButtonElement>()
        .ok()?
        .set_onclick(Some(callback.as_ref().unchecked_ref()));
    callback.forget();
    Some(())
}

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    console_log!("ho");
    // init_websockets()?;
    return Ok(());
}

// let document = web_sys::window().unwrap().document().unwrap();
// let canvas = document.get_element_by_id("canvas").unwrap();
// let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
