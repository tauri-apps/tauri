use std::sync::Mutex;

use lazy_static::lazy_static;
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
  SALTS
    .lock()
    .expect("Failed to lock Salt mutex: generate()")
    .push(Salt {
      value: salt.to_string(),
      one_time: true,
    });
  salt.to_string()
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
  salt.to_string()
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
