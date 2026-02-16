// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cef::{rc::*, *};
use std::sync::{Arc, Mutex, mpsc::Sender};

cef::wrap_cookie_visitor! {
  pub struct CollectUrlCookiesVisitor {
    pub tx: Sender<tauri_runtime::Result<Vec<tauri_runtime::Cookie<'static>>>>,
    pub collected: Arc<Mutex<Vec<tauri_runtime::Cookie<'static>>>>,
  }

  impl CookieVisitor {
    fn visit(
      &self,
      cookie: Option<&cef::Cookie>,
      count: ::std::os::raw::c_int,
      total: ::std::os::raw::c_int,
      _delete_cookie: Option<&mut ::std::os::raw::c_int>,
    ) -> ::std::os::raw::c_int {
      if let Some(c) = cookie {
        let name = c.name.to_string();
        let value = c.value.to_string();
        let domain = c.domain.to_string();
        let path = c.path.to_string();

        let mut builder = tauri_runtime::Cookie::build((name, value));
        if !domain.is_empty() { builder = builder.domain(domain); }
        if !path.is_empty() { builder = builder.path(path); }
        if c.secure == 1 { builder = builder.secure(true); }
        if c.httponly == 1 { builder = builder.http_only(true); }
        let ck = builder.build();

        self.collected.lock().unwrap().push(ck.into_owned());
      }

      if (count + 1) >= total {
        let _ = self.tx.send(Ok(self.collected.lock().unwrap().clone()));
      }
      1
    }
  }
}

cef::wrap_cookie_visitor! {
  pub struct CollectAllCookiesVisitor {
    pub tx: Sender<tauri_runtime::Result<Vec<tauri_runtime::Cookie<'static>>>>,
    pub collected: Arc<Mutex<Vec<tauri_runtime::Cookie<'static>>>>,
  }

  impl CookieVisitor {
    fn visit(
      &self,
      cookie: Option<&cef::Cookie>,
      count: ::std::os::raw::c_int,
      total: ::std::os::raw::c_int,
      _delete_cookie: Option<&mut ::std::os::raw::c_int>,
    ) -> ::std::os::raw::c_int {
      if let Some(c) = cookie {
        let name = c.name.to_string();
        let value = c.value.to_string();
        let domain = c.domain.to_string();
        let path = c.path.to_string();

        let mut builder = tauri_runtime::Cookie::build((name, value));
        if !domain.is_empty() { builder = builder.domain(domain); }
        if !path.is_empty() { builder = builder.path(path); }
        if c.secure == 1 { builder = builder.secure(true); }
        if c.httponly == 1 { builder = builder.http_only(true); }
        let ck = builder.build();

        self.collected.lock().unwrap().push(ck.into_owned());
      }

      if (count + 1) >= total {
        let _ = self.tx.send(Ok(self.collected.lock().unwrap().clone()));
      }
      1
    }
  }
}
