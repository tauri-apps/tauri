// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{Result, WebviewId, WebviewWrapper, WindowWrapper};
use std::{cell::RefCell, collections::BTreeMap, sync::atomic::Ordering, sync::Arc};
use tao::window::Window;
use tauri_runtime::{window::WindowId, Error};

type WindowMap = BTreeMap<WindowId, WindowWrapper>;

#[derive(Debug, Default)]
pub struct WindowsStore(RefCell<BTreeMap<WindowId, WindowWrapper>>);

impl WindowsStore {
  pub fn window<F, T>(&self, window: WindowId, f: F) -> Result<Option<T>>
  where
    F: FnOnce(&WindowWrapper) -> Option<T>,
  {
    self.store(|store| store.get(&window).and_then(f))
  }

  pub fn window_mut<F, T>(&self, window: WindowId, f: F) -> Result<Option<T>>
  where
    F: FnOnce(&mut WindowWrapper) -> Option<T>,
  {
    self.store_mut(|store| store.get_mut(&window).and_then(f))
  }

  pub fn window_inner(&self, window: WindowId) -> Result<Option<Arc<Window>>> {
    self.store(|store| store.get(&window).and_then(|w| w.inner.clone()))
  }

  pub fn window_and_webview(
    &self,
    window: WindowId,
    webview: WebviewId,
  ) -> Result<Option<(Arc<Window>, WebviewWrapper)>> {
    self.store(|store| {
      store.get(&window).and_then(|w| {
        w.inner
          .clone()
          .zip(w.webviews.iter().find(|wv| wv.id == webview).cloned())
      })
    })
  }

  pub fn insert(&self, id: WindowId, window: WindowWrapper) -> Result<Option<WindowWrapper>> {
    self.store_mut(|store| store.insert(id, window))
  }

  pub fn remove(&self, id: WindowId) -> Result<Option<WindowWrapper>> {
    self.store_mut(|store| store.remove(&id))
  }

  pub fn add_webview(&self, window: WindowId, webview: WebviewWrapper) -> Result<()> {
    self.store_mut(|store| {
      if let Some(w) = store.get_mut(&window) {
        w.webviews.push(webview);
        w.has_children.store(true, Ordering::Relaxed);
      }
    })
  }

  pub fn remove_webview(
    &self,
    window: WindowId,
    webview: WebviewId,
  ) -> Result<Option<WebviewWrapper>> {
    self.store_mut(|store| {
      store.get_mut(&window).and_then(|w| {
        w.webviews
          .iter()
          .position(|wv| wv.id == webview)
          .map(|i| w.webviews.remove(i))
      })
    })
  }

  pub fn store<F, T>(&self, f: F) -> Result<T>
  where
    F: FnOnce(&WindowMap) -> T,
  {
    self
      .0
      .try_borrow()
      .map(|s| f(&s))
      .map_err(|e| Error::WindowsStore(Box::new(e)))
  }

  pub fn store_mut<F, T>(&self, f: F) -> Result<T>
  where
    F: FnOnce(&mut WindowMap) -> T,
  {
    self
      .0
      .try_borrow_mut()
      .map(|mut s| f(&mut s))
      .map_err(|e| Error::WindowsStore(Box::new(e)))
  }
}
