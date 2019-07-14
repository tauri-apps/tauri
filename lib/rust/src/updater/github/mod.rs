mod release;
pub use release::*;
pub use super::error::Error;

use super::super::http;

pub fn get_latest_release(repo_owner: &str, repo_name: &str) -> Result<Release, Error> {
    set_ssl_vars!();
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        repo_owner, repo_name
    );
    let mut resp = http::get(&api_url)?;
    if !resp.status().is_success() {
        bail!(
            Error::Network,
            "api request failed with status: {:?} - for: {:?}",
            resp.status(),
            api_url
        )
    }
    let json = resp.json::<serde_json::Value>()?;
    Ok(Release::parse(&json)?)
}

pub fn get_release_version(repo_owner: &str, repo_name: &str, ver: &str) -> Result<Release, Error> {
    set_ssl_vars!();
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/releases/tags/{}",
        repo_owner, repo_name, ver
    );
    let mut resp = http::get(&api_url)?;
    if !resp.status().is_success() {
        bail!(
            Error::Network,
            "api request failed with status: {:?} - for: {:?}",
            resp.status(),
            api_url
        )
    }
    let json = resp.json::<serde_json::Value>()?;
    Ok(Release::parse(&json)?)
}
