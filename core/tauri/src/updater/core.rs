// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::error::{Error, Result};
#[cfg(feature = "updater")]
use crate::api::file::{ArchiveFormat, Extract, Move};
use crate::{
  api::{
    http::{ClientBuilder, HttpRequestBuilder},
    version,
  },
  AppHandle, Manager, Runtime,
};
use base64::decode;
use http::{
  header::{HeaderName, HeaderValue},
  HeaderMap, StatusCode,
};
use minisign_verify::{PublicKey, Signature};
use tauri_utils::{platform::current_exe, Env};

#[cfg(feature = "updater")]
use std::io::Seek;
use std::{
  env, fmt,
  io::{Cursor, Read},
  path::{Path, PathBuf},
  str::from_utf8,
  time::Duration,
};

#[cfg(feature = "updater")]
#[cfg(not(target_os = "macos"))]
use std::ffi::OsStr;

#[cfg(all(feature = "updater", not(target_os = "windows")))]
use crate::api::file::Compression;

#[cfg(target_os = "windows")]
use std::{
  fs::read_dir,
  process::{exit, Command},
};

#[derive(Debug)]
pub struct RemoteRelease {
  /// Version to install
  pub version: String,
  /// Release date
  pub date: String,
  /// Download URL for current platform
  pub download_url: String,
  /// Update short description
  pub body: Option<String>,
  /// Optional signature for the current platform
  pub signature: String,
  #[cfg(target_os = "windows")]
  /// Optional: Windows only try to use elevated task
  pub with_elevated_task: bool,
}

impl RemoteRelease {
  // Read JSON and confirm this is a valid Schema
  fn from_release(release: &serde_json::Value, target: &str) -> Result<RemoteRelease> {
    // Version or name is required for static and dynamic JSON
    // if `version` is not announced, we fallback to `name` (can be the tag name example v1.0.0)
    let version = match release.get("version") {
      Some(version) => version
        .as_str()
        .ok_or_else(|| Error::InvalidResponseType("version", "string", version.clone()))?
        .trim_start_matches('v')
        .to_string(),
      None => {
        let name = release
          .get("name")
          .ok_or(Error::MissingResponseField("version or name"))?;
        name
          .as_str()
          .ok_or_else(|| Error::InvalidResponseType("name", "string", name.clone()))?
          .trim_start_matches('v')
          .to_string()
      }
    };

    // pub_date is required default is: `N/A` if not provided by the remote JSON
    let date = if let Some(date) = release.get("pub_date") {
      date
        .as_str()
        .map(|d| d.to_string())
        .ok_or_else(|| Error::InvalidResponseType("pub_date", "string", date.clone()))?
    } else {
      "N/A".into()
    };

    // body is optional to build our update
    let body = if let Some(notes) = release.get("notes") {
      Some(
        notes
          .as_str()
          .map(|n| n.to_string())
          .ok_or_else(|| Error::InvalidResponseType("notes", "string", notes.clone()))?,
      )
    } else {
      None
    };

    let download_url;
    #[cfg(target_os = "windows")]
    let with_elevated_task;
    let signature;

    match release.get("platforms") {
      //
      // Did we have a platforms field?
      // If we did, that mean it's a static JSON.
      // The main difference with STATIC and DYNAMIC is static announce ALL platforms
      // and dynamic announce only the current platform.
      //
      // This could be used if you do NOT want an update server and use
      // a GIST, S3 or any static JSON file to announce your updates.
      //
      // Notes:
      // Dynamic help to reduce bandwidth usage or to intelligently update your clients
      // based on the request you give. The server can remotely drive behaviors like
      // rolling back or phased rollouts.
      //
      Some(platforms) => {
        // make sure we have our target available
        if let Some(current_target_data) = platforms.get(target) {
          // use provided signature if available
          signature = current_target_data
            .get("signature")
            .ok_or(Error::MissingResponseField("signature"))
            .and_then(|signature| {
              signature
                .as_str()
                .ok_or_else(|| Error::InvalidResponseType("signature", "string", signature.clone()))
            })?;
          // Download URL is required
          let url = current_target_data
            .get("url")
            .ok_or(Error::MissingResponseField("url"))?;
          download_url = url
            .as_str()
            .ok_or_else(|| Error::InvalidResponseType("url", "string", url.clone()))?
            .to_string();
          #[cfg(target_os = "windows")]
          {
            with_elevated_task = current_target_data
              .get("with_elevated_task")
              .map(|v| {
                v.as_bool().ok_or_else(|| {
                  Error::InvalidResponseType("with_elevated_task", "boolean", v.clone())
                })
              })
              .unwrap_or(Ok(false))?;
          }
        } else {
          // make sure we have an available platform from the static
          return Err(Error::TargetNotFound(target.into()));
        }
      }
      // We don't have the `platforms` field announced, let's assume our
      // download URL is at the root of the JSON.
      None => {
        signature = release
          .get("signature")
          .ok_or(Error::MissingResponseField("signature"))
          .and_then(|signature| {
            signature
              .as_str()
              .ok_or_else(|| Error::InvalidResponseType("signature", "string", signature.clone()))
          })?;
        let url = release
          .get("url")
          .ok_or(Error::MissingResponseField("url"))?;
        download_url = url
          .as_str()
          .ok_or_else(|| Error::InvalidResponseType("url", "string", url.clone()))?
          .to_string();
        #[cfg(target_os = "windows")]
        {
          with_elevated_task = match release.get("with_elevated_task") {
            Some(with_elevated_task) => with_elevated_task.as_bool().unwrap_or(false),
            None => false,
          };
        }
      }
    }
    // Return our formatted release
    Ok(RemoteRelease {
      version,
      date,
      download_url,
      body,
      signature: signature.to_string(),
      #[cfg(target_os = "windows")]
      with_elevated_task,
    })
  }
}

pub struct UpdateBuilder<R: Runtime> {
  /// Application handle.
  pub app: AppHandle<R>,
  /// Current version we are running to compare with announced version
  pub current_version: String,
  /// The URLs to checks updates. We suggest at least one fallback on a different domain.
  pub urls: Vec<String>,
  /// The platform the updater will check and install the update. Default is from `get_updater_target`
  pub target: Option<String>,
  /// The current executable path. Default is automatically extracted.
  pub executable_path: Option<PathBuf>,
  should_install: Option<Box<dyn FnOnce(&str, &str) -> bool + Send>>,
  timeout: Option<Duration>,
  headers: HeaderMap,
}

