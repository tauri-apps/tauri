// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  any::{Any, TypeId},
  collections::HashMap,
  hash::BuildHasherDefault,
  pin::Pin,
  sync::Mutex,
};

use crate::{
  ipc::{CommandArg, CommandItem, InvokeError},
  Runtime,
};

/// A guard for a state value.
///
/// See [`Manager::manage`](`crate::Manager::manage`) for usage examples.
pub struct State<'r, T: Send + Sync + 'static>(&'r T);

impl<'r, T: Send + Sync + 'static> State<'r, T> {
  /// Retrieve a borrow to the underlying value with a lifetime of `'r`.
  /// Using this method is typically unnecessary as `State` implements
  /// [`std::ops::Deref`] with a [`std::ops::Deref::Target`] of `T`.
  #[inline(always)]
  pub fn inner(&self) -> &'r T {
    self.0
  }
}

impl<T: Send + Sync + 'static> std::ops::Deref for State<'_, T> {
  type Target = T;

  #[inline(always)]
  fn deref(&self) -> &T {
    self.0
  }
}

impl<T: Send + Sync + 'static> Clone for State<'_, T> {
  fn clone(&self) -> Self {
    State(self.0)
  }
}

impl<T: Send + Sync + 'static + PartialEq> PartialEq for State<'_, T> {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl<T: Send + Sync + std::fmt::Debug> std::fmt::Debug for State<'_, T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("State").field(&self.0).finish()
  }
}

impl<'r, 'de: 'r, T: Send + Sync + 'static, R: Runtime> CommandArg<'de, R> for State<'r, T> {
  /// Grabs the [`State`] from the [`CommandItem`]. This will never fail.
  fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
    command.message.state_ref().try_get().ok_or_else(|| {
      InvokeError::from_anyhow(anyhow::anyhow!(
        "state not managed for field `{}` on command `{}`. You must call `.manage()` before using this command",
        command.key, command.name
      ))
    })
  }
}

// Taken from: https://github.com/SergioBenitez/state/blob/556c1b94db8ce8427a0e72de7983ab5a9af4cc41/src/ident_hash.rs
// This is a _super_ stupid hash. It just uses its input as the hash value. This
// hash is meant to be used _only_ for "prehashed" values. In particular, we use
// this so that hashing a TypeId is essentially a noop. This is because TypeIds
// are already unique integers.
#[derive(Default)]
struct IdentHash(u64);

impl std::hash::Hasher for IdentHash {
  fn finish(&self) -> u64 {
    self.0
  }

  fn write(&mut self, bytes: &[u8]) {
    for byte in bytes {
      self.write_u8(*byte);
    }
  }

  fn write_u8(&mut self, i: u8) {
    self.0 = (self.0 << 8) | (i as u64);
  }

  fn write_u64(&mut self, i: u64) {
    self.0 = i;
  }
}

/// Safety:
/// - The `key` must equal to `(*value).type_id()`, see the safety doc in methods of [StateManager] for details.
/// - Once you insert a value, you can't remove/mutated/move it anymore, see [StateManager::try_get] for details.
type TypeIdMap = HashMap<TypeId, Pin<Box<dyn Any + Sync + Send>>, BuildHasherDefault<IdentHash>>;

/// The Tauri state manager.
#[derive(Debug)]
pub struct StateManager {
  map: Mutex<TypeIdMap>,
}

impl StateManager {
  pub(crate) fn new() -> Self {
    Self {
      map: Default::default(),
    }
  }

  pub(crate) fn set<T: Send + Sync + 'static>(&self, state: T) -> bool {
    let mut map = self.map.lock().unwrap();
    let type_id = TypeId::of::<T>();
    let already_set = map.contains_key(&type_id);
    if !already_set {
      let ptr = Box::new(state) as Box<dyn Any + Sync + Send>;
      let pinned_ptr = Box::into_pin(ptr);
      map.insert(
        type_id,
        // SAFETY: keep the type of the key is the same as the type of the value，
        // see [try_get] methods for details.
        pinned_ptr,
      );
    }
    !already_set
  }

  /// SAFETY: Calling this method will move the `value`,
  /// which will cause references obtained through [Self::try_get] to dangle.
  pub(crate) unsafe fn unmanage<T: Send + Sync + 'static>(&self) -> Option<T> {
    let mut map = self.map.lock().unwrap();
    let type_id = TypeId::of::<T>();
    let pinned_ptr = map.remove(&type_id)?;
    // SAFETY: The caller decides to break the immovability/safety here, then OK, just let it go.
    let ptr = unsafe { Pin::into_inner_unchecked(pinned_ptr) };
    let value = unsafe {
      ptr
        .downcast::<T>()
        // SAFETY: the type of the key is the same as the type of the value
        .unwrap_unchecked()
    };
    Some(*value)
  }

  /// Gets the state associated with the specified type.
  pub fn get<T: Send + Sync + 'static>(&self) -> State<'_, T> {
    self
      .try_get()
      .unwrap_or_else(|| panic!("state not found for type {}", std::any::type_name::<T>()))
  }

  /// Gets the state associated with the specified type.
  pub fn try_get<T: Send + Sync + 'static>(&self) -> Option<State<'_, T>> {
    let map = self.map.lock().unwrap();
    let type_id = TypeId::of::<T>();
    let ptr = map.get(&type_id)?;
    let value = unsafe {
      ptr
        .downcast_ref::<T>()
        // SAFETY: the type of the key is the same as the type of the value
        .unwrap_unchecked()
    };
    // SAFETY: We ensure the lifetime of `value` is the same as [StateManager] and `value` will not be mutated/moved.
    let v_ref = unsafe { &*(value as *const T) };
    Some(State(v_ref))
  }
}

