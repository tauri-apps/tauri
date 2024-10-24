#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{de::Deserializer, Deserialize, Serialize};

use http::response::Builder;

use serde_untagged::UntaggedEnumVisitor;
use serde_with::skip_serializing_none;
use std::{
  collections::HashMap,
  fmt::{self, Display},
  path::PathBuf,
  string::String,
  vec::Vec,
};

use crate::acl::capability::Capability;

/// A Content-Security-Policy directive source list.
/// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy/Sources#sources>.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", untagged)]
pub enum CspDirectiveSources {
  /// An inline list of CSP sources. Same as [`Self::List`], but concatenated with a space separator.
  Inline(String),
  /// A list of CSP sources. The collection will be concatenated with a space separator for the CSP string.
  List(Vec<String>),
}

impl Default for CspDirectiveSources {
  fn default() -> Self {
    Self::List(Vec::new())
  }
}

impl From<CspDirectiveSources> for Vec<String> {
  fn from(sources: CspDirectiveSources) -> Self {
    match sources {
      CspDirectiveSources::Inline(source) => source.split(' ').map(|s| s.to_string()).collect(),
      CspDirectiveSources::List(l) => l,
    }
  }
}

impl CspDirectiveSources {
  /// Whether the given source is configured on this directive or not.
  pub fn contains(&self, source: &str) -> bool {
    match self {
      Self::Inline(s) => s.contains(&format!("{source} ")) || s.contains(&format!(" {source}")),
      Self::List(l) => l.contains(&source.into()),
    }
  }

  /// Appends the given source to this directive.
  pub fn push<S: AsRef<str>>(&mut self, source: S) {
    match self {
      Self::Inline(s) => {
        s.push(' ');
        s.push_str(source.as_ref());
      }
      Self::List(l) => {
        l.push(source.as_ref().to_string());
      }
    }
  }

  /// Extends this CSP directive source list with the given array of sources.
  pub fn extend(&mut self, sources: Vec<String>) {
    for s in sources {
      self.push(s);
    }
  }
}

/// A Content-Security-Policy definition.
/// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP>.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", untagged)]
pub enum Csp {
  /// The entire CSP policy in a single text string.
  Policy(String),
  /// An object mapping a directive with its sources values as a list of strings.
  DirectiveMap(HashMap<String, CspDirectiveSources>),
}

impl From<HashMap<String, CspDirectiveSources>> for Csp {
  fn from(map: HashMap<String, CspDirectiveSources>) -> Self {
    Self::DirectiveMap(map)
  }
}

impl From<Csp> for HashMap<String, CspDirectiveSources> {
  fn from(csp: Csp) -> Self {
    match csp {
      Csp::Policy(policy) => {
        let mut map = HashMap::new();
        for directive in policy.split(';') {
          let mut tokens = directive.trim().split(' ');
          if let Some(directive) = tokens.next() {
            let sources = tokens.map(|s| s.to_string()).collect::<Vec<String>>();
            map.insert(directive.to_string(), CspDirectiveSources::List(sources));
          }
        }
        map
      }
      Csp::DirectiveMap(m) => m,
    }
  }
}

impl Display for Csp {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Policy(s) => write!(f, "{s}"),
      Self::DirectiveMap(m) => {
        let len = m.len();
        let mut i = 0;
        for (directive, sources) in m {
          let sources: Vec<String> = sources.clone().into();
          write!(f, "{} {}", directive, sources.join(" "))?;
          i += 1;
          if i != len {
            write!(f, "; ")?;
          }
        }
        Ok(())
      }
    }
  }
}

/// The possible values for the `dangerous_disable_asset_csp_modification` config option.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum DisabledCspModificationKind {
  /// If `true`, disables all CSP modification.
  /// `false` is the default value and it configures Tauri to control the CSP.
  Flag(bool),
  /// Disables the given list of CSP directives modifications.
  List(Vec<String>),
}

impl DisabledCspModificationKind {
  /// Determines whether the given CSP directive can be modified or not.
  pub fn can_modify(&self, directive: &str) -> bool {
    match self {
      Self::Flag(f) => !f,
      Self::List(l) => !l.contains(&directive.into()),
    }
  }
}

impl Default for DisabledCspModificationKind {
  fn default() -> Self {
    Self::Flag(false)
  }
}

