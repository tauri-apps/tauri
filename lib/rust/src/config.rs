#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub struct Config {
  #[serde(default)]
  pub width: i32,
  #[serde(default)]
  pub height: i32,
  #[serde(default)]
  pub resizable: bool,
  #[serde(default)]
  pub title: String,
}

pub fn get() -> Config {
  serde_json::from_str(include_str!("../../../config.json")).unwrap()
}
