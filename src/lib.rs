use std::collections::HashMap;

use quickjs_wasm_rs::{from_qjs_value, JSContextRef, JSValue};
use serde::{Deserialize, Serialize};

use wasm_minimal_protocol::*;

initiate_protocol!();

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MyJSValue {
    /// Represents the JavaScript `undefined` value
    Undefined,
    /// Represents the JavaScript `null` value
    Null,
    /// Represents a JavaScript boolean value
    Bool(bool),
    /// Represents a JavaScript integer
    Int(i32),
    /// Represents a JavaScript floating-point number
    Float(f64),
    /// Represents a JavaScript string value
    String(String),
    /// Represents a JavaScript array of `JSValue`s
    Array(Vec<MyJSValue>),
    /// Represents a JavaScript ArrayBuffer of bytes
    ArrayBuffer(Vec<u8>),
    /// Represents a JavaScript object, with string keys and `JSValue` values
    Object(HashMap<String, MyJSValue>),
}

impl From<JSValue> for MyJSValue {
    fn from(value: JSValue) -> Self {
        match value {
            JSValue::Undefined => MyJSValue::Undefined,
            JSValue::Null => MyJSValue::Null,
            JSValue::Bool(b) => MyJSValue::Bool(b),
            JSValue::Int(i) => MyJSValue::Int(i),
            JSValue::Float(f) => MyJSValue::Float(f),
            JSValue::String(s) => MyJSValue::String(s),
            JSValue::Array(arr) => MyJSValue::Array(arr.into_iter().map(|v| v.into()).collect()),
            JSValue::ArrayBuffer(buf) => MyJSValue::ArrayBuffer(buf),
            JSValue::Object(obj) => {
                MyJSValue::Object(obj.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
        }
    }
}

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
    let res = MyJSValue::from(res);
    match serde_cbor::to_vec(&res) {
        Ok(res) => Ok(res),
        Err(err) => Err(err.to_string()),
    }
}