/// Protocol scope definition.
/// It is a list of glob patterns that restrict the API access from the webview.
///
/// Each pattern can start with a variable that resolves to a system base directory.
/// The variables are: `$AUDIO`, `$CACHE`, `$CONFIG`, `$DATA`, `$LOCALDATA`, `$DESKTOP`,
/// `$DOCUMENT`, `$DOWNLOAD`, `$EXE`, `$FONT`, `$HOME`, `$PICTURE`, `$PUBLIC`, `$RUNTIME`,
/// `$TEMPLATE`, `$VIDEO`, `$RESOURCE`, `$APP`, `$LOG`, `$TEMP`, `$APPCONFIG`, `$APPDATA`,
/// `$APPLOCALDATA`, `$APPCACHE`, `$APPLOG`.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum FsScope {
  /// A list of paths that are allowed by this scope.
  AllowedPaths(Vec<PathBuf>),
  /// A complete scope configuration.
  #[serde(rename_all = "camelCase")]
  Scope {
    /// A list of paths that are allowed by this scope.
    #[serde(default)]
    allow: Vec<PathBuf>,
    /// A list of paths that are not allowed by this scope.
    /// This gets precedence over the [`Self::Scope::allow`] list.
    #[serde(default)]
    deny: Vec<PathBuf>,
    /// Whether or not paths that contain components that start with a `.`
    /// will require that `.` appears literally in the pattern; `*`, `?`, `**`,
    /// or `[...]` will not match. This is useful because such files are
    /// conventionally considered hidden on Unix systems and it might be
    /// desirable to skip them when listing files.
    ///
    /// Defaults to `true` on Unix systems and `false` on Windows
    // dotfiles are not supposed to be exposed by default on unix
    #[serde(alias = "require-literal-leading-dot")]
    require_literal_leading_dot: Option<bool>,
  },
}

impl Default for FsScope {
  fn default() -> Self {
    Self::AllowedPaths(Vec::new())
  }
}

impl FsScope {
  /// The list of allowed paths.
  pub fn allowed_paths(&self) -> &Vec<PathBuf> {
    match self {
      Self::AllowedPaths(p) => p,
      Self::Scope { allow, .. } => allow,
    }
  }

  /// The list of forbidden paths.
  pub fn forbidden_paths(&self) -> Option<&Vec<PathBuf>> {
    match self {
      Self::AllowedPaths(_) => None,
      Self::Scope { deny, .. } => Some(deny),
    }
  }
}

/// Config for the asset custom protocol.
///
/// See more: <https://v2.tauri.app/reference/config/#assetprotocolconfig>
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssetProtocolConfig {
  /// The access scope for the asset protocol.
  #[serde(default)]
  pub scope: FsScope,
  /// Enables the asset protocol.
  #[serde(default)]
  pub enable: bool,
}

/// A capability entry which can be either an inlined capability or a reference to a capability defined on its own file.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum CapabilityEntry {
  /// An inlined capability.
  Inlined(Capability),
  /// Reference to a capability identifier.
  Reference(String),
}

impl<'de> Deserialize<'de> for CapabilityEntry {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    UntaggedEnumVisitor::new()
      .string(|string| Ok(Self::Reference(string.to_owned())))
      .map(|map| map.deserialize::<Capability>().map(Self::Inlined))
      .deserialize(deserializer)
  }
}

/// The application pattern.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "use", content = "options")]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum PatternKind {
  /// Brownfield pattern.
  Brownfield,
  /// Isolation pattern. Recommended for security purposes.
  Isolation {
    /// The dir containing the index.html file that contains the secure isolation application.
    dir: PathBuf,
  },
}

impl Default for PatternKind {
  fn default() -> Self {
    Self::Brownfield
  }
}

/// definition of a header source
///
/// The header value to a header name
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", untagged)]
pub enum HeaderSource {
  /// string version of the header Value
  Inline(String),
  /// list version of the header value. Item are joined by "," for the real header value
  List(Vec<String>),
  /// (Rust struct | Json | JavaScript Object) equivalent of the header value. Items are composed from: key + space + value. Item are then joined by ";" for the real header value
  Map(HashMap<String, String>),
}

impl Display for HeaderSource {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Inline(s) => write!(f, "{s}"),
      Self::List(l) => write!(f, "{}", l.join(", ")),
      Self::Map(m) => {
        let len = m.len();
        let mut i = 0;
        for (key, value) in m {
          write!(f, "{} {}", key, value)?;
          i += 1;
          if i != len {
            write!(f, "; ")?;
          }
        }
        Ok(())
      }
    }
  }
}