impl<R: Runtime> fmt::Debug for UpdateBuilder<R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("UpdateBuilder")
      .field("app", &self.app)
      .field("current_version", &self.current_version)
      .field("urls", &self.urls)
      .field("target", &self.target)
      .field("executable_path", &self.executable_path)
      .field("timeout", &self.timeout)
      .field("headers", &self.headers)
      .finish()
  }
}

// Create new updater instance and return an Update
impl<R: Runtime> UpdateBuilder<R> {
  pub fn new(app: AppHandle<R>) -> Self {
    UpdateBuilder {
      app,
      urls: Vec::new(),
      target: None,
      executable_path: None,
      current_version: env!("CARGO_PKG_VERSION").into(),
      should_install: None,
      timeout: None,
      headers: Default::default(),
    }
  }

  #[allow(dead_code)]
  pub fn url(mut self, url: String) -> Self {
    self.urls.push(
      percent_encoding::percent_decode(url.as_bytes())
        .decode_utf8_lossy()
        .to_string(),
    );
    self
  }

  /// Add multiple URLS at once inside a Vec for future reference
  pub fn urls(mut self, urls: &[String]) -> Self {
    let mut formatted_vec: Vec<String> = Vec::new();
    for url in urls {
      formatted_vec.push(
        percent_encoding::percent_decode(url.as_bytes())
          .decode_utf8_lossy()
          .to_string(),
      );
    }
    self.urls = formatted_vec;
    self
  }

  /// Set the current app version, used to compare against the latest available version.
  /// The `cargo_crate_version!` macro can be used to pull the version from your `Cargo.toml`
  pub fn current_version(mut self, ver: impl Into<String>) -> Self {
    self.current_version = ver.into();
    self
  }

  /// Set the target name. Represents the string that is looked up on the updater API or response JSON.
  pub fn target(mut self, target: impl Into<String>) -> Self {
    self.target.replace(target.into());
    self
  }

  /// Set the executable path
  #[allow(dead_code)]
  pub fn executable_path<A: AsRef<Path>>(mut self, executable_path: A) -> Self {
    self.executable_path = Some(PathBuf::from(executable_path.as_ref()));
    self
  }

  pub fn should_install<F: FnOnce(&str, &str) -> bool + Send + 'static>(mut self, f: F) -> Self {
    self.should_install.replace(Box::new(f));
    self
  }

  pub fn timeout(mut self, timeout: Duration) -> Self {
    self.timeout.replace(timeout);
    self
  }

  /// Add a `Header` to the request.
  pub fn header<K, V>(mut self, key: K, value: V) -> Result<Self>
  where
    HeaderName: TryFrom<K>,
    <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
    HeaderValue: TryFrom<V>,
    <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
  {
    let key: std::result::Result<HeaderName, http::Error> = key.try_into().map_err(Into::into);
    let value: std::result::Result<HeaderValue, http::Error> = value.try_into().map_err(Into::into);
    self.headers.insert(key?, value?);
    Ok(self)
  }

  pub async fn build(mut self) -> Result<Update<R>> {
    let mut remote_release: Option<RemoteRelease> = None;

    // make sure we have at least one url
    if self.urls.is_empty() {
      return Err(Error::Builder(
        "Unable to check update, `url` is required.".into(),
      ));
    };

    // If no executable path provided, we use current_exe from tauri_utils
    let executable_path = self.executable_path.unwrap_or(current_exe()?);

    let arch = get_updater_arch().ok_or(Error::UnsupportedArch)?;
    // `target` is the `{{target}}` variable we replace in the endpoint
    // `json_target` is the value we search if the updater server returns a JSON with the `platforms` object
    let (target, json_target) = if let Some(target) = self.target {
      (target.clone(), target)
    } else {
      let target = get_updater_target().ok_or(Error::UnsupportedOs)?;
      (target.to_string(), format!("{}-{}", target, arch))
    };

    // Get the extract_path from the provided executable_path
    let extract_path = extract_path_from_executable(&self.app.state::<Env>(), &executable_path);

    // Set SSL certs for linux if they aren't available.
    // We do not require to recheck in the download_and_install as we use
    // ENV variables, we can expect them to be set for the second call.
    #[cfg(target_os = "linux")]
    {
      if env::var_os("SSL_CERT_FILE").is_none() {
        env::set_var("SSL_CERT_FILE", "/etc/ssl/certs/ca-certificates.crt");
      }
      if env::var_os("SSL_CERT_DIR").is_none() {
        env::set_var("SSL_CERT_DIR", "/etc/ssl/certs");
      }
    }

    // we want JSON only
    let mut headers = self.headers;
    headers.insert("Accept", HeaderValue::from_str("application/json").unwrap());

    // Allow fallback if more than 1 urls is provided
    let mut last_error: Option<Error> = None;
    for url in &self.urls {
      // replace {{current_version}}, {{target}} and {{arch}} in the provided URL
      // this is usefull if we need to query example
      // https://releases.myapp.com/update/{{target}}/{{arch}}/{{current_version}}
      // will be transleted into ->
      // https://releases.myapp.com/update/darwin/aarch64/1.0.0
      // The main objective is if the update URL is defined via the Cargo.toml
      // the URL will be generated dynamicly
      let fixed_link = url
        .replace("{{current_version}}", &self.current_version)
        .replace("{{target}}", &target)
        .replace("{{arch}}", arch);

      let mut request = HttpRequestBuilder::new("GET", &fixed_link)?.headers(headers.clone());
      if let Some(timeout) = self.timeout {
        request = request.timeout(timeout);
      }
      let resp = ClientBuilder::new().build()?.send(request).await;

      // If we got a success, we stop the loop
      // and we set our remote_release variable
      if let Ok(res) = resp {
        let res = res.read().await?;
        // got status code 2XX
        if StatusCode::from_u16(res.status)
          .map_err(|e| Error::Builder(e.to_string()))?
          .is_success()
        {
          // if we got 204
          if StatusCode::NO_CONTENT.as_u16() == res.status {
            // return with `UpToDate` error
            // we should catch on the client
            return Err(Error::UpToDate);
          };
          // Convert the remote result to our local struct
          let built_release = RemoteRelease::from_release(&res.data, &json_target);
          // make sure all went well and the remote data is compatible
          // with what we need locally
          match built_release {
            Ok(release) => {
              last_error = None;
              remote_release = Some(release);
              break;
            }
            Err(err) => last_error = Some(err),
          }
        } // if status code is not 2XX we keep loopin' our urls
      }
    }

    // Last error is cleaned on success -- shouldn't be triggered if
    // we have a successful call
    if let Some(error) = last_error {
      return Err(error);
    }

    // Extracted remote metadata
    let final_release = remote_release.ok_or(Error::ReleaseNotFound)?;

    // did the announced version is greated than our current one?
    let should_update = if let Some(comparator) = self.should_install.take() {
      comparator(&self.current_version, &final_release.version)
    } else {
      version::is_greater(&self.current_version, &final_release.version).unwrap_or(false)
    };

    headers.remove("Accept");

    // create our new updater
    Ok(Update {
      app: self.app,
      target,
      extract_path,
      should_update,
      version: final_release.version,
      date: final_release.date,
      current_version: self.current_version.to_owned(),
      download_url: final_release.download_url,
      body: final_release.body,
      signature: final_release.signature,
      #[cfg(target_os = "windows")]
      with_elevated_task: final_release.with_elevated_task,
      timeout: self.timeout,
      headers,
    })
  }
}

