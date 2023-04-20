// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Runtime;
use jni::{
  errors::Error as JniError,
  objects::{JObject, JValue},
  JNIEnv,
};
use serde_json::Value as JsonValue;
use tauri_runtime::RuntimeHandle;

fn json_to_java<'a, R: Runtime>(
  env: JNIEnv<'a>,
  activity: JObject<'a>,
  runtime_handle: &R::Handle,
  json: &JsonValue,
) -> Result<(&'static str, JValue<'a>), JniError> {
  let (class, v) = match json {
    JsonValue::Null => ("Ljava/lang/Object;", JObject::null().into()),
    JsonValue::Bool(val) => ("Z", (*val).into()),
    JsonValue::Number(val) => {
      if let Some(v) = val.as_i64() {
        ("J", v.into())
      } else if let Some(v) = val.as_f64() {
        ("D", v.into())
      } else {
        ("Ljava/lang/Object;", JObject::null().into())
      }
    }
    JsonValue::String(val) => (
      "Ljava/lang/Object;",
      JObject::from(env.new_string(&val)?).into(),
    ),
    JsonValue::Array(val) => {
      let js_array_class = runtime_handle.find_class(env, activity, "app/tauri/plugin/JSArray")?;
      let data = env.new_object(js_array_class, "()V", &[])?;

      for v in val {
        let (signature, val) = json_to_java::<R>(env, activity, runtime_handle, v)?;
        env.call_method(
          data,
          "put",
          format!("({signature})Lorg/json/JSONArray;"),
          &[val],
        )?;
      }

      ("Ljava/lang/Object;", data.into())
    }
    JsonValue::Object(val) => {
      let js_object_class =
        runtime_handle.find_class(env, activity, "app/tauri/plugin/JSObject")?;
      let data = env.new_object(js_object_class, "()V", &[])?;

      for (key, value) in val {
        let (signature, val) = json_to_java::<R>(env, activity, runtime_handle, value)?;
        env.call_method(
          data,
          "put",
          format!("(Ljava/lang/String;{signature})Lapp/tauri/plugin/JSObject;"),
          &[env.new_string(&key)?.into(), val],
        )?;
      }

      ("Ljava/lang/Object;", data.into())
    }
  };
  Ok((class, v))
}

pub fn to_jsobject<'a, R: Runtime>(
  env: JNIEnv<'a>,
  activity: JObject<'a>,
  runtime_handle: &R::Handle,
  json: &JsonValue,
) -> Result<JValue<'a>, JniError> {
  if let JsonValue::Object(_) = json {
    json_to_java::<R>(env, activity, runtime_handle, json).map(|(_class, data)| data)
  } else {
    // currently the Kotlin lib cannot handle nulls or raw values, it must be an object
    let js_object_class = runtime_handle.find_class(env, activity, "app/tauri/plugin/JSObject")?;
    let data = env.new_object(js_object_class, "()V", &[])?;
    Ok(data.into())
  }
}
