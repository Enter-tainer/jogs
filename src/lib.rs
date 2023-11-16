use std::collections::HashMap;

use anyhow::{bail, Context, Result};
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
fn eval(input: &[u8]) -> Result<Vec<u8>> {
    let context = JSContextRef::default();
    let input = std::str::from_utf8(input).context("failed to parse input as utf8")?;
    let res = from_qjs_value(
        context
            .eval_global("<evalScript>", input)
            .context("failed to convert result to MyJSValue")?,
    )
    .context("failed to convert result to MyJSValue")?;
    let res = MyJSValue::from(res);
    let mut buffer = vec![];
    ciborium::ser::into_writer(&res, &mut buffer).context("failed to serialize result")?;
    Ok(buffer)
}

#[wasm_func]
fn compile(input: &[u8]) -> Result<Vec<u8>> {
    let context = JSContextRef::default();
    let Ok(input) = std::str::from_utf8(input) else {
        bail!("input is not utf8");
    };
    context.compile_global("<compiledScript>", input)
}

#[wasm_func]
fn list_property_keys(input: &[u8]) -> Result<Vec<u8>> {
    let context = JSContextRef::default();
    context.eval_binary(input)?;
    let mut props = context
        .global_object()
        .context("failed to get global object")?
        .properties()
        .context("failed to get properties for global object")?;
    let mut keys: Vec<MyJSValue> = vec![];
    while let Some(key) = props
        .next_key()
        .context("failed to get next key in props")?
    {
        keys.push(
            from_qjs_value(key)
                .context("failed to convert key to MyJSValue")?
                .into(),
        );
    }
    let mut buffer = vec![];
    ciborium::ser::into_writer(&keys, &mut buffer).context("failed to serialize result")?;
    Ok(buffer)
}

#[wasm_func]
fn call_function(bytecode: &[u8], function_name: &[u8], args: &[u8]) -> Result<Vec<u8>> {
    let context = JSContextRef::default();
    let function_name = std::str::from_utf8(function_name).context("function name is not utf8")?;
    let arguments: Vec<MyJSValue> =
        ciborium::from_reader(args).context("failed to deserialize arguments")?;
    let arguments: Vec<JSValueRef> = arguments
        .into_iter()
        .map(|v| {
            let v: JSValue = v.into();
            to_qjs_value(&context, &v)
        })
        .collect::<Result<Vec<_>, _>>()
        .context("failed to convert arguments to JSValueRef")?;
    context.eval_binary(bytecode)?;
    let global_this = context
        .global_object()
        .context("failed to get global object")?;
    let function = global_this
        .get_property(function_name)
        .with_context(|| format!("failed to get function: {}", function_name))?;
    let res = function
        .call(&global_this, &arguments)
        .context("failed to call function")?;
    let res = from_qjs_value(res).context("failed to convert result to MyJSValue")?;
    let res = MyJSValue::from(res);
    let mut buffer = vec![];
    ciborium::ser::into_writer(&res, &mut buffer).context("failed to serialize result")?;
    Ok(buffer)
}
