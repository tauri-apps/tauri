#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
    Init,
    ReadAsString { path: String, callback: String, error: String },
    ReadAsBinary { path: String, callback: String, error: String },
    Write { file: String, contents: String, callback: String, error: String },
    List { path: String, callback: String, error: String },
    ListDirs { path: String, callback: String, error: String },
    SetTitle { title: String },
    Call { command: String, args: Vec<String>, callback: String, error: String }
}
