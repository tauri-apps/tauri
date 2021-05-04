// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::command::{CommandArg, CommandItem};
use crate::{InvokeError, Params};
use state::Container;
use std::marker::PhantomData;

/// A guard for a state value.
pub struct State<'r, T: Send + Sync + 'static>(&'r T);

impl<'r, T: Send + Sync + 'static> State<'r, T> {
  /// Retrieve a borrow to the underlying value with a lifetime of `'r`.
  /// Using this method is typically unnecessary as `State` implements
  /// [`Deref`] with a [`Deref::Target`] of `T`.
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

/// A guard for a state value that references [`Params`].
pub struct StateP<'r, T: Send + Sync + 'static, P: Params> {
  state: State<'r, T>,
  _params: PhantomData<P>,
}

impl<'r, T: Send + Sync + 'static, P: Params> StateP<'r, T, P> {
  /// Retrieve a borrow to the underlying value with a lifetime of `'r`.
  /// Using this method is typically unnecessary as `State` implements
  /// [`Deref`] with a [`Deref::Target`] of `T`.
  #[inline(always)]
  pub fn inner(&self) -> &'r T {
    self.state.inner()
  }
}

impl<T: Send + Sync + 'static, P: Params> std::ops::Deref for StateP<'_, T, P> {
  type Target = T;

  #[inline(always)]
  fn deref(&self) -> &T {
    self.state.deref()
  }
}

impl<T: Send + Sync + 'static> Clone for State<'_, T> {
  fn clone(&self) -> Self {
    State(self.0)
  }
}

impl<'r, 'de: 'r, T: Send + Sync + 'static, P: Params> CommandArg<'de, P> for State<'r, T> {
  /// Grabs the [`State`] from the [`CommandItem`]. This will never fail.
  fn from_command(command: CommandItem<'de, P>) -> Result<Self, InvokeError> {
    Ok(command.message.state_ref().get())
  }
}

impl<'r, 'de: 'r, T: Send + Sync + 'static, P: Params> CommandArg<'de, P> for StateP<'r, T, P> {
  /// Grabs the [`State`] from the [`CommandItem`]. This will never fail.
  fn from_command(command: CommandItem<'de, P>) -> Result<Self, InvokeError> {
    Ok(command.message.state_ref().with_params())
  }
}

/// The Tauri state manager.
pub struct StateManager(pub(crate) Container);

impl StateManager {
  pub(crate) fn new() -> Self {
    Self(Container::new())
  }

  pub(crate) fn set<T: Send + Sync + 'static>(&self, state: T) -> bool {
    self.0.set(state)
  }

  /// Gets the state associated with the specified type.
  pub fn get<T: Send + Sync + 'static>(&self) -> State<'_, T> {
    State(self.0.get())
  }

  /// Gets the state associated with the specified type - while specifying [`Params`]
  pub fn with_params<T: Send + Sync + 'static, P: Params>(&self) -> StateP<'_, T, P> {
    StateP {
      state: State(self.0.get()),
      _params: PhantomData,
    }
  }
}
