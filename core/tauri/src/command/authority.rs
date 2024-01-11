use std::collections::BTreeMap;
use std::fmt::Debug;

use serde::de::DeserializeOwned;
use state::TypeMap;

use serde_json::Value;
use tauri_utils::acl::{
  resolved::{CommandKey, Resolved, ResolvedCommand},
  ExecutionContext,
};

/// The runtime authority used to authorize IPC execution based on the Access Control List.
pub struct RuntimeAuthority {
  allowed_commands: BTreeMap<CommandKey, ResolvedCommand>,
  denied_commands: BTreeMap<CommandKey, ResolvedCommand>,
}

impl RuntimeAuthority {
  pub(crate) fn new(acl: Resolved) -> Self {
    Self {
      allowed_commands: acl.allowed_commands,
      denied_commands: acl.denied_commands,
    }
  }

  /// Checks if the given IPC execution is allowed.
  pub fn is_allowed(&self, command: &str, window: &String, context: ExecutionContext) -> bool {
    let key = CommandKey {
      name: command.into(),
      context,
    };
    if self.denied_commands.contains_key(&key) {
      false
    } else if let Some(allowed) = self.allowed_commands.get(&key) {
      allowed.windows.contains(window)
    } else {
      false
    }
  }
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct ScopeManager {
  raw: BTreeMap<String, Value>,
  cache: TypeMap![Send + Sync],
}

#[allow(dead_code)]
impl ScopeManager {
  pub fn get_typed<T: Send + Sync + DeserializeOwned + Debug + 'static>(
    &self,
    key: &str,
  ) -> Option<&T> {
    match self.cache.try_get() {
      cached @ Some(_) => cached,
      None => match self
        .raw
        .get(key)
        .and_then(|r| dbg!(serde_json::from_value::<T>(dbg!(r.clone()))).ok())
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
