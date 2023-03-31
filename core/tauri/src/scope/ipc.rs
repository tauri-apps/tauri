use std::sync::{Arc, Mutex};

use url::Url;

/// IPC access configuration for a remote domain.
#[derive(Clone)]
pub struct RemoteDomainAccessScope {
  domain: String,
  windows: Vec<String>,
  plugins: Vec<String>,
  enable_tauri_api: bool,
}

impl RemoteDomainAccessScope {
  /// Creates a new access scope.
  pub fn new(domain: impl Into<String>) -> Self {
    Self {
      domain: domain.into(),
      windows: Vec::new(),
      plugins: Vec::new(),
      enable_tauri_api: false,
    }
  }

  /// Adds the given window label to the list of windows that uses this scope.
  pub fn add_window(mut self, window: impl Into<String>) -> Self {
    self.windows.push(window.into());
    self
  }

  /// Adds the given plugin to the allowed plugin list.
  pub fn add_plugin(mut self, plugin: impl Into<String>) -> Self {
    self.plugins.push(plugin.into());
    self
  }

  /// Enables access to the Tauri API.
  pub fn enable_tauri_api(mut self) -> Self {
    self.enable_tauri_api = true;
    self
  }

  /// The domain of the URLs that can access this scope.
  pub fn domain(&self) -> &str {
    &self.domain
  }

  /// The list of window labels that can access this scope.
  pub fn windows(&self) -> &Vec<String> {
    &self.windows
  }

  /// The list of plugins enabled by this scope.
  pub fn plugins(&self) -> &Vec<String> {
    &self.plugins
  }

  /// Whether this scope enables Tauri API access or not.
  pub fn enables_tauri_api(&self) -> bool {
    self.enable_tauri_api
  }
}

pub(crate) struct RemoteAccessError {
  pub matches_window: bool,
  pub matches_domain: bool,
}

/// IPC scope.
#[derive(Clone)]
pub struct Scope {
  remote_access: Arc<Mutex<Vec<RemoteDomainAccessScope>>>,
}

impl Scope {
  pub(crate) fn new(config: Vec<crate::utils::config::RemoteDomainAccessScope>) -> Self {
    Self {
      remote_access: Arc::new(Mutex::new(
        config
          .into_iter()
          .map(|s| RemoteDomainAccessScope {
            domain: s.domain,
            windows: s.windows,
            plugins: s.plugins,
            enable_tauri_api: s.enable_tauri_api,
          })
          .collect(),
      )),
    }
  }

  /// Adds the given configuration for remote access.
  ///
  /// # Examples
  ///
  /// ```
  /// use tauri::{Manager, scope::ipc::RemoteDomainAccessScope};
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     app.ipc_scope().configure_remote_access(
  ///       RemoteDomainAccessScope::new("tauri.app")
  ///         .add_window("main")
  ///         .enable_tauri_api()
  ///       );
  ///     Ok(())
  ///   });
  /// ```
  pub fn configure_remote_access(&self, access: RemoteDomainAccessScope) {
    self.remote_access.lock().unwrap().push(access);
  }

  pub(crate) fn remote_access_for(
    &self,
    window_label: &String,
    url: &Url,
  ) -> Result<RemoteDomainAccessScope, RemoteAccessError> {
    let mut scope = None;
    let mut found_scope_for_window = false;
    let mut found_scope_for_domain = false;
    for s in &*self.remote_access.lock().unwrap() {
      let matches_window = s.windows.contains(window_label);
      let matches_domain = url.domain().map(|d| d == s.domain).unwrap_or_default();
      found_scope_for_window = found_scope_for_window || matches_window;
      found_scope_for_domain = found_scope_for_domain || matches_domain;
      if matches_window && matches_domain && scope.is_none() {
        scope.replace(s.clone());
      }
    }

    if let Some(s) = scope {
      Ok(s)
    } else {
      Err(RemoteAccessError {
        matches_window: found_scope_for_window,
        matches_domain: found_scope_for_domain,
      })
    }
  }
}
