// Copyright 2020-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{ffi::c_void, ptr::null_mut};

use objc2::{
  define_class, msg_send,
  rc::Retained,
  runtime::{AnyObject, NSObject},
  AllocAnyThread, DefinedClass,
};
use objc2_foundation::{
  ns_string, NSDictionary, NSKeyValueChangeKey, NSKeyValueObservingOptions,
  NSObjectNSKeyValueObserverRegistration, NSObjectProtocol, NSString,
};

use crate::WryWebView;
pub struct DocumentTitleChangedObserverIvars {
  pub object: Retained<WryWebView>,
  pub handler: Box<dyn Fn(String)>,
}

define_class!(
  #[unsafe(super(NSObject))]
  #[ivars = DocumentTitleChangedObserverIvars]
  pub struct DocumentTitleChangedObserver;

  /// NSKeyValueObserving.
  impl DocumentTitleChangedObserver {
    #[unsafe(method(observeValueForKeyPath:ofObject:change:context:))]
    fn observe_value_for_key_path(
      &self,
      key_path: Option<&NSString>,
      of_object: Option<&AnyObject>,
      _change: Option<&NSDictionary<NSKeyValueChangeKey, AnyObject>>,
      _context: *mut c_void,
    ) {
      if let (Some(key_path), Some(object)) = (key_path, of_object) {
        unsafe {
          if key_path.isEqualToString(ns_string!("title")) {
            let handler = &self.ivars().handler;
            // if !handler.is_null() {
            let title: *const NSString = msg_send![object, title];
            handler((*title).to_string());
            // }
          }
        }
      }
    }
  }

  unsafe impl NSObjectProtocol for DocumentTitleChangedObserver {}
);

impl DocumentTitleChangedObserver {
  pub fn new(webview: Retained<WryWebView>, handler: Box<dyn Fn(String)>) -> Retained<Self> {
    let observer = Self::alloc().set_ivars(DocumentTitleChangedObserverIvars {
      object: webview,
      handler,
    });

    let observer: Retained<Self> = unsafe { msg_send![super(observer), init] };

    unsafe {
      observer
        .ivars()
        .object
        .addObserver_forKeyPath_options_context(
          &observer,
          ns_string!("title"),
          NSKeyValueObservingOptions::New,
          null_mut(),
        );
    }

    observer
  }
}

impl Drop for DocumentTitleChangedObserver {
  fn drop(&mut self) {
    unsafe {
      self
        .ivars()
        .object
        .removeObserver_forKeyPath(self, ns_string!("title"));
    }
  }
}