impl From<HeaderSource> for Vec<String> {
  fn from(val: HeaderSource) -> Self {
    match val {
      HeaderSource::Inline(s) => {
        let mut out: Vec<String> = Vec::new();
        let mut separator = ',';
        if s.contains(';') {
          separator = ';';
        }
        for item in s.split(separator) {
          out.push(String::from(item));
        }
        out
      }
      HeaderSource::List(l) => l,
      HeaderSource::Map(m) => {
        let mut out: Vec<String> = Vec::new();
        for (key, value) in m.iter() {
          let mut item = String::new();
          item.push_str(key);
          item.push(' ');
          item.push_str(value);
          out.push(item);
        }
        out
      }
    }
  }
}

impl From<HeaderSource> for HashMap<String, String> {
  fn from(val: HeaderSource) -> Self {
    match val {
      HeaderSource::Inline(s) => {
        let mut out: HashMap<String, String> = HashMap::new();
        for item in s.split(';') {
          let index = item.find(' ');
          if index.is_some() {
            let (key, value) = item.split_at(index.unwrap());
            out.insert(String::from(key), String::from(value));
          }
        }
        out
      }
      HeaderSource::List(l) => {
        let mut out: HashMap<String, String> = HashMap::new();
        for item in l.iter() {
          let index = item.find(' ');
          if index.is_some() {
            let (key, value) = item.split_at(index.unwrap());
            out.insert(String::from(key), String::from(value));
          }
        }
        out
      }
      HeaderSource::Map(m) => m,
    }
  }
}

impl From<String> for HeaderSource {
  fn from(value: String) -> Self {
    Self::Inline(value)
  }
}

impl From<Vec<String>> for HeaderSource {
  fn from(value: Vec<String>) -> Self {
    Self::List(value)
  }
}

impl From<HashMap<String, String>> for HeaderSource {
  fn from(value: HashMap<String, String>) -> Self {
    Self::Map(value)
  }
}

/// A trait which implements on the [`Builder`] of the http create
///
/// Must add headers defined in the tauri configuration file to http responses
pub trait HeaderAddition {
  /// adds all headers defined on the config file, given the current HeaderConfig
  fn add_configured_headers(self, headers: Option<&HeaderConfig>) -> Builder;
}

impl HeaderAddition for Builder {
  /// Add the headers defined in the tauri configuration file to http responses
  ///
  /// this is a utility function, which is used in the same way as the `.header(..)` of the rust http library
  fn add_configured_headers(mut self, headers: Option<&HeaderConfig>) -> Builder {
    self = match headers {
      Some(headers) => {
        // Add the header Access-Control-Allow-Credentials, if we find a value for it
        self = match &headers.access_control_allow_credentials {
          Some(value) => self.header("Access-Control-Allow-Credentials", value.to_string()),
          None => self,
        };

        // Add the header Access-Control-Allow-Headers, if we find a value for it
        self = match &headers.access_control_allow_headers {
          Some(value) => self.header("Access-Control-Allow-Headers", value.to_string()),
          None => self,
        };

        // Add the header Access-Control-Allow-Methods, if we find a value for it
        self = match &headers.access_control_allow_methods {
          Some(value) => self.header("Access-Control-Allow-Methods", value.to_string()),
          None => self,
        };

        // Add the header Access-Control-Expose-Headers, if we find a value for it
        self = match &headers.access_control_expose_headers {
          Some(value) => self.header("Access-Control-Expose-Headers", value.to_string()),
          None => self,
        };

        // Add the header Access-Control-Max-Age, if we find a value for it
        self = match &headers.access_control_max_age {
          Some(value) => self.header("Access-Control-Max-Age", value.to_string()),
          None => self,
        };

        // Add the header Cross-Origin-Embedder-Policy, if we find a value for it
        self = match &headers.cross_origin_embedder_policy {
          Some(value) => self.header("Cross-Origin-Embedder-Policy", value.to_string()),
          None => self,
        };

        // Add the header Cross-Origin-Opener-Policy, if we find a value for it
        self = match &headers.cross_origin_opener_policy {
          Some(value) => self.header("Cross-Origin-Opener-Policy", value.to_string()),
          None => self,
        };

        // Add the header Cross-Origin-Resource-Policy, if we find a value for it
        self = match &headers.cross_origin_resource_policy {
          Some(value) => self.header("Cross-Origin-Resource-Policy", value.to_string()),
          None => self,
        };

        // Add the header Permission-Policy, if we find a value for it
        self = match &headers.permissions_policy {
          Some(value) => self.header("Permission-Policy", value.to_string()),
          None => self,
        };

        // Add the header Timing-Allow-Origin, if we find a value for it
        self = match &headers.timing_allow_origin {
          Some(value) => self.header("Timing-Allow-Origin", value.to_string()),
          None => self,
        };

        // Add the header X-Content-Type-Options, if we find a value for it
        self = match &headers.x_content_type_options {
          Some(value) => self.header("X-Content-Type-Options", value.to_string()),
          None => self,
        };

        // Add the header Tauri-Custom-Header, if we find a value for it
        self = match &headers.tauri_custom_header {
          // Keep in mind to correctly set the Access-Control-Expose-Headers
          Some(value) => self.header("Tauri-Custom-Header", value.to_string()),
          None => self,
        };
        self
      }
      None => self,
    };
    self
  }
}

