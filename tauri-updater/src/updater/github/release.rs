use crate::http::link_value::{LinkValue, RelationType};

use serde_json;

/// GitHub release-asset information
#[derive(Clone, Debug)]
pub struct ReleaseAsset {
  pub download_url: String,
  pub name: String,
}
impl ReleaseAsset {
  /// Parse a release-asset json object
  ///
  /// Errors:
  ///     * Missing required name & browser_download_url keys
  fn from_asset(asset: &serde_json::Value) -> crate::Result<ReleaseAsset> {
    let download_url = asset["browser_download_url"].as_str().ok_or_else(|| {
      format_err!(
        crate::ErrorKind::Network,
        "Asset missing `browser_download_url`"
      )
    })?;
    let name = asset["name"]
      .as_str()
      .ok_or_else(|| format_err!(crate::ErrorKind::Network, "Asset missing `name`"))?;
    Ok(ReleaseAsset {
      download_url: download_url.to_owned(),
      name: name.to_owned(),
    })
  }
}

#[derive(Clone, Debug)]
pub struct Release {
  pub name: String,
  pub body: String,
  pub tag: String,
  pub date_created: String,
  pub assets: Vec<ReleaseAsset>,
}
impl Release {
  pub fn parse(release: &serde_json::Value) -> crate::Result<Release> {
    let tag = release["tag_name"]
      .as_str()
      .ok_or_else(|| format_err!(crate::ErrorKind::Network, "Release missing `tag_name`"))?;
    let date_created = release["created_at"]
      .as_str()
      .ok_or_else(|| format_err!(crate::ErrorKind::Network, "Release missing `created_at`"))?;
    let name = release["name"].as_str().unwrap_or(tag);
    let body = release["body"].as_str().unwrap_or("");
    let assets = release["assets"]
      .as_array()
      .ok_or_else(|| format_err!(crate::ErrorKind::Network, "No assets found"))?;
    let assets = assets
      .iter()
      .map(ReleaseAsset::from_asset)
      .collect::<crate::Result<Vec<ReleaseAsset>>>()?;
    Ok(Release {
      name: name.to_owned(),
      body: body.to_owned(),
      tag: tag.to_owned(),
      date_created: date_created.to_owned(),
      assets,
    })
  }

  /// Check if release has an asset who's name contains the specified `target`
  pub fn has_target_asset(&self, target: &str) -> bool {
    self.assets.iter().any(|asset| asset.name.contains(target))
  }

  /// Return the first `ReleaseAsset` for the current release who's name
  /// contains the specified `target`
  pub fn asset_for(&self, target: &str) -> Option<ReleaseAsset> {
    self
      .assets
      .iter()
      .filter(|asset| asset.name.contains(target))
      .cloned()
      .nth(0)
  }

  pub fn version(&self) -> &str {
    self.tag.trim_start_matches('v')
  }
}

/// `ReleaseList` Builder
#[derive(Clone, Debug)]
pub struct ReleaseListBuilder {
  repo_owner: Option<String>,
  repo_name: Option<String>,
  target: Option<String>,
}
impl ReleaseListBuilder {
  /// Set the repo owner, used to build a github api url
  pub fn repo_owner(&mut self, owner: &str) -> &mut Self {
    self.repo_owner = Some(owner.to_owned());
    self
  }

  /// Set the repo name, used to build a github api url
  pub fn repo_name(&mut self, name: &str) -> &mut Self {
    self.repo_name = Some(name.to_owned());
    self
  }

  /// Set the optional arch `target` name, used to filter available releases
  pub fn target(&mut self, target: &str) -> &mut Self {
    self.target = Some(target.to_owned());
    self
  }

  /// Verify builder args, returning a `ReleaseList`
  pub fn build(&self) -> crate::Result<ReleaseList> {
    Ok(ReleaseList {
      repo_owner: if let Some(ref owner) = self.repo_owner {
        owner.to_owned()
      } else {
        bail!(crate::ErrorKind::Config, "`repo_owner` required")
      },
      repo_name: if let Some(ref name) = self.repo_name {
        name.to_owned()
      } else {
        bail!(crate::ErrorKind::Config, "`repo_name` required")
      },
      target: self.target.clone(),
    })
  }
}

/// `ReleaseList` provides a builder api for querying a GitHub repo,
/// returning a `Vec` of available `Release`s
#[derive(Clone, Debug)]
pub struct ReleaseList {
  repo_owner: String,
  repo_name: String,
  target: Option<String>,
}
impl ReleaseList {
  /// Initialize a ReleaseListBuilder
  pub fn configure() -> ReleaseListBuilder {
    ReleaseListBuilder {
      repo_owner: None,
      repo_name: None,
      target: None,
    }
  }

  /// Retrieve a list of `Release`s.
  /// If specified, filter for those containing a specified `target`
  pub fn fetch(self) -> crate::Result<Vec<Release>> {
    set_ssl_vars!();
    let api_url = format!(
      "https://api.github.com/repos/{}/{}/releases",
      self.repo_owner, self.repo_name
    );
    let releases = Self::fetch_releases(&api_url)?;
    let releases = match self.target {
      None => releases,
      Some(ref target) => releases
        .into_iter()
        .filter(|r| r.has_target_asset(target))
        .collect::<Vec<_>>(),
    };
    Ok(releases)
  }

  fn fetch_releases(url: &str) -> crate::Result<Vec<Release>> {
    let (status, headers, reader) = attohttpc::get(url).send()?.split();

    if !status.is_success() {
      bail!(
        crate::ErrorKind::Network,
        "api request failed with status: {:?} - for: {:?}",
        status,
        url
      )
    }

    let releases = reader.json::<serde_json::Value>()?.clone();
    let releases = releases
      .as_array()
      .ok_or_else(|| format_err!(crate::ErrorKind::Network, "No releases found"))?;
    let mut releases = releases
      .iter()
      .map(Release::parse)
      .collect::<crate::Result<Vec<Release>>>()?;

    // handle paged responses containing `Link` header:
    // `Link: <https://api.github.com/resource?page=2>; rel="next"`
    let links = headers.get_all(attohttpc::header::LINK);

    let next_link = links
      .iter()
      .filter_map(|link| {
        if let Ok(link) = link.to_str() {
          let lv = LinkValue::new(link.to_owned());
          if let Some(rels) = lv.rel() {
            if rels.contains(&RelationType::Next) {
              return Some(link);
            }
          }
          None
        } else {
          None
        }
      })
      .nth(0);

    Ok(match next_link {
      None => releases,
      Some(link) => {
        releases.extend(Self::fetch_releases(link)?);
        releases
      }
    })
  }
}
