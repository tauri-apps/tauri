use tauri::{Manager, Runtime};

use crate::models::*;

impl<R: Runtime, T: Manager<R>> crate::SampleExt<R> for T {
  fn ping(&self, payload: PingRequest) -> tauri::Result<Result<PingResponse, String>> {
    Ok(ping(payload))
  }
}

fn ping(payload: PingRequest) -> Result<PingResponse, String> {
  Ok(PingResponse {
    value: payload.value,
  })
}
