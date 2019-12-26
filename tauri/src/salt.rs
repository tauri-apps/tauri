use std::sync::Mutex;
use uuid::Uuid;
use web_view::WebView;

struct Salt {
  value: String,
  one_time: bool,
}

lazy_static! {
  static ref SALTS: Mutex<Vec<Salt>> = Mutex::new(vec![]);
}

pub fn generate() -> String {
  let salt = Uuid::new_v4();
  SALTS
    .lock()
    .expect("Failed to lock Salt mutex: generate()")
    .push(Salt {
      value: salt.to_string(),
      one_time: true,
    });
  return salt.to_string();
}

pub fn generate_static() -> String {
  let salt = Uuid::new_v4();
  SALTS
    .lock()
    .expect("Failed to lock SALT mutex: generate_static()")
    .push(Salt {
      value: salt.to_string(),
      one_time: false,
    });
  return salt.to_string();
}

pub fn is_valid(salt: String) -> bool {
  let mut salts = SALTS.lock().expect("Failed to lock Salt mutex: is_valid()");
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
  let response = if is_valid(salt) {
    Ok("'VALID'".to_string())
  } else {
    Err("'INVALID SALT'".to_string())
  };
  let callback_string = crate::api::rpc::format_callback_result(response, callback, error);
  webview
    .eval(callback_string.as_str())
    .expect("Failed to eval JS from validate()");
}
