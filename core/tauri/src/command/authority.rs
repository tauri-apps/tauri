use std::collections::BTreeMap;
use std::fmt::Debug;

use serde::de::DeserializeOwned;
use state::TypeMap;

use serde_json::Value;

pub struct RuntimeAuthority {
  cmds: (),
}

#[derive(Debug, Default)]
pub struct RuntimeAuthorityScope {
  raw: BTreeMap<String, Value>,
  cache: TypeMap![Send + Sync],
}

impl RuntimeAuthorityScope {
  pub fn get_typed<T: Scoped + Debug + 'static>(&self, key: &str) -> Option<&T> {
    match self.cache.try_get() {
      cached @ Some(_) => cached,
      //None => match dbg!(self.raw.get(key).and_then(Value::deserialize::<T>)) {
      None => match self
        .raw
        .get(key)
        .and_then(|r| dbg!(serde_json::from_value::<T>(dbg!(r.clone()))).ok())
        //.and_then(|r| dbg!(serde_json::from_str::<T>(&r)).ok())
      {
        None => None,
        Some(value) => {
          let _ = self.cache.set(value);
          self.cache.try_get()
        }
      },
    }
  }
}

trait Scoped: Send + Sync + DeserializeOwned {}