// Ported from https://github.com/SergioBenitez/state/blob/556c1b94db8ce8427a0e72de7983ab5a9af4cc41/tests/main.rs
#[cfg(test)]
mod tests {
  use super::StateManager;

  use std::sync::{Arc, RwLock};
  use std::thread;

  // Tiny structures to test that dropping works as expected.
  struct DroppingStruct(Arc<RwLock<bool>>);
  struct DroppingStructWrap(#[allow(dead_code)] DroppingStruct);

  impl Drop for DroppingStruct {
    fn drop(&mut self) {
      *self.0.write().unwrap() = true;
    }
  }

  #[test]
  #[should_panic(expected = "state not found for type core::option::Option<alloc::string::String>")]
  fn get_panics() {
    let state = StateManager::new();
    state.get::<Option<String>>();
  }

  #[test]
  fn simple_set_get() {
    let state = StateManager::new();
    assert!(state.set(1u32));
    assert_eq!(*state.get::<u32>(), 1);
  }

  #[test]
  fn simple_set_get_unmanage() {
    let state = StateManager::new();
    assert!(state.set(1u32));
    assert_eq!(*state.get::<u32>(), 1);
    // safety: the reference returned by `try_get` is already dropped.
    assert!(unsafe { state.unmanage::<u32>() }.is_some());
    assert!(unsafe { state.unmanage::<u32>() }.is_none());
    assert_eq!(state.try_get::<u32>(), None);
    assert!(state.set(2u32));
    assert_eq!(*state.get::<u32>(), 2);
  }

  #[test]
  fn dst_set_get() {
    let state = StateManager::new();
    assert!(state.set::<[u32; 4]>([1, 2, 3, 4u32]));
    assert_eq!(*state.get::<[u32; 4]>(), [1, 2, 3, 4]);
  }

  #[test]
  fn set_get_remote() {
    let state = Arc::new(StateManager::new());
    let sate_ = Arc::clone(&state);
    thread::spawn(move || {
      sate_.set(10isize);
    })
    .join()
    .unwrap();

    assert_eq!(*state.get::<isize>(), 10);
  }

  #[test]
  fn two_put_get() {
    let state = StateManager::new();
    assert!(state.set("Hello, world!".to_string()));

    let s_old = state.get::<String>();
    assert_eq!(*s_old, "Hello, world!");

    assert!(!state.set::<String>("Bye bye!".into()));
    assert_eq!(*state.get::<String>(), "Hello, world!");
    assert_eq!(state.get::<String>(), s_old);
  }

  #[test]
  fn many_puts_only_one_succeeds() {
    let state = Arc::new(StateManager::new());
    let mut threads = vec![];
    for _ in 0..1000 {
      let state_ = Arc::clone(&state);
      threads.push(thread::spawn(move || state_.set(10i64)))
    }

    let results: Vec<bool> = threads.into_iter().map(|t| t.join().unwrap()).collect();
    assert_eq!(results.into_iter().filter(|&b| b).count(), 1);
    assert_eq!(*state.get::<i64>(), 10);
  }

  // Ensure setting when already set doesn't cause a drop.
  #[test]
  fn test_no_drop_on_set() {
    let state = StateManager::new();
    let drop_flag = Arc::new(RwLock::new(false));
    let dropping_struct = DroppingStruct(drop_flag.clone());

    let _drop_flag_ignore = Arc::new(RwLock::new(false));
    let _dropping_struct_ignore = DroppingStruct(_drop_flag_ignore);

    state.set::<DroppingStruct>(dropping_struct);
    assert!(!state.set::<DroppingStruct>(_dropping_struct_ignore));
    assert!(!*drop_flag.read().unwrap());
  }

  // Ensure dropping a type_map drops its contents.
  #[test]
  fn drop_inners_on_drop() {
    let drop_flag_a = Arc::new(RwLock::new(false));
    let dropping_struct_a = DroppingStruct(drop_flag_a.clone());

    let drop_flag_b = Arc::new(RwLock::new(false));
    let dropping_struct_b = DroppingStructWrap(DroppingStruct(drop_flag_b.clone()));

    {
      let state = StateManager::new();
      state.set(dropping_struct_a);
      assert!(!*drop_flag_a.read().unwrap());

      state.set(dropping_struct_b);
      assert!(!*drop_flag_a.read().unwrap());
      assert!(!*drop_flag_b.read().unwrap());
    }

    assert!(*drop_flag_a.read().unwrap());
    assert!(*drop_flag_b.read().unwrap());
  }
}