/// ## Header Config
/// A struct, where the keys are some specific http header names.
/// If the values to those keys are defined, then they will be send as part of a response message.
/// This does not include error messages and ipc messages
///
/// ## Example configuration
/// ```javascript
/// {
///  //..
///   app:{
///     //..
///     security: {
///       headers: {
///         "Cross-Origin-Opener-Policy": "same-origin",
///         "Cross-Origin-Embedder-Policy": "require-corp",
///         "Timing-Allow-Origin": [
///           "https://developer.mozilla.org",
///           "https://example.com",
///         ],
///         "Access-Control-Expose-Headers": "Tauri-Custom-Header",
///         "Tauri-Custom-Header": {
///           "key1": "'value1' 'value2'",
///           "key2": "'value3'"
///         }
///       },
///       csp: "default-src 'self'; connect-src ipc: http://ipc.localhost",
///     }
///     //..
///   }
///  //..
/// }
/// ```
/// In this example `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` are set to allow for the use of [`SharedArrayBuffer`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer).
/// The result is, that those headers are then set on every response sent via the `get_response` function in crates/tauri/src/protocol/tauri.rs.
/// The Content-Security-Policy header is defined separately, because it is also handled separately.
///
/// For the helloworld example, this config translates into those response headers:
/// ```http
/// access-control-allow-origin:  http://tauri.localhost
/// access-control-expose-headers: Tauri-Custom-Header
/// content-security-policy: default-src 'self'; connect-src ipc: http://ipc.localhost; script-src 'self' 'sha256-Wjjrs6qinmnr+tOry8x8PPwI77eGpUFR3EEGZktjJNs='
/// content-type: text/html
/// cross-origin-embedder-policy: require-corp
/// cross-origin-opener-policy: same-origin
/// tauri-custom-header: key1 'value1' 'value2'; key2 'value3'
/// timing-allow-origin: https://developer.mozilla.org, https://example.com
/// ```
/// Since the resulting header values are always 'string-like'. So depending on the what data type the HeaderSource is, they need to be converted.
///  - `String`(JS/Rust): stay the same for the resulting header value
///  - `Array`(JS)/`Vec\<String\>`(Rust): Item are joined by ", " for the resulting header value
///  - `Object`(JS)/ `Hashmap\<String,String\>`(Rust): Items are composed from: key + space + value. Item are then joined by "; " for the resulting header value
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct HeaderConfig {
  /// The Access-Control-Allow-Credentials response header tells browsers whether the
  /// server allows cross-origin HTTP requests to include credentials.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Credentials>
  #[serde(rename = "Access-Control-Allow-Credentials")]
  pub access_control_allow_credentials: Option<HeaderSource>,
  /// The Access-Control-Allow-Headers response header is used in response
  /// to a preflight request which includes the Access-Control-Request-Headers
  /// to indicate which HTTP headers can be used during the actual request.
  ///
  /// This header is required if the request has an Access-Control-Request-Headers header.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Headers>
  #[serde(rename = "Access-Control-Allow-Headers")]
  pub access_control_allow_headers: Option<HeaderSource>,
  /// The Access-Control-Allow-Methods response header specifies one or more methods
  /// allowed when accessing a resource in response to a preflight request.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Methods>
  #[serde(rename = "Access-Control-Allow-Methods")]
  pub access_control_allow_methods: Option<HeaderSource>,
  /// The Access-Control-Expose-Headers response header allows a server to indicate
  /// which response headers should be made available to scripts running in the browser,
  /// in response to a cross-origin request.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Expose-Headers>
  #[serde(rename = "Access-Control-Expose-Headers")]
  pub access_control_expose_headers: Option<HeaderSource>,
  /// The Access-Control-Max-Age response header indicates how long the results of a
  /// preflight request (that is the information contained in the
  /// Access-Control-Allow-Methods and Access-Control-Allow-Headers headers) can
  /// be cached.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Max-Age>
  #[serde(rename = "Access-Control-Max-Age")]
  pub access_control_max_age: Option<HeaderSource>,
  /// The HTTP Cross-Origin-Embedder-Policy (COEP) response header configures embedding
  /// cross-origin resources into the document.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Embedder-Policy>
  #[serde(rename = "Cross-Origin-Embedder-Policy")]
  pub cross_origin_embedder_policy: Option<HeaderSource>,
  /// The HTTP Cross-Origin-Opener-Policy (COOP) response header allows you to ensure a
  /// top-level document does not share a browsing context group with cross-origin documents.
  /// COOP will process-isolate your document and potential attackers can't access your global
  /// object if they were to open it in a popup, preventing a set of cross-origin attacks dubbed XS-Leaks.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Opener-Policy>
  #[serde(rename = "Cross-Origin-Opener-Policy")]
  pub cross_origin_opener_policy: Option<HeaderSource>,
  /// The HTTP Cross-Origin-Resource-Policy response header conveys a desire that the
  /// browser blocks no-cors cross-origin/cross-site requests to the given resource.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Resource-Policy>
  #[serde(rename = "Cross-Origin-Resource-Policy")]
  pub cross_origin_resource_policy: Option<HeaderSource>,
  /// The HTTP Permissions-Policy header provides a mechanism to allow and deny the
  /// use of browser features in a document or within any \<iframe\> elements in the document.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Permissions-Policy>
  #[serde(rename = "Permissions-Policy")]
  pub permissions_policy: Option<HeaderSource>,
  /// The Timing-Allow-Origin response header specifies origins that are allowed to see values
  /// of attributes retrieved via features of the Resource Timing API, which would otherwise be
  /// reported as zero due to cross-origin restrictions.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Timing-Allow-Origin>
  #[serde(rename = "Timing-Allow-Origin")]
  pub timing_allow_origin: Option<HeaderSource>,
  /// The X-Content-Type-Options response HTTP header is a marker used by the server to indicate
  /// that the MIME types advertised in the Content-Type headers should be followed and not be
  /// changed. The header allows you to avoid MIME type sniffing by saying that the MIME types
  /// are deliberately configured.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Content-Type-Options>
  #[serde(rename = "X-Content-Type-Options")]
  pub x_content_type_options: Option<HeaderSource>,
  /// A custom header field Tauri-Custom-Header, don't use it.
  /// Remember to set Access-Control-Expose-Headers accordingly
  ///
  /// **NOT INTENDED FOR PRODUCTION USE**
  #[serde(rename = "Tauri-Custom-Header")]
  pub tauri_custom_header: Option<HeaderSource>,
}

