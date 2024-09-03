// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use axum::{routing::get, Router};
use tower_service::Service;
use worker::*;

mod config;

#[worker::event(fetch)]
async fn main(
  req: HttpRequest,
  _env: Env,
  _ctx: Context,
) -> worker::Result<axum::http::Response<axum::body::Body>> {
  console_error_panic_hook::set_once();
  Ok(router().call(req).await?)
}

fn router() -> Router {
  Router::new().route("/", get(root)).merge(config::router())
}

async fn root() -> &'static str {
  "tauri schema worker"
}