pub fn builder<R: Runtime>(app: AppHandle<R>) -> UpdateBuilder<R> {
  UpdateBuilder::new(app)
}

#[derive(Debug)]
pub struct Update<R: Runtime> {
  /// Application handle.
  pub app: AppHandle<R>,
  /// Update description
  pub body: Option<String>,
  /// Should we update or not
  pub should_update: bool,
  /// Version announced
  pub version: String,
  /// Running version
  pub current_version: String,
  /// Update publish date
  pub date: String,
  /// Target
  #[allow(dead_code)]
  target: String,
  /// Extract path
  extract_path: PathBuf,
  /// Download URL announced
  download_url: String,
  /// Signature announced
  signature: String,
  #[cfg(target_os = "windows")]
  /// Optional: Windows only try to use elevated task
  /// Default to false
  with_elevated_task: bool,
  /// Request timeout
  timeout: Option<Duration>,
  /// Request headers
  headers: HeaderMap,
}

impl<R: Runtime> Clone for Update<R> {
  fn clone(&self) -> Self {
    Self {
      app: self.app.clone(),
      body: self.body.clone(),
      should_update: self.should_update,
      version: self.version.clone(),
      current_version: self.current_version.clone(),
      date: self.date.clone(),
      target: self.target.clone(),
      extract_path: self.extract_path.clone(),
      download_url: self.download_url.clone(),
      signature: self.signature.clone(),
      #[cfg(target_os = "windows")]
      with_elevated_task: self.with_elevated_task,
      timeout: self.timeout,
      headers: self.headers.clone(),
    }
  }
}

impl<R: Runtime> Update<R> {
  // Download and install our update
  // @todo(lemarier): Split into download and install (two step) but need to be thread safe
  pub(crate) async fn download_and_install<C: Fn(usize, Option<u64>), D: FnOnce()>(
    &self,
    pub_key: String,
    on_chunk: C,
    on_download_finish: D,
  ) -> Result {
    // make sure we can install the update on linux
    // We fail here because later we can add more linux support
    // actually if we use APPIMAGE, our extract path should already
    // be set with our APPIMAGE env variable, we don't need to do
    // anythin with it yet
    #[cfg(target_os = "linux")]
    if self.app.state::<Env>().appimage.is_none() {
      return Err(Error::UnsupportedLinuxPackage);
    }

    // set our headers
    let mut headers = self.headers.clone();
    headers.insert(
      "Accept",
      HeaderValue::from_str("application/octet-stream").unwrap(),
    );
    headers.insert(
      "User-Agent",
      HeaderValue::from_str("tauri/updater").unwrap(),
    );

    let client = ClientBuilder::new().build()?;
    // Create our request
    let mut req = HttpRequestBuilder::new("GET", self.download_url.as_str())?.headers(headers);
    if let Some(timeout) = self.timeout {
      req = req.timeout(timeout);
    }

    let response = client.send(req).await?;

    // make sure it's success
    if !response.status().is_success() {
      return Err(Error::Network(format!(
        "Download request failed with status: {}",
        response.status()
      )));
    }

    let content_length: Option<u64> = response
      .headers()
      .get("Content-Length")
      .and_then(|value| value.to_str().ok())
      .and_then(|value| value.parse().ok());

    let mut buffer = Vec::new();
    #[cfg(feature = "reqwest-client")]
    {
      use futures::StreamExt;
      let mut stream = response.bytes_stream();
      while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let bytes = chunk.as_ref().to_vec();
        on_chunk(bytes.len(), content_length);
        buffer.extend(bytes);
      }
    }
    #[cfg(not(feature = "reqwest-client"))]
    {
      let mut reader = response.reader();
      let mut buf = [0; 16384];
      loop {
        match reader.read(&mut buf) {
          Ok(b) => {
            if b == 0 {
              break;
            } else {
              let bytes = buf[0..b].to_vec();
              on_chunk(bytes.len(), content_length);
              buffer.extend(bytes);
            }
          }
          Err(e) => return Err(e.into()),
        }
      }
    }

    on_download_finish();

    // create memory buffer from our archive (Seek + Read)
    let mut archive_buffer = Cursor::new(buffer);

    // We need an announced signature by the server
    // if there is no signature, bail out.
    verify_signature(&mut archive_buffer, &self.signature, &pub_key)?;

    #[cfg(feature = "updater")]
    {
      // we copy the files depending of the operating system
      // we run the setup, appimage re-install or overwrite the
      // macos .app
      #[cfg(target_os = "windows")]
      copy_files_and_run(
        archive_buffer,
        &self.extract_path,
        self.with_elevated_task,
        self
          .app
          .config()
          .tauri
          .updater
          .windows
          .install_mode
          .clone()
          .msiexec_args(),
      )?;
      #[cfg(not(target_os = "windows"))]
      copy_files_and_run(archive_buffer, &self.extract_path)?;
    }
    // We are done!
    Ok(())
  }
}

// Linux (AppImage)

// ### Expected structure:
// ├── [AppName]_[version]_amd64.AppImage.tar.gz    # GZ generated by tauri-bundler
// │   └──[AppName]_[version]_amd64.AppImage        # Application AppImage
// └── ...

// We should have an AppImage already installed to be able to copy and install
// the extract_path is the current AppImage path
// tmp_dir is where our new AppImage is found
#[cfg(feature = "updater")]
#[cfg(target_os = "linux")]
fn copy_files_and_run<R: Read + Seek>(archive_buffer: R, extract_path: &Path) -> Result {
  let tmp_dir = tempfile::Builder::new()
    .prefix("tauri_current_app")
    .tempdir()?;

  let tmp_app_image = &tmp_dir.path().join("current_app.AppImage");

  // create a backup of our current app image
  Move::from_source(extract_path).to_dest(tmp_app_image)?;

  // extract the buffer to the tmp_dir
  // we extract our signed archive into our final directory without any temp file
  let mut extractor =
    Extract::from_cursor(archive_buffer, ArchiveFormat::Tar(Some(Compression::Gz)));

  extractor.with_files(|entry| {
    let path = entry.path()?;
    if path.extension() == Some(OsStr::new("AppImage")) {
      // if something went wrong during the extraction, we should restore previous app
      if let Err(err) = entry.extract(extract_path) {
        Move::from_source(tmp_app_image).to_dest(extract_path)?;
        return Err(crate::api::Error::Extract(err.to_string()));
      }
      // early finish we have everything we need here
      return Ok(true);
    }
    Ok(false)
  })?;

  Ok(())
}