impl HeaderConfig {
  /// creates a new header config
  pub fn new() -> Self {
    HeaderConfig {
      access_control_allow_credentials: None,
      access_control_allow_methods: None,
      access_control_allow_headers: None,
      access_control_expose_headers: None,
      access_control_max_age: None,
      cross_origin_embedder_policy: None,
      cross_origin_opener_policy: None,
      cross_origin_resource_policy: None,
      permissions_policy: None,
      timing_allow_origin: None,
      x_content_type_options: None,
      tauri_custom_header: None,
    }
  }
}

impl Display for HeaderConfig {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        write!(f,"Access-Control-Allow-Credentials: ")?;
        match &self.access_control_allow_credentials {
          Some(value) => writeln!(f, "{}", value)?,
          None => writeln!(f, "null")?,
        };

        write!(f,"Access-Control-Allow-Headers: ")?;
        match  &self.access_control_allow_headers {
          Some(value) => writeln!(f, "{}", value)?,
          None =>  writeln!(f, "null")?,
        };

        write!(f,"Access-Control-Allow-Methods: ")?;
        match  &self.access_control_allow_methods {
          Some(value) => writeln!(f,"{}", value)?,
          None => writeln!(f, "null")?,
        };

        write!(f,"Access-Control-Expose-Headers: ")?;
       match  &self.access_control_expose_headers {
          Some(value) => writeln!(f,"{}", value)?,
          None => writeln!(f, "null")?,
        };

