// Copyright 2020-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{cell::RefCell, path::PathBuf, rc::Rc};

use objc2::{define_class, msg_send, rc::Retained, runtime::NSObject, MainThreadOnly};
use objc2_foundation::{
  MainThreadMarker, NSData, NSError, NSObjectProtocol, NSString, NSURLResponse, NSURL,
};
use objc2_web_kit::{WKDownload, WKDownloadDelegate};

use crate::wkwebview::download::{download_did_fail, download_did_finish, download_policy};

pub struct WryDownloadDelegateIvars {
  pub started: Option<RefCell<Box<dyn FnMut(String, &mut PathBuf) -> bool + 'static>>>,
  pub completed: Option<Rc<dyn Fn(String, Option<PathBuf>, bool) + 'static>>,
}

define_class!(
  #[unsafe(super(NSObject))]
  #[thread_kind = MainThreadOnly]
  #[ivars = WryDownloadDelegateIvars]
  pub struct WryDownloadDelegate;

  unsafe impl NSObjectProtocol for WryDownloadDelegate {}

  unsafe impl WKDownloadDelegate for WryDownloadDelegate {
    #[unsafe(method(download:decideDestinationUsingResponse:suggestedFilename:completionHandler:))]
    fn download_policy(
      &self,
      download: &WKDownload,
      response: &NSURLResponse,
      suggested_filename: &NSString,
      handler: &block2::Block<dyn Fn(*const NSURL)>,
    ) {
      download_policy(self, download, response, suggested_filename, handler);
    }

    #[unsafe(method(downloadDidFinish:))]
    fn download_did_finish(&self, download: &WKDownload) {
      download_did_finish(self, download);
    }

    #[unsafe(method(download:didFailWithError:resumeData:))]
    fn download_did_fail(&self, download: &WKDownload, error: &NSError, resume_data: &NSData) {
      download_did_fail(self, download, error, resume_data);
    }
  }
);

impl WryDownloadDelegate {
  pub fn new(
    download_started_handler: Option<Box<dyn FnMut(String, &mut PathBuf) -> bool + 'static>>,
    download_completed_handler: Option<Rc<dyn Fn(String, Option<PathBuf>, bool) + 'static>>,
    mtm: MainThreadMarker,
  ) -> Retained<Self> {
    let delegate = mtm
      .alloc::<WryDownloadDelegate>()
      .set_ivars(WryDownloadDelegateIvars {
        started: download_started_handler.map(|handler| RefCell::new(handler)),
        completed: download_completed_handler,
      });

    unsafe { msg_send![super(delegate), init] }
  }
}