// Windows

// ### Expected structure:
// ├── [AppName]_[version]_x64.msi.zip          # ZIP generated by tauri-bundler
// │   └──[AppName]_[version]_x64.msi           # Application MSI
// └── ...

// ## MSI
// Update server can provide a MSI for Windows. (Generated with tauri-bundler from *Wix*)
// To replace current version of the application. In later version we'll offer
// incremental update to push specific binaries.

// ## EXE
// Update server can provide a custom EXE (installer) who can run any task.
#[cfg(feature = "updater")]
#[cfg(target_os = "windows")]
#[allow(clippy::unnecessary_wraps)]
fn copy_files_and_run<R: Read + Seek>(
  archive_buffer: R,
  _extract_path: &Path,
  with_elevated_task: bool,
  msiexec_args: &[&str],
) -> Result {
  // FIXME: We need to create a memory buffer with the MSI and then run it.
  //        (instead of extracting the MSI to a temp path)
  //
  // The tricky part is the MSI need to be exposed and spawned so the memory allocation
  // shouldn't drop but we should be able to pass the reference so we can drop it once the installation
  // is done, otherwise we have a huge memory leak.

  let tmp_dir = tempfile::Builder::new().tempdir()?.into_path();

  // extract the buffer to the tmp_dir
  // we extract our signed archive into our final directory without any temp file
  let mut extractor = Extract::from_cursor(archive_buffer, ArchiveFormat::Zip);

  // extract the msi
  extractor.extract_into(&tmp_dir)?;

  let paths = read_dir(&tmp_dir)?;
  // This consumes the TempDir without deleting directory on the filesystem,
  // meaning that the directory will no longer be automatically deleted.

  for path in paths {
    let found_path = path?.path();
    // we support 2 type of files exe & msi for now
    // If it's an `exe` we expect an installer not a runtime.
    if found_path.extension() == Some(OsStr::new("exe")) {
      // Run the EXE
      Command::new(found_path)
        .spawn()
        .expect("installer failed to start");

      exit(0);
    } else if found_path.extension() == Some(OsStr::new("msi")) {
      if with_elevated_task {
        if let Some(bin_name) = current_exe()
          .ok()
          .and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
          .and_then(|s| s.into_string().ok())
        {
          let product_name = bin_name.replace(".exe", "");

          // Check if there is a task that enables the updater to skip the UAC prompt
          let update_task_name = format!("Update {} - Skip UAC", product_name);
          if let Ok(output) = Command::new("schtasks")
            .arg("/QUERY")
            .arg("/TN")
            .arg(update_task_name.clone())
            .output()
          {
            if output.status.success() {
              // Rename the MSI to the match file name the Skip UAC task is expecting it to be
              let temp_msi = tmp_dir.with_file_name(bin_name).with_extension("msi");
              Move::from_source(&found_path)
                .to_dest(&temp_msi)
                .expect("Unable to move update MSI");
              let exit_status = Command::new("schtasks")
                .arg("/RUN")
                .arg("/TN")
                .arg(update_task_name)
                .status()
                .expect("failed to start updater task");

              if exit_status.success() {
                // Successfully launched task that skips the UAC prompt
                exit(0);
              }
            }
            // Failed to run update task. Following UAC Path
          }
        }
      }

      // restart should be handled by WIX as we exit the process
      Command::new("msiexec.exe")
        .arg("/i")
        .arg(found_path)
        .args(msiexec_args)
        .arg("/promptrestart")
        .spawn()
        .expect("installer failed to start");

      exit(0);
    }
  }

  Ok(())
}

// MacOS
// ### Expected structure:
// ├── [AppName]_[version]_x64.app.tar.gz       # GZ generated by tauri-bundler
// │   └──[AppName].app                         # Main application
// │      └── Contents                          # Application contents...
// │          └── ...
// └── ...
#[cfg(feature = "updater")]
#[cfg(target_os = "macos")]
fn copy_files_and_run<R: Read + Seek>(archive_buffer: R, extract_path: &Path) -> Result {
  let mut extracted_files: Vec<PathBuf> = Vec::new();

  // extract the buffer to the tmp_dir
  // we extract our signed archive into our final directory without any temp file
  let mut extractor =
    Extract::from_cursor(archive_buffer, ArchiveFormat::Tar(Some(Compression::Gz)));
  // the first file in the tar.gz will always be
  // <app_name>/Contents
  let tmp_dir = tempfile::Builder::new()
    .prefix("tauri_current_app")
    .tempdir()?;

  // create backup of our current app
  Move::from_source(extract_path).to_dest(tmp_dir.path())?;

  // extract all the files
  extractor.with_files(|entry| {
    let path = entry.path()?;
    // skip the first folder (should be the app name)
    let collected_path: PathBuf = path.iter().skip(1).collect();
    let extraction_path = extract_path.join(collected_path);

    // if something went wrong during the extraction, we should restore previous app
    if let Err(err) = entry.extract(&extraction_path) {
      for file in &extracted_files {
        // delete all the files we extracted
        if file.is_dir() {
          std::fs::remove_dir(file)?;
        } else {
          std::fs::remove_file(file)?;
        }
      }
      Move::from_source(tmp_dir.path()).to_dest(extract_path)?;
      return Err(crate::api::Error::Extract(err.to_string()));
    }

    extracted_files.push(extraction_path);

    Ok(false)
  })?;

  Ok(())
}

pub(crate) fn get_updater_target() -> Option<&'static str> {
  if cfg!(target_os = "linux") {
    Some("linux")
  } else if cfg!(target_os = "macos") {
    Some("darwin")
  } else if cfg!(target_os = "windows") {
    Some("windows")
  } else {
    None
  }
}

pub(crate) fn get_updater_arch() -> Option<&'static str> {
  if cfg!(target_arch = "x86") {
    Some("i686")
  } else if cfg!(target_arch = "x86_64") {
    Some("x86_64")
  } else if cfg!(target_arch = "arm") {
    Some("armv7")
  } else if cfg!(target_arch = "aarch64") {
    Some("aarch64")
  } else {
    None
  }
}

