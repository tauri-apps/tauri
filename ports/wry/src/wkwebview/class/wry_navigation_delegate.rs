// Copyright 2020-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::{Arc, Mutex};

use objc2::{define_class, msg_send, rc::Retained, runtime::NSObject, MainThreadOnly};
use objc2_foundation::{MainThreadMarker, NSObjectProtocol};
use objc2_web_kit::{
  WKDownload, WKNavigation, WKNavigationAction, WKNavigationActionPolicy, WKNavigationDelegate,
  WKNavigationResponse, WKNavigationResponsePolicy,
};

#[cfg(target_os = "ios")]
use crate::wkwebview::ios::WKWebView::WKWebView;
#[cfg(target_os = "macos")]
use objc2_web_kit::WKWebView;

use crate::{
  url_from_webview,
  wkwebview::{
    download::{navigation_download_action, navigation_download_response},
    navigation::{
      did_commit_navigation, did_finish_navigation, navigation_policy, navigation_policy_response,
      web_content_process_did_terminate,
    },
  },
  PageLoadEvent, WryWebView,
};

use super::wry_download_delegate::WryDownloadDelegate;

pub struct WryNavigationDelegateIvars {
  pub pending_scripts: Arc<Mutex<Option<Vec<String>>>>,
  pub has_download_handler: bool,
  pub navigation_policy_function: Box<dyn Fn(String) -> bool>,
  pub download_delegate: Option<Retained<WryDownloadDelegate>>,
  pub on_page_load_handler: Option<Box<dyn Fn(PageLoadEvent)>>,
  pub on_web_content_process_terminate_handler: Option<Box<dyn Fn()>>,
}

define_class!(
  #[unsafe(super(NSObject))]
  #[thread_kind = MainThreadOnly]
  #[ivars = WryNavigationDelegateIvars]
  pub struct WryNavigationDelegate;

  unsafe impl NSObjectProtocol for WryNavigationDelegate {}

  unsafe impl WKNavigationDelegate for WryNavigationDelegate {
    #[unsafe(method(webView:decidePolicyForNavigationAction:decisionHandler:))]
    fn navigation_policy(
      &self,
      webview: &WKWebView,
      action: &WKNavigationAction,
      handler: &block2::Block<dyn Fn(WKNavigationActionPolicy)>,
    ) {
      navigation_policy(self, webview, action, handler);
    }

    #[unsafe(method(webView:decidePolicyForNavigationResponse:decisionHandler:))]
    fn navigation_policy_response(
      &self,
      webview: &WKWebView,
      response: &WKNavigationResponse,
      handler: &block2::Block<dyn Fn(WKNavigationResponsePolicy)>,
    ) {
      navigation_policy_response(self, webview, response, handler);
    }

    #[unsafe(method(webView:didFinishNavigation:))]
    fn did_finish_navigation(&self, webview: &WKWebView, navigation: &WKNavigation) {
      did_finish_navigation(self, webview, navigation);
    }

    #[unsafe(method(webView:didCommitNavigation:))]
    fn did_commit_navigation(&self, webview: &WKWebView, navigation: &WKNavigation) {
      did_commit_navigation(self, webview, navigation);
    }

    #[unsafe(method(webView:navigationAction:didBecomeDownload:))]
    fn navigation_download_action(
      &self,
      webview: &WKWebView,
      action: &WKNavigationAction,
      download: &WKDownload,
    ) {
      navigation_download_action(self, webview, action, download);
    }

    #[unsafe(method(webView:navigationResponse:didBecomeDownload:))]
    fn navigation_download_response(
      &self,
      webview: &WKWebView,
      response: &WKNavigationResponse,
      download: &WKDownload,
    ) {
      navigation_download_response(self, webview, response, download);
    }

    #[unsafe(method(webViewWebContentProcessDidTerminate:))]
    fn web_content_process_did_terminate(&self, webview: &WKWebView) {
      web_content_process_did_terminate(self, webview);
    }
  }
);

impl WryNavigationDelegate {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    webview: Retained<WryWebView>,
    pending_scripts: Arc<Mutex<Option<Vec<String>>>>,
    has_download_handler: bool,
    navigation_handler: Option<Box<dyn Fn(String) -> bool>>,
    download_delegate: Option<Retained<WryDownloadDelegate>>,
    on_page_load_handler: Option<Box<dyn Fn(PageLoadEvent, String)>>,
    on_web_content_process_terminate_handler: Option<Box<dyn Fn()>>,
    mtm: MainThreadMarker,
  ) -> Retained<Self> {
    let navigation_policy_function = Box::new(move |url: String| -> bool {
      navigation_handler
        .as_ref()
        .map_or(true, |navigation_handler| (navigation_handler)(url))
    });

    let on_page_load_handler = if let Some(handler) = on_page_load_handler {
      let custom_handler = Box::new(move |event| {
        handler(event, url_from_webview(&webview).unwrap_or_default());
      }) as Box<dyn Fn(PageLoadEvent)>;
      Some(custom_handler)
    } else {
      None
    };

    let on_web_content_process_terminate_handler =
      if let Some(handler) = on_web_content_process_terminate_handler {
        let custom_handler = Box::new(move || {
          handler();
        }) as Box<dyn Fn()>;
        Some(custom_handler)
      } else {
        None
      };

    let delegate = mtm
      .alloc::<WryNavigationDelegate>()
      .set_ivars(WryNavigationDelegateIvars {
        pending_scripts,
        navigation_policy_function,
        has_download_handler,
        download_delegate,
        on_page_load_handler,
        on_web_content_process_terminate_handler,
      });

    unsafe { msg_send![super(delegate), init] }
  }
}
