const COMMANDS: &[&str] = &[];

fn main() {
  tauri_plugin::Builder::new(COMMANDS).build()
}
