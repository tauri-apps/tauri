use crate::{Result, WebviewId, WebviewWrapper, WindowWrapper};
use std::cell::{ RefCell, RefMut};
use std::collections::BTreeMap;
use std::sync::Arc;
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
    self.try_store_mut().map(|mut s| f(&mut s))
  }

  pub fn try_store_mut(&self) -> Result<RefMut<'_, WindowMap>> {
    self
      .0
      .try_borrow_mut()
      .map_err(|e| Error::WindowsStore(Box::new(e)))
  }
}