        write!(f,"Access-Control-Max-Age: ")?;
       match  &self.access_control_max_age {
          Some(value) => writeln!(f,"{}", value)?,
          None => writeln!(f, "null")?,
        };

        write!(f,"Cross-Origin-Embedder-Policy: ")?;
        match  &self.cross_origin_embedder_policy {
          Some(value) => writeln!(f,"{}", value)?,
          None => writeln!(f,"null")?,
        };

        write!(f,"Cross-Origin-Opener-Policy: ")?;
        match  &self.cross_origin_opener_policy {
          Some(value) => writeln!(f,"{}", value)?,
          None => writeln!(f, "null")?,
        };

        write!(f,"Cross-Origin-Resource-Policy: ")?;
        match  &self.cross_origin_resource_policy {
          Some(value) => writeln!(f,"{}", value)?,
          None => writeln!(f, "null")?,
        };

        write!(f,"Permission-Policy: ")?;
        match  &self.permissions_policy {
          Some(value) => writeln!(f,"{}", value)?,
          None => writeln!(f, "null")?,
        };

        write!(f,"Timing-Allow-Origin: ")?;
        match  &self.timing_allow_origin {
          Some(value) => writeln!(f,"{}", value)?,
          None => writeln!(f, "null")?,
        };

        write!(f,"X-Content-Type-Options: ")?;
        match  &self.x_content_type_options {
          Some(value) => writeln!(f,"{}", value)?,
          None => writeln!(f, "null")?,
        };

        write!(f,"Tauri-Custom-Header: ")?;
        match  &self.tauri_custom_header {
          // also allow the X-Custom Header to be exposed
          Some(value) => writeln!(f,"{}", value)?,
          None => writeln!(f, "null")?,
        };
        Ok(())
  }
}


/// Security configuration.
///
/// See more: <https://v2.tauri.app/reference/config/#securityconfig>
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityConfig {
  /// The Content Security Policy that will be injected on all HTML files on the built application.
  /// If [`dev_csp`](#SecurityConfig.devCsp) is not specified, this value is also injected on dev.
  ///
  /// This is a really important part of the configuration since it helps you ensure your WebView is secured.
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP>.
  pub csp: Option<Csp>,
  /// The Content Security Policy that will be injected on all HTML files on development.
  ///
  /// This is a really important part of the configuration since it helps you ensure your WebView is secured.
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP>.
  #[serde(alias = "dev-csp")]
  pub dev_csp: Option<Csp>,
  /// Freeze the `Object.prototype` when using the custom protocol.
  #[serde(default, alias = "freeze-prototype")]
  pub freeze_prototype: bool,
  /// Disables the Tauri-injected CSP sources.
  ///
  /// At compile time, Tauri parses all the frontend assets and changes the Content-Security-Policy
  /// to only allow loading of your own scripts and styles by injecting nonce and hash sources.
  /// This stricts your CSP, which may introduce issues when using along with other flexing sources.
  ///
  /// This configuration option allows both a boolean and a list of strings as value.
  /// A boolean instructs Tauri to disable the injection for all CSP injections,
  /// and a list of strings indicates the CSP directives that Tauri cannot inject.
  ///
  /// **WARNING:** Only disable this if you know what you are doing and have properly configured the CSP.
  /// Your application might be vulnerable to XSS attacks without this Tauri protection.
  #[serde(default, alias = "dangerous-disable-asset-csp-modification")]
  pub dangerous_disable_asset_csp_modification: DisabledCspModificationKind,
  /// Custom protocol config.
  #[serde(default, alias = "asset-protocol")]
  pub asset_protocol: AssetProtocolConfig,
  /// The pattern to use.
  #[serde(default)]
  pub pattern: PatternKind,
  /// List of capabilities that are enabled on the application.
  ///
  /// If the list is empty, all capabilities are included.
  #[serde(default)]
  pub capabilities: Vec<CapabilityEntry>,
  /// The headers, which are added to every http response from tauri to the web view
  /// This doesn't include IPC Messages and error responses
  #[serde(default)]
  pub headers: Option<HeaderConfig>,
}
