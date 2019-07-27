use proton_ui::WebView;
use std::sync::Mutex;
use uuid::Uuid;

lazy_static! {
  static ref SALTS: Mutex<Vec<String>> = Mutex::new(vec![]);
}

pub fn generate() -> String {
  let salt = Uuid::new_v4();
  SALTS.lock().unwrap().push(salt.to_string());
  return salt.to_string();
}

pub fn validate<T: 'static>(
  webview: &mut WebView<'_, T>,
  salt: String,
  callback: String,
  error: String,
) {
  crate::run_async(
    webview,
    move || {
      if SALTS.lock().unwrap().contains(&salt.to_string()) {
        Ok("'VALID'".to_string())
      } else {
        Err("'INVALID SALT'".to_string())
      }
    },
    callback,
    error,
  );
}
