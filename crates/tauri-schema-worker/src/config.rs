// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use anyhow::Context;
use axum::{
  extract::Path,
  http::{header, StatusCode},
  response::Result,
  routing::get,
  Router,
};
use semver::{Version, VersionReq};
use serde::Deserialize;
use worker::*;

#[derive(Deserialize)]
pub struct CrateReleases {
  pub versions: Vec<CrateRelease>,
}

#[derive(Debug, Deserialize)]
pub struct CrateRelease {
  #[serde(alias = "num")]
  pub version: Version,
  pub yanked: Option<bool>,
}

#[derive(Deserialize)]
pub struct CrateMetadataFull {
  #[serde(rename = "crate")]
  pub crate_: CrateMetadata,
}

#[derive(Deserialize)]
pub struct CrateMetadata {
  pub max_stable_version: Version,
}

const USERAGENT: &str = "tauri-schema-worker (contact@tauri.app)";

pub fn router() -> Router {
  Router::new()
    .route("/config", get(stable_schema))
    .route("/config/latest", get(stable_schema))
    .route("/config/stable", get(stable_schema))
    .route("/config/next", get(next_schema)) // pre-releases versions, (rc, alpha and beta)
    .route("/config/:version", get(schema_for_version))
}

async fn schema_for_version(Path(version): Path<String>) -> Result<String> {
  try_schema_for_version(version)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    .map_err(Into::into)
}

async fn stable_schema() -> Result<String> {
  try_stable_schema()
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    .map_err(Into::into)
}

async fn next_schema() -> Result<String> {
  try_next_schema()
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    .map_err(Into::into)
}

#[worker::send]
async fn try_schema_for_version(version: String) -> anyhow::Result<String> {
  let version = version.parse::<VersionReq>()?;

  let releases = crate_releases("tauri").await?;

  if releases.is_empty() {
    return try_stable_schema().await;
  }

  let Some(version) = releases.into_iter().find(|r| version.matches(&r.version)) else {
    return try_stable_schema().await;
  };

  schema_file_for_version(version.version).await
}

#[worker::send]
async fn try_stable_schema() -> anyhow::Result<String> {
  let max = stable_version("tauri").await?;
  schema_file_for_version(max).await
}

#[worker::send]
async fn try_next_schema() -> anyhow::Result<String> {
  let releases = crate_releases("tauri").await?;
  let version = releases
    .into_iter()
    .filter(|r| !r.version.pre.is_empty())
    .map(|r| r.version)
    .max()
    .context("Couldn't find latest pre-release")?;
  schema_file_for_version(version).await
}

async fn schema_file_for_version(version: Version) -> anyhow::Result<String> {
  let cache = Cache::open("schema".to_string()).await;
  let cache_key = format!("https://schema.tauri.app/config/{version}");
  if let Some(mut cached) = cache.get(cache_key.clone(), true).await? {
    console_log!("Serving schema for {version} from cache");
    return cached.text().await.map_err(Into::into);
  }

  console_log!("Fetching schema for {version} from remote");

  let path = if version.major >= 2 {
    "crates/tauri-schema-generator/schemas/config.schema.json"
  } else {
    "core/tauri-config-schema/schema.json"
  };
  let url = format!("https://raw.githubusercontent.com/tauri-apps/tauri/tauri-v{version}/{path}");
  let mut res = Fetch::Request(fetch_req(&url)?).send().await?;

  cache.put(cache_key, res.cloned()?).await?;

  res.text().await.map_err(Into::into)
}

async fn crate_releases(crate_: &str) -> anyhow::Result<Vec<CrateRelease>> {
  let url = format!("https://crates.io/api/v1/crates/{crate_}/versions");
  let mut res = Fetch::Request(fetch_req(&url)?).send().await?;

  let versions: CrateReleases = res.json().await?;
  let versions = versions.versions;

  let flt = |r: &CrateRelease| r.yanked == Some(false);
  Ok(versions.into_iter().filter(flt).collect())
}

async fn stable_version(crate_: &str) -> anyhow::Result<Version> {
  let url = format!("https://crates.io/api/v1/crates/{crate_}");
  let mut res = Fetch::Request(fetch_req(&url)?).send().await?;
  let metadata: CrateMetadataFull = res.json().await?;
  Ok(metadata.crate_.max_stable_version)
}

fn fetch_req(url: &str) -> anyhow::Result<worker::Request> {
  let mut headers = Headers::new();
  headers.append(header::USER_AGENT.as_str(), USERAGENT)?;

  worker::Request::new_with_init(
    url,
    &RequestInit {
      method: Method::Get,
      headers,
      cf: CfProperties {
        cache_ttl: Some(86400),
        cache_everything: Some(true),
        cache_ttl_by_status: Some(
          [
            ("200-299".to_string(), 86400),
            ("404".to_string(), 1),
            ("500-599".to_string(), 0),
          ]
          .into(),
        ),
        ..Default::default()
      },
      ..Default::default()
    },
  )
  .map_err(Into::into)
}