/// Get the extract_path from the provided executable_path
#[allow(unused_variables)]
pub fn extract_path_from_executable(env: &Env, executable_path: &Path) -> PathBuf {
  // Return the path of the current executable by default
  // Example C:\Program Files\My App\
  let extract_path = executable_path
    .parent()
    .map(PathBuf::from)
    .expect("Can't determine extract path");

  // MacOS example binary is in /Applications/TestApp.app/Contents/MacOS/myApp
  // We need to get /Applications/<app>.app
  // todo(lemarier): Need a better way here
  // Maybe we could search for <*.app> to get the right path
  #[cfg(target_os = "macos")]
  if extract_path
    .display()
    .to_string()
    .contains("Contents/MacOS")
  {
    return extract_path
      .parent()
      .map(PathBuf::from)
      .expect("Unable to find the extract path")
      .parent()
      .map(PathBuf::from)
      .expect("Unable to find the extract path");
  }

  // We should use APPIMAGE exposed env variable
  // This is where our APPIMAGE should sit and should be replaced
  #[cfg(target_os = "linux")]
  if let Some(app_image_path) = &env.appimage {
    return PathBuf::from(app_image_path);
  }

  extract_path
}

// Convert base64 to string and prevent failing
fn base64_to_string(base64_string: &str) -> Result<String> {
  let decoded_string = &decode(base64_string)?;
  let result = from_utf8(decoded_string)
    .map_err(|_| Error::SignatureUtf8(base64_string.into()))?
    .to_string();
  Ok(result)
}

// Validate signature
// need to be public because its been used
// by our tests in the bundler
//
// NOTE: The buffer position is not reset.
pub fn verify_signature<R>(
  archive_reader: &mut R,
  release_signature: &str,
  pub_key: &str,
) -> Result<bool>
where
  R: Read,
{
  // we need to convert the pub key
  let pub_key_decoded = base64_to_string(pub_key)?;
  let public_key = PublicKey::decode(&pub_key_decoded)?;
  let signature_base64_decoded = base64_to_string(release_signature)?;
  let signature = Signature::decode(&signature_base64_decoded)?;

  // read all bytes until EOF in the buffer
  let mut data = Vec::new();
  archive_reader.read_to_end(&mut data)?;

  // Validate signature or bail out
  public_key.verify(&data, &signature, true)?;
  Ok(true)
}

#[cfg(test)]
mod test {
  use super::*;
  #[cfg(target_os = "macos")]
  use std::fs::File;

  macro_rules! block {
    ($e:expr) => {
      tokio_test::block_on($e)
    };
  }

