use std::sync::Mutex;
use tauri_ui::WebView;
use uuid::Uuid;

struct Salt {
  value: String,
  one_time: bool,
}

lazy_static! {
  static ref SALTS: Mutex<Vec<Salt>> = Mutex::new(vec![]);
}

pub fn generate() -> String {
  let salt = Uuid::new_v4();
  SALTS.lock().unwrap().push(Salt {
    value: salt.to_string(),
    one_time: true,
  });
  return salt.to_string();
}

pub fn generate_static() -> String {
  let salt = Uuid::new_v4();
  SALTS.lock().unwrap().push(Salt {
    value: salt.to_string(),
    one_time: false,
  });
  return salt.to_string();
}

pub fn is_valid(salt: String) -> bool {
  let mut salts = SALTS.lock().unwrap();
  match salts.iter().position(|s| s.value == salt) {
    Some(index) => {
      if salts[index].one_time {
        salts.remove(index);
      }
      true
    }
    None => false,
  }
}

pub fn validate<T: 'static>(
  webview: &mut WebView<'_, T>,
  salt: String,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      if is_valid(salt) {
        Ok("'VALID'".to_string())
      } else {
        Err("'INVALID SALT'".to_string())
      }
    },
    callback,
    error,
  );
}
