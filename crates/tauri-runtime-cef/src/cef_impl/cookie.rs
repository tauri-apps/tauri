// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::{Arc, Mutex, mpsc::Sender};

use cef::{rc::*, *};
use tauri_runtime::{Cookie, Result};
use url::Url;

use crate::AppWebview;

type CookieResultSender = Sender<Result<Vec<Cookie<'static>>>>;
type CollectedCookies = Arc<Mutex<Vec<Cookie<'static>>>>;

fn cookie_from_cef(cookie: &cef::Cookie) -> Cookie<'static> {
  let name = cookie.name.to_string();
  let value = cookie.value.to_string();
  let domain = cookie.domain.to_string();
  let path = cookie.path.to_string();

  let mut builder = Cookie::build((name, value));
  if !domain.is_empty() {
    builder = builder.domain(domain);
  }
  if !path.is_empty() {
    builder = builder.path(path);
  }
  if cookie.secure == 1 {
    builder = builder.secure(true);
  }
  if cookie.httponly == 1 {
    builder = builder.http_only(true);
  }

  builder.build().into_owned()
}

fn cef_cookie_from_cookie(cookie: &Cookie<'_>) -> cef::Cookie {
  let mut cef_cookie = cef::Cookie {
    name: cef::CefString::from(cookie.name()),
    value: cef::CefString::from(cookie.value()),
    ..Default::default()
  };

  if let Some(domain) = cookie.domain() {
    cef_cookie.domain = cef::CefString::from(domain);
  }
  if let Some(path) = cookie.path() {
    cef_cookie.path = cef::CefString::from(path);
  }
  if cookie.secure().unwrap_or(false) {
    cef_cookie.secure = 1;
  }
  if cookie.http_only().unwrap_or(false) {
    cef_cookie.httponly = 1;
  }

  cef_cookie
}

fn push_cookie_and_finish_if_done(
  cookie: Option<&cef::Cookie>,
  count: ::std::os::raw::c_int,
  total: ::std::os::raw::c_int,
  collected: &CollectedCookies,
  tx: &CookieResultSender,
) {
  if let Some(cookie) = cookie {
    let cookie = cookie_from_cef(cookie);
    collected.lock().unwrap().push(cookie);
  }

  if (count + 1) >= total {
    let _ = tx.send(Ok(collected.lock().unwrap().clone()));
  }
}

cef::wrap_cookie_visitor! {
  struct CollectUrlCookiesVisitor {
    tx: CookieResultSender,
    collected: CollectedCookies,
  }

  impl CookieVisitor {
    fn visit(
      &self,
      cookie: Option<&cef::Cookie>,
      count: ::std::os::raw::c_int,
      total: ::std::os::raw::c_int,
      _delete_cookie: Option<&mut ::std::os::raw::c_int>,
    ) -> ::std::os::raw::c_int {
      push_cookie_and_finish_if_done(cookie, count, total, &self.collected, &self.tx);
      1
    }
  }
}

cef::wrap_cookie_visitor! {
  struct CollectAllCookiesVisitor {
    tx: CookieResultSender,
    collected: CollectedCookies,
  }

  impl CookieVisitor {
    fn visit(
      &self,
      cookie: Option<&cef::Cookie>,
      count: ::std::os::raw::c_int,
      total: ::std::os::raw::c_int,
      _delete_cookie: Option<&mut ::std::os::raw::c_int>,
    ) -> ::std::os::raw::c_int {
      push_cookie_and_finish_if_done(cookie, count, total, &self.collected, &self.tx);
      1
    }
  }
}

pub(crate) fn visit_url_cookies(manager: CookieManager, url: Url, tx: CookieResultSender) {
  let collected = Arc::new(Mutex::new(Vec::new()));
  let mut visitor = CollectUrlCookiesVisitor::new(tx.clone(), collected);
  let url = cef::CefString::from(url.as_str());

  if manager.visit_url_cookies(Some(&url), 1, Some(&mut visitor)) == 0 {
    let _ = tx.send(Ok(Vec::new()));
  }
}

pub(crate) fn visit_all_cookies(manager: CookieManager, tx: CookieResultSender) {
  let collected = Arc::new(Mutex::new(Vec::new()));
  let mut visitor = CollectAllCookiesVisitor::new(tx.clone(), collected);

  if manager.visit_all_cookies(Some(&mut visitor)) == 0 {
    let _ = tx.send(Ok(Vec::new()));
  }
}

pub(crate) fn set_cookie(manager: CookieManager, url: Option<String>, cookie: Cookie<'static>) {
  let cef_cookie = cef_cookie_from_cookie(&cookie);
  let url = url.as_ref().map(|u| cef::CefString::from(u.as_str()));
  manager.set_cookie(
    url.as_ref(),
    Some(&cef_cookie),
    Option::<&mut cef::SetCookieCallback>::None,
  );
}

pub(crate) fn delete_cookie(manager: CookieManager, url: Option<String>, cookie: Cookie<'static>) {
  let url = url.as_ref().map(|u| cef::CefString::from(u.as_str()));
  let name = cef::CefString::from(cookie.name());
  manager.delete_cookies(
    url.as_ref(),
    Some(&name),
    Option::<&mut cef::DeleteCookiesCallback>::None,
  );
}

impl AppWebview {
  pub fn cookie_manager(&self) -> Option<CookieManager> {
    self
      .host
      .request_context()
      .and_then(|rc| rc.cookie_manager(None))
  }
}