  fn generate_sample_raw_json() -> String {
    r#"{
      "version": "v2.0.0",
      "notes": "Test version !",
      "pub_date": "2020-06-22T19:25:57Z",
      "platforms": {
        "darwin-aarch64": {
          "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUldUTE5QWWxkQnlZOVJZVGdpKzJmRWZ0SkRvWS9TdFpqTU9xcm1mUmJSSG5OWVlwSklrWkN1SFpWbmh4SDlBcTU3SXpjbm0xMmRjRkphbkpVeGhGcTdrdzlrWGpGVWZQSWdzPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNTkyOTE1MDU3CWZpbGU6L1VzZXJzL3J1bm5lci9ydW5uZXJzLzIuMjYzLjAvd29yay90YXVyaS90YXVyaS90YXVyaS9leGFtcGxlcy9jb21tdW5pY2F0aW9uL3NyYy10YXVyaS90YXJnZXQvZGVidWcvYnVuZGxlL29zeC9hcHAuYXBwLnRhci5negp4ZHFlUkJTVnpGUXdDdEhydTE5TGgvRlVPeVhjTnM5RHdmaGx3c0ZPWjZXWnFwVDRNWEFSbUJTZ1ZkU1IwckJGdmlwSzJPd00zZEZFN2hJOFUvL1FDZz09Cg==",
          "url": "https://github.com/tauri-apps/updater-test/releases/download/v1.0.0/app.app.tar.gz"
        },
        "darwin-x86_64": {
          "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUldUTE5QWWxkQnlZOVJZVGdpKzJmRWZ0SkRvWS9TdFpqTU9xcm1mUmJSSG5OWVlwSklrWkN1SFpWbmh4SDlBcTU3SXpjbm0xMmRjRkphbkpVeGhGcTdrdzlrWGpGVWZQSWdzPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNTkyOTE1MDU3CWZpbGU6L1VzZXJzL3J1bm5lci9ydW5uZXJzLzIuMjYzLjAvd29yay90YXVyaS90YXVyaS90YXVyaS9leGFtcGxlcy9jb21tdW5pY2F0aW9uL3NyYy10YXVyaS90YXJnZXQvZGVidWcvYnVuZGxlL29zeC9hcHAuYXBwLnRhci5negp4ZHFlUkJTVnpGUXdDdEhydTE5TGgvRlVPeVhjTnM5RHdmaGx3c0ZPWjZXWnFwVDRNWEFSbUJTZ1ZkU1IwckJGdmlwSzJPd00zZEZFN2hJOFUvL1FDZz09Cg==",
          "url": "https://github.com/tauri-apps/updater-test/releases/download/v1.0.0/app.app.tar.gz"
        },
        "linux-x86_64": {
          "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUldUTE5QWWxkQnlZOWZSM29hTFNmUEdXMHRoOC81WDFFVVFRaXdWOUdXUUdwT0NlMldqdXkyaWVieXpoUmdZeXBJaXRqSm1YVmczNXdRL1Brc0tHb1NOTzhrL1hadFcxdmdnPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNTkyOTE3MzQzCWZpbGU6L2hvbWUvcnVubmVyL3dvcmsvdGF1cmkvdGF1cmkvdGF1cmkvZXhhbXBsZXMvY29tbXVuaWNhdGlvbi9zcmMtdGF1cmkvdGFyZ2V0L2RlYnVnL2J1bmRsZS9hcHBpbWFnZS9hcHAuQXBwSW1hZ2UudGFyLmd6CmRUTUM2bWxnbEtTbUhOZGtERUtaZnpUMG5qbVo5TGhtZWE1SFNWMk5OOENaVEZHcnAvVW0zc1A2ajJEbWZUbU0yalRHT0FYYjJNVTVHOHdTQlYwQkF3PT0K",
          "url": "https://github.com/tauri-apps/updater-test/releases/download/v1.0.0/app.AppImage.tar.gz"
        },
        "windows-x86_64": {
          "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUldUTE5QWWxkQnlZOVJHMWlvTzRUSlQzTHJOMm5waWpic0p0VVI2R0hUNGxhQVMxdzBPRndlbGpXQXJJakpTN0toRURtVzBkcm15R0VaNTJuS1lZRWdzMzZsWlNKUVAzZGdJPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNTkyOTE1NTIzCWZpbGU6RDpcYVx0YXVyaVx0YXVyaVx0YXVyaVxleGFtcGxlc1xjb21tdW5pY2F0aW9uXHNyYy10YXVyaVx0YXJnZXRcZGVidWdcYXBwLng2NC5tc2kuemlwCitXa1lQc3A2MCs1KzEwZnVhOGxyZ2dGMlZqbjBaVUplWEltYUdyZ255eUF6eVF1dldWZzFObStaVEQ3QU1RS1lzcjhDVU4wWFovQ1p1QjJXbW1YZUJ3PT0K",
          "url": "https://github.com/tauri-apps/updater-test/releases/download/v1.0.0/app.x64.msi.zip"
        }
      }
    }"#.into()
  }

  fn generate_sample_platform_json(
    version: &str,
    public_signature: &str,
    download_url: &str,
  ) -> String {
    format!(
      r#"
        {{
          "name": "v{}",
          "notes": "This is the latest version! Once updated you shouldn't see this prompt.",
          "pub_date": "2020-06-25T14:14:19Z",
          "signature": "{}",
          "url": "{}"
        }}
      "#,
      version, public_signature, download_url
    )
  }

  fn generate_sample_with_elevated_task_platform_json(
    version: &str,
    public_signature: &str,
    download_url: &str,
    with_elevated_task: bool,
  ) -> String {
    format!(
      r#"
        {{
          "name": "v{}",
          "notes": "This is the latest version! Once updated you shouldn't see this prompt.",
          "pub_date": "2020-06-25T14:14:19Z",
          "signature": "{}",
          "url": "{}",
          "with_elevated_task": "{}"
        }}
      "#,
      version, public_signature, download_url, with_elevated_task
    )
  }

  #[test]
  fn simple_http_updater() {
    let _m = mockito::mock("GET", "/")
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(generate_sample_raw_json())
      .create();

    let app = crate::test::mock_app();
    let check_update = block!(builder(app.handle())
      .current_version("0.0.0")
      .url(mockito::server_url())
      .build());

    let updater = check_update.expect("Can't check update");

    assert!(updater.should_update);
  }

  #[test]
  fn simple_http_updater_raw_json() {
    let _m = mockito::mock("GET", "/")
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(generate_sample_raw_json())
      .create();

    let app = crate::test::mock_app();
    let check_update = block!(builder(app.handle())
      .current_version("0.0.0")
      .url(mockito::server_url())
      .build());

    let updater = check_update.expect("Can't check update");

    assert!(updater.should_update);
  }

  #[test]
  fn simple_http_updater_raw_json_windows_x86_64() {
    let _m = mockito::mock("GET", "/")
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(generate_sample_raw_json())
      .create();

    let app = crate::test::mock_app();
    let check_update = block!(builder(app.handle())
      .current_version("0.0.0")
      .target("windows-x86_64")
      .url(mockito::server_url())
      .build());

    let updater = check_update.expect("Can't check update");

    assert!(updater.should_update);
    assert_eq!(updater.version, "2.0.0");
    assert_eq!(updater.signature, "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUldUTE5QWWxkQnlZOVJHMWlvTzRUSlQzTHJOMm5waWpic0p0VVI2R0hUNGxhQVMxdzBPRndlbGpXQXJJakpTN0toRURtVzBkcm15R0VaNTJuS1lZRWdzMzZsWlNKUVAzZGdJPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNTkyOTE1NTIzCWZpbGU6RDpcYVx0YXVyaVx0YXVyaVx0YXVyaVxleGFtcGxlc1xjb21tdW5pY2F0aW9uXHNyYy10YXVyaVx0YXJnZXRcZGVidWdcYXBwLng2NC5tc2kuemlwCitXa1lQc3A2MCs1KzEwZnVhOGxyZ2dGMlZqbjBaVUplWEltYUdyZ255eUF6eVF1dldWZzFObStaVEQ3QU1RS1lzcjhDVU4wWFovQ1p1QjJXbW1YZUJ3PT0K");
    assert_eq!(
      updater.download_url,
      "https://github.com/tauri-apps/updater-test/releases/download/v1.0.0/app.x64.msi.zip"
    );
  }

  #[test]
  fn simple_http_updater_raw_json_uptodate() {
    let _m = mockito::mock("GET", "/")
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(generate_sample_raw_json())
      .create();

    let app = crate::test::mock_app();
    let check_update = block!(builder(app.handle())
      .current_version("10.0.0")
      .url(mockito::server_url())
      .build());

    let updater = check_update.expect("Can't check update");

    assert!(!updater.should_update);
  }

  #[test]
  fn simple_http_updater_without_version() {
    let _m = mockito::mock("GET", "/darwin-aarch64/1.0.0")
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(generate_sample_platform_json(
        "2.0.0",
        "SampleTauriKey",
        "https://tauri.studio",
      ))
      .create();

    let app = crate::test::mock_app();
    let check_update = block!(builder(app.handle())
      .current_version("1.0.0")
      .url(format!(
        "{}/darwin-aarch64/{{{{current_version}}}}",
        mockito::server_url()
      ))
      .build());

    let updater = check_update.expect("Can't check update");

    assert!(updater.should_update);
  }

  #[test]
  fn simple_http_updater_percent_decode() {
    let _m = mockito::mock("GET", "/darwin-aarch64/1.0.0")
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(generate_sample_platform_json(
        "2.0.0",
        "SampleTauriKey",
        "https://tauri.studio",
      ))
      .create();

    let app = crate::test::mock_app();
    let check_update = block!(builder(app.handle())
      .current_version("1.0.0")
      .url(
        url::Url::parse(&format!(
          "{}/darwin-aarch64/{{{{current_version}}}}",
          mockito::server_url()
        ))
        .unwrap()
        .to_string()
      )
      .build());

    let updater = check_update.expect("Can't check update");

    assert!(updater.should_update);

    let app = crate::test::mock_app();
    let check_update = block!(builder(app.handle())
      .current_version("1.0.0")
      .urls(&[url::Url::parse(&format!(
        "{}/darwin-aarch64/{{{{current_version}}}}",
        mockito::server_url()
      ))
      .unwrap()
      .to_string()])
      .build());

    let updater = check_update.expect("Can't check update");

    assert!(updater.should_update);
  }

  #[test]
  fn simple_http_updater_with_elevated_task() {
    let _m = mockito::mock("GET", "/windows-x86_64/1.0.0")
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(generate_sample_with_elevated_task_platform_json(
        "2.0.0",
        "SampleTauriKey",
        "https://tauri.studio",
        true,
      ))
      .create();

    let app = crate::test::mock_app();
    let check_update = block!(builder(app.handle())
      .current_version("1.0.0")
      .url(format!(
        "{}/windows-x86_64/{{{{current_version}}}}",
        mockito::server_url()
      ))
      .build());

    let updater = check_update.expect("Can't check update");

    assert!(updater.should_update);
  }

  #[test]
  fn http_updater_uptodate() {
    let _m = mockito::mock("GET", "/darwin-aarch64/10.0.0")
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(generate_sample_platform_json(
        "2.0.0",
        "SampleTauriKey",
        "https://tauri.studio",
      ))
      .create();

    let app = crate::test::mock_app();
    let check_update = block!(builder(app.handle())
      .current_version("10.0.0")
      .url(format!(
        "{}/darwin-aarch64/{{{{current_version}}}}",
        mockito::server_url()
      ))
      .build());

    let updater = check_update.expect("Can't check update");

    assert!(!updater.should_update);
  }

  #[test]
  fn http_updater_fallback_urls() {
    let _m = mockito::mock("GET", "/")
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(generate_sample_raw_json())
      .create();

    let app = crate::test::mock_app();
    let check_update = block!(builder(app.handle())
      .url("http://badurl.www.tld/1".into())
      .url(mockito::server_url())
      .current_version("0.0.1")
      .build());

    let updater = check_update.expect("Can't check remote update");

    assert!(updater.should_update);
  }

  #[test]
  fn http_updater_fallback_urls_withs_array() {
    let _m = mockito::mock("GET", "/")
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(generate_sample_raw_json())
      .create();

    let app = crate::test::mock_app();
    let check_update = block!(builder(app.handle())
      .urls(&["http://badurl.www.tld/1".into(), mockito::server_url(),])
      .current_version("0.0.1")
      .build());

    let updater = check_update.expect("Can't check remote update");

    assert!(updater.should_update);
  }

  #[test]
  fn http_updater_invalid_remote_data() {
    let invalid_signature = r#"{
      "version": "v0.0.3",
      "notes": "Blablaa",
      "pub_date": "2020-02-20T15:41:00Z",
      "url": "https://github.com/tauri-apps/updater-test/releases/download/v0.0.1/update3.tar.gz",
      "signature": true
    }"#;
    let invalid_version = r#"{
      "version": 5,
      "notes": "Blablaa",
      "pub_date": "2020-02-20T15:41:00Z",
      "url": "https://github.com/tauri-apps/updater-test/releases/download/v0.0.1/update3.tar.gz",
      "signature": "x"
    }"#;
    let invalid_name = r#"{
      "name": false,
      "notes": "Blablaa",
      "pub_date": "2020-02-20T15:41:00Z",
      "url": "https://github.com/tauri-apps/updater-test/releases/download/v0.0.1/update3.tar.gz",
      "signature": "x"
    }"#;
    let invalid_date = r#"{
      "version": "1.0.0",
      "notes": "Blablaa",
      "pub_date": 345645646,
      "url": "https://github.com/tauri-apps/updater-test/releases/download/v0.0.1/update3.tar.gz",
      "signature": "x"
    }"#;
    let invalid_notes = r#"{
      "version": "v0.0.3",
      "notes": ["bla", "bla"],
      "pub_date": "2020-02-20T15:41:00Z",
      "url": "https://github.com/tauri-apps/updater-test/releases/download/v0.0.1/update3.tar.gz",
      "signature": "x"
    }"#;
    let invalid_url = r#"{
      "version": "v0.0.3",
      "notes": "Blablaa",
      "pub_date": "2020-02-20T15:41:00Z",
      "url": ["https://github.com/tauri-apps/updater-test/releases/download/v0.0.1/update3.tar.gz", "https://github.com/tauri-apps/updater-test/releases/download/v0.0.1/update3.tar.gz"],
      "signature": "x"
    }"#;
    let invalid_platform_signature = r#"{
      "version": "v0.0.3",
      "notes": "Blablaa",
      "pub_date": "2020-02-20T15:41:00Z",
      "platforms": {
        "test-target": {
          "url": "https://github.com/tauri-apps/updater-test/releases/download/v0.0.1/update3.tar.gz",
          "signature": {
            "test-target": "x"
          }
        }
      }
    }"#;
    let invalid_platform_url = r#"{
      "version": "v0.0.3",
      "notes": "Blablaa",
      "pub_date": "2020-02-20T15:41:00Z",
      "platforms": {
        "test-target": {
          "url": {
            "first": "https://github.com/tauri-apps/updater-test/releases/download/v0.0.1/update3.tar.gz"
          }
          "signature": "x"
        }
      }
    }"#;

    let test_cases = [
      (
        invalid_signature,
        Box::new(|e| matches!(e, Error::InvalidResponseType("signature", "string", _)))
          as Box<dyn FnOnce(Error) -> bool>,
      ),
      (
        invalid_version,
        Box::new(|e| matches!(e, Error::InvalidResponseType("version", "string", _)))
          as Box<dyn FnOnce(Error) -> bool>,
      ),
      (
        invalid_name,
        Box::new(|e| matches!(e, Error::InvalidResponseType("name", "string", _)))
          as Box<dyn FnOnce(Error) -> bool>,
      ),
      (
        invalid_date,
        Box::new(|e| matches!(e, Error::InvalidResponseType("pub_date", "string", _)))
          as Box<dyn FnOnce(Error) -> bool>,
      ),
      (
        invalid_notes,
        Box::new(|e| matches!(e, Error::InvalidResponseType("notes", "string", _)))
          as Box<dyn FnOnce(Error) -> bool>,
      ),
      (
        invalid_url,
        Box::new(|e| matches!(e, Error::InvalidResponseType("url", "string", _)))
          as Box<dyn FnOnce(Error) -> bool>,
      ),
      (
        invalid_platform_signature,
        Box::new(|e| matches!(e, Error::InvalidResponseType("signature", "string", _)))
          as Box<dyn FnOnce(Error) -> bool>,
      ),
      (
        invalid_platform_url,
        Box::new(|e| matches!(e, Error::InvalidResponseType("url", "string", _)))
          as Box<dyn FnOnce(Error) -> bool>,
      ),
    ];

    for (response, validator) in test_cases {
      let _m = mockito::mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response)
        .create();

      let app = crate::test::mock_app();
      let check_update = block!(builder(app.handle())
        .url(mockito::server_url())
        .current_version("0.0.1")
        .target("test-target")
        .build());
      if let Err(e) = check_update {
        validator(e);
      } else {
        panic!("unexpected Ok response");
      }
    }
  }

  #[test]
  fn http_updater_missing_remote_data() {
    let missing_signature = r#"{
      "version": "v0.0.3",
      "notes": "Blablaa",
      "pub_date": "2020-02-20T15:41:00Z",
      "url": "https://github.com/tauri-apps/updater-test/releases/download/v0.0.1/update3.tar.gz"
    }"#;
    let missing_version = r#"{
      "notes": "Blablaa",
      "pub_date": "2020-02-20T15:41:00Z",
      "url": "https://github.com/tauri-apps/updater-test/releases/download/v0.0.1/update3.tar.gz",
      "signature": "x"
    }"#;
    let missing_url = r#"{
      "version": "v0.0.3",
      "notes": "Blablaa",
      "pub_date": "2020-02-20T15:41:00Z",
      "signature": "x"
    }"#;
    let missing_target = r#"{
      "version": "v0.0.3",
      "notes": "Blablaa",
      "pub_date": "2020-02-20T15:41:00Z",
      "platforms": {
        "unknown-target": {
          "url": "https://github.com/tauri-apps/updater-test/releases/download/v0.0.1/update3.tar.gz",
          "signature": "x"
        }
      }
    }"#;
    let missing_platform_signature = r#"{
      "version": "v0.0.3",
      "notes": "Blablaa",
      "pub_date": "2020-02-20T15:41:00Z",
      "platforms": {
        "test-target": {
          "url": "https://github.com/tauri-apps/updater-test/releases/download/v0.0.1/update3.tar.gz"
        }
      }
    }"#;
    let missing_platform_url = r#"{
      "version": "v0.0.3",
      "notes": "Blablaa",
      "pub_date": "2020-02-20T15:41:00Z",
      "platforms": {
        "test-target": {
          "signature": "x"
        }
      }
    }"#;

    let test_cases = [
      (missing_signature, Error::MissingResponseField("signature")),
      (
        missing_version,
        Error::MissingResponseField("version or name"),
      ),
      (missing_url, Error::MissingResponseField("url")),
      (missing_target, Error::TargetNotFound("test-target".into())),
      (
        missing_platform_signature,
        Error::MissingResponseField("signature"),
      ),
      (missing_platform_url, Error::MissingResponseField("url")),
    ];

    for (response, error) in test_cases {
      let _m = mockito::mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response)
        .create();

      let app = crate::test::mock_app();
      let check_update = block!(builder(app.handle())
        .url(mockito::server_url())
        .current_version("0.0.1")
        .target("test-target")
        .build());
      if let Err(e) = check_update {
        assert_eq!(e.to_string(), error.to_string());
      } else {
        panic!("unexpected Ok response");
      }
    }
  }

  // run complete process on mac only for now as we don't have
  // server (api) that we can use to test
  #[test]
  #[cfg(target_os = "macos")]
  fn http_updater_complete_process() {
    #[cfg(target_os = "macos")]
    let archive_file = "archive.macos.tar.gz";
    #[cfg(target_os = "linux")]
    let archive_file = "archive.linux.tar.gz";
    #[cfg(target_os = "windows")]
    let archive_file = "archive.windows.zip";

    let good_archive_url = format!("{}/{}", mockito::server_url(), archive_file);

    let mut signature_file = File::open(format!(
      "./test/updater/fixture/archives/{}.sig",
      archive_file
    ))
    .expect("Unable to open signature");
    let mut signature = String::new();
    signature_file
      .read_to_string(&mut signature)
      .expect("Unable to read signature as string");

    let mut pubkey_file = File::open("./test/updater/fixture/good_signature/update.key.pub")
      .expect("Unable to open pubkey");
    let mut pubkey = String::new();
    pubkey_file
      .read_to_string(&mut pubkey)
      .expect("Unable to read signature as string");

    // add sample file
    let _m = mockito::mock("GET", format!("/{}", archive_file).as_str())
      .with_status(200)
      .with_header("content-type", "application/octet-stream")
      .with_body_from_file(format!("./test/updater/fixture/archives/{}", archive_file))
      .create();

    // sample mock for update file
    let _m = mockito::mock("GET", "/")
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(generate_sample_platform_json(
        "2.0.1",
        signature.as_ref(),
        good_archive_url.as_ref(),
      ))
      .create();

    // Build a tmpdir so we can test our extraction inside
    // We dont want to overwrite our current executable or the directory
    // Otherwise tests are failing...
    let executable_path = current_exe().expect("Can't extract executable path");
    let parent_path = executable_path
      .parent()
      .expect("Can't find the parent path");

    let tmp_dir = tempfile::Builder::new()
      .prefix("tauri_updater_test")
      .tempdir_in(parent_path);

    assert!(tmp_dir.is_ok());
    let tmp_dir_unwrap = tmp_dir.expect("Can't find tmp_dir");
    let tmp_dir_path = tmp_dir_unwrap.path();

    #[cfg(target_os = "linux")]
    let my_executable = &tmp_dir_path.join("updater-example_0.1.0_amd64.AppImage");
    #[cfg(target_os = "macos")]
    let my_executable = &tmp_dir_path.join("my_app");
    #[cfg(target_os = "windows")]
    let my_executable = &tmp_dir_path.join("my_app.exe");

    // configure the updater
    let app = crate::test::mock_app();
    let check_update = block!(builder(app.handle())
      .url(mockito::server_url())
      // It should represent the executable path, that's why we add my_app.exe in our
      // test path -- in production you shouldn't have to provide it
      .executable_path(my_executable)
      // make sure we force an update
      .current_version("1.0.0")
      .build());

    #[cfg(target_os = "linux")]
    {
      env::set_var("APPIMAGE", my_executable);
    }

    // unwrap our results
    let updater = check_update.expect("Can't check remote update");

    // make sure we need to update
    assert!(updater.should_update);
    // make sure we can read announced version
    assert_eq!(updater.version, "2.0.1");

    // download, install and validate signature
    let install_process = block!(updater.download_and_install(pubkey, |_, _| (), || ()));
    assert!(install_process.is_ok());

    // make sure the extraction went well (it should have skipped the main app.app folder)
    // as we can't extract in /Applications directly
    #[cfg(target_os = "macos")]
    let bin_file = tmp_dir_path.join("Contents").join("MacOS").join("app");
    #[cfg(target_os = "linux")]
    // linux should extract at same place as the executable path
    let bin_file = my_executable;
    #[cfg(target_os = "windows")]
    let bin_file = tmp_dir_path.join("with").join("long").join("path.json");

    assert!(bin_file.exists());
  }
}
