// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::WindowWrapper;
use std::{
  cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut},
  collections::BTreeMap,
  fmt,
  fmt::Formatter,
};
use tauri_runtime::window::WindowId;

type WindowMap = BTreeMap<WindowId, WindowWrapper>;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
  Borrow(BorrowError),
  BorrowMut(BorrowMutError),
  WindowNotFound(WindowId),
}

impl std::error::Error for Error {}
impl fmt::Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Error::Borrow(e) => e.fmt(f),
      Error::BorrowMut(e) => e.fmt(f),
      Error::WindowNotFound(id) => write!(f, "Window not in store: {id:?}"),
    }
  }
}

impl From<Error> for tauri_runtime::Error {
  fn from(value: Error) -> Self {
    Self::WindowsStore(Box::new(value))
  }
}

#[derive(Debug, Default)]
pub struct WindowsStore(RefCell<BTreeMap<WindowId, WindowWrapper>>);

impl WindowsStore {
  pub fn window<F, T>(&self, window: WindowId, f: F) -> Result<T>
  where
    F: FnOnce(&WindowWrapper) -> T,
  {
    let store = self.try_store()?;
    let window = store.get(&window).ok_or(Error::WindowNotFound(window))?;
    Ok(f(window))
  }

  pub fn window_mut<F, T>(&self, window: WindowId, f: F) -> Result<T, Error>
  where
    F: FnOnce(&mut WindowWrapper) -> T,
  {
    let mut store = self.try_store_mut()?;
    let window = store
      .get_mut(&window)
      .ok_or(Error::WindowNotFound(window))?;
    Ok(f(window))
  }

  pub fn store<F, T>(&self, f: F) -> Result<T, Error>
  where
    F: FnOnce(&WindowMap) -> T,
  {
    self.0.try_borrow().map(|s| f(&s)).map_err(Error::Borrow)
  }

  pub fn store_mut<F, T>(&self, f: F) -> Result<T, Error>
  where
    F: FnOnce(&mut WindowMap) -> T,
  {
    self.try_store_mut().map(|mut s| f(&mut s))
  }

  fn try_store_mut(&self) -> Result<RefMut<'_, WindowMap>, Error> {
    self.0.try_borrow_mut().map_err(Error::BorrowMut)
  }

  fn try_store(&self) -> Result<Ref<'_, WindowMap>, Error> {
    self.0.try_borrow().map_err(Error::Borrow)
  }
}
