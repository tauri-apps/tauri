// spawn an updater process.
#[cfg(feature = "updater")]
#[allow(dead_code)]
pub(super) fn spawn_updater() {
  std::thread::spawn(|| {
    tauri_api::command::spawn_relative_command(
      "updater".to_string(),
      Vec::new(),
      std::process::Stdio::inherit(),
    )
    .expect("Unable to spawn relative command");
  });
}

#[cfg(test)]
mod test {
  use crate::{generate_context, AsContext};

  #[test]
  fn check_get_url() {
    let context = generate_context!("test/fixture/src-tauri/tauri.conf.json");
    let context = AsContext::new(context);
    let res = super::get_url(&context);
    #[cfg(custom_protocol)]
    assert!(res == "tauri://studio.tauri.example");

    #[cfg(dev)]
    {
      let config = &context.config;
      assert_eq!(res, config.build.dev_path);
    }
  }
}
