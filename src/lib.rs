use quickjs_wasm_rs::{JSContextRef, from_qjs_value};
use wasm_minimal_protocol::*;

initiate_protocol!();

#[wasm_func]
fn eval(input: &[u8]) -> Result<Vec<u8>, String> {
    let context = JSContextRef::default();
    let Ok(input) = std::str::from_utf8(input) else {
        return Err("input is not utf8".to_string());
    };
    let res = match context.eval_global("<evalScript>", input) {
        Ok(res) => match from_qjs_value(res) {
            Ok(res) => res,
            Err(err) => return Err(err.to_string()),
        },
        Err(err) => return Err(err.to_string()),
    };
    match serde_cbor::to_vec(&res) {
        Ok(res) => Ok(res),
        Err(err) => Err(err.to_string()),
    }
}
