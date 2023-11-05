use quickjs_wasm_rs::JSContextRef;
use wasm_minimal_protocol::*;

initiate_protocol!();

#[wasm_func]
fn eval(input: &[u8]) -> Result<Vec<u8>, String> {
    let context = JSContextRef::default();
    let Ok(input) = std::str::from_utf8(input) else {
        return Err("input is not utf8".to_string());
    };
    let res = match context.eval_global("test.js", input) {
        Ok(res) => res,
        Err(err) => return Err(err.to_string()),
    };
    Ok(res.to_string().into_bytes())
}
