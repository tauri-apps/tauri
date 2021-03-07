use serde::Deserialize;
use tauri::command;

#[derive(Debug, Deserialize)]
pub struct RequestBody {
  id: i32,
  name: String,
}

#[command]
pub fn log_operation(event: String, payload: Option<String>) {
  println!("{} {:?}", event, payload);
}

#[command]
pub fn perform_request(endpoint: String, body: RequestBody) -> String {
  println!("{} {:?}", endpoint, body);
  "message response".into()
}
