use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

fn main() {
  let mut out = io::stdout().lock();
  writeln!(out, "{}", json!({"protocol": 1})).unwrap();
  out.flush().unwrap();
  for line in io::stdin().lock().lines() {
    let request: Value = serde_json::from_str(&line.unwrap()).unwrap();
    let count = request["state"]["count"].as_i64().unwrap_or(0);
    let count = if request["command"] == "increment" {
      count + 1
    } else {
      count
    };
    // Change this text, save, then invoke again: count and WebView survive.
    let result = json!({"count": count, "label": "Rust generation one"});
    writeln!(
      out,
      "{}",
      json!({"state": {"count": count}, "result": result})
    )
    .unwrap();
    out.flush().unwrap();
  }
}
