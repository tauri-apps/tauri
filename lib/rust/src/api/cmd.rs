#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
#[cfg(feature = "api")]
pub enum Cmd {
  #[cfg(any(feature = "all-api", feature = "readAsString"))]
  ReadAsString {
    path: String,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "readAsBinary"))]
  ReadAsBinary {
    path: String,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "write"))]
  Write {
    file: String,
    contents: String,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "list"))]
  List {
    path: String,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "listDirs"))]
  ListDirs {
    path: String,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "setTitle"))]
  SetTitle { title: String },
  #[cfg(any(feature = "all-api", feature = "call"))]
  Call {
    command: String,
    args: Vec<String>,
    callback: String,
    error: String,
  },
}
