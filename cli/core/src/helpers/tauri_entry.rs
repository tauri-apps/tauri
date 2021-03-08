#[derive(Default)]
pub struct TauriScript {
  global_tauri: bool,
}

impl TauriScript {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn global_tauri(mut self, global_tauri: bool) -> Self {
    self.global_tauri = global_tauri;
    self
  }

  pub fn get(self) -> String {
    let mut scripts = Vec::new();
    scripts.push(include_str!("../templates/tauri.js"));

    if self.global_tauri {
      scripts.insert(
        0,
        include_str!(concat!(env!("OUT_DIR"), "/tauri.bundle.umd.js")),
      );
    }

    scripts.join("\n\n")
  }
}
