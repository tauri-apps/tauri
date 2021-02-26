use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RequestBody {
  id: i32,
  name: String,
}

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  LogOperation {
    event: String,
    payload: Option<String>,
  },
  PerformRequest {
    endpoint: String,
    body: RequestBody,
  },
}
