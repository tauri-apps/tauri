mod release;

pub use release::*;

use crate::http;

pub fn get_latest_release(repo_owner: &str, repo_name: &str) -> crate::Result<Release> {
  set_ssl_vars!();
  let api_url = format!(
    "https://api.github.com/repos/{}/{}/releases/latest",
    repo_owner, repo_name
  );
  let resp = http::get(&api_url)?;
  if !resp.status().is_success() {
    bail!(
      crate::ErrorKind::Network,
      "api request failed with status: {:?} - for: {:?}",
      resp.status(),
      api_url
    )
  }
  let json = resp.json::<serde_json::Value>()?;
  Ok(Release::parse(&json)?)
}

pub fn get_release_version(repo_owner: &str, repo_name: &str, ver: &str) -> crate::Result<Release> {
  set_ssl_vars!();
  let api_url = format!(
    "https://api.github.com/repos/{}/{}/releases/tags/{}",
    repo_owner, repo_name, ver
  );
  let resp = http::get(&api_url)?;
  if !resp.status().is_success() {
    bail!(
      crate::ErrorKind::Network,
      "api request failed with status: {:?} - for: {:?}",
      resp.status(),
      api_url
    )
  }
  let json = resp.json::<serde_json::Value>()?;
  Ok(Release::parse(&json)?)
}
