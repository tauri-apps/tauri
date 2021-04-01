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
