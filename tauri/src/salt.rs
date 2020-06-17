use std::sync::Mutex;

use lazy_static::lazy_static;
use uuid::Uuid;

/// A salt definition.
struct Salt {
  value: String,
  one_time: bool,
}

lazy_static! {
  static ref SALTS: Mutex<Vec<Salt>> = Mutex::new(vec![]);
}

/// Generates a one time Salt and returns its string representation.
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

/// Generates a static Salt and returns its string representation.
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

/// Checks if the given Salt representation is valid.
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
