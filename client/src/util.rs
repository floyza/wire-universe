use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (crate::util::log(&format_args!($($t)*).to_string()))
}

pub fn document() -> Result<Document, JsValue> {
    if let Some(doc) = web_sys::window().and_then(|x| x.document()) {
        Ok(doc)
    } else {
        Err(JsValue::from_str("nonexistant document"))
    }
}

pub(crate) use console_log;
use web_sys::Document;
