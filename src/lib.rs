use std::collections::HashMap;

use quickjs_wasm_rs::{from_qjs_value, to_qjs_value, JSContextRef, JSValue, JSValueRef};
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

impl From<MyJSValue> for JSValue {
    fn from(value: MyJSValue) -> Self {
        match value {
            MyJSValue::Undefined => JSValue::Undefined,
            MyJSValue::Null => JSValue::Null,
            MyJSValue::Bool(b) => JSValue::Bool(b),
            MyJSValue::Int(i) => JSValue::Int(i),
            MyJSValue::Float(f) => JSValue::Float(f),
            MyJSValue::String(s) => JSValue::String(s),
            MyJSValue::Array(arr) => JSValue::Array(arr.into_iter().map(|v| v.into()).collect()),
            MyJSValue::ArrayBuffer(buf) => JSValue::ArrayBuffer(buf),
            MyJSValue::Object(obj) => {
                JSValue::Object(obj.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
        }
    }
}

#[wasm_func]
fn eval(input: &[u8]) -> Result<Vec<u8>, String> {
    let context = JSContextRef::default();
    let input = std::str::from_utf8(input).map_err(|e| e.to_string())?;
    let res = from_qjs_value(
        context
            .eval_global("<evalScript>", input)
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let res = MyJSValue::from(res);
    let mut buffer = vec![];
    ciborium::ser::into_writer(&res, &mut buffer).map_err(|e| e.to_string())?;
    Ok(buffer)
}

#[wasm_func]
fn compile(input: &[u8]) -> Result<Vec<u8>, String> {
    let context = JSContextRef::default();
    let Ok(input) = std::str::from_utf8(input) else {
        return Err("input is not utf8".to_string());
    };
    context
        .compile_global("<compiledScript>", input)
        .map_err(|e| e.to_string())
}

#[wasm_func]
fn list_property_keys(input: &[u8]) -> Result<Vec<u8>, String> {
    let context = JSContextRef::default();
    context.eval_binary(input).map_err(|e| e.to_string())?;
    let mut props = context
        .global_object()
        .map_err(|e| e.to_string())?
        .properties()
        .map_err(|e| e.to_string())?;
    let mut keys: Vec<MyJSValue> = vec![];
    while let Some(key) = props.next_key().map_err(|e| e.to_string())? {
        keys.push(from_qjs_value(key).map_err(|e| e.to_string())?.into());
    }
    let mut buffer = vec![];
    ciborium::ser::into_writer(&keys, &mut buffer).map_err(|e| e.to_string())?;
    Ok(buffer)
}

#[derive(Debug, Serialize, Deserialize)]
struct CallFunction {
    #[serde(with = "serde_bytes")]
    bytecode: Vec<u8>,
    function_name: String,
    arguments: Vec<MyJSValue>,
}

#[wasm_func]
fn call_function(input: &[u8]) -> Result<Vec<u8>, String> {
    let context = JSContextRef::default();
    let CallFunction {
        bytecode,
        function_name,
        arguments,
    } = ciborium::from_reader(input).map_err(|e| e.to_string())?;
    // return Err("not implemented".to_string());
    let arguments: Vec<JSValueRef> = arguments
        .into_iter()
        .map(|v| {
            let v: JSValue = v.into();
            to_qjs_value(&context, &v).map_err(|e| e.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    context.eval_binary(&bytecode).map_err(|e| e.to_string())?;
    let global_this = context.global_object().map_err(|e| e.to_string())?;
    let function = global_this
        .get_property(function_name)
        .map_err(|e| e.to_string())?;
    let res = function
        .call(&global_this, &arguments)
        .map_err(|e| e.to_string())?;
    let res = from_qjs_value(res).map_err(|e| e.to_string())?;
    let res = MyJSValue::from(res);
    let mut buffer = vec![];
    ciborium::ser::into_writer(&res, &mut buffer).map_err(|e| e.to_string())?;
    Ok(buffer)
}
