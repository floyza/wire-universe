use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::KeyboardEvent;
use wire_universe::CellState;

use crate::{state::State, util::window};

pub fn install_keyhandler(st: Rc<RefCell<State>>) -> Result<(), JsValue> {
    let callback = Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
        process_key(st.clone(), event);
    });
    window()?.set_onkeydown(Some(callback.as_ref().unchecked_ref()));
    callback.forget();
    Ok(())
}

fn process_key(st: Rc<RefCell<State>>, event: KeyboardEvent) {
    let mut st = st.borrow_mut();
    match event.key().as_ref() {
        "a" => st.set_brush(CellState::Wire).unwrap(),
        "s" => st.set_brush(CellState::Alive).unwrap(),
        "d" => st.set_brush(CellState::Dead).unwrap(),
        "f" => st.set_brush(CellState::Empty).unwrap(),
        _ => {}
    }
}
