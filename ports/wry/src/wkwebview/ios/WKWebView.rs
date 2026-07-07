//! This file has been imported from `objc2-web-kit` and modified to match iOS.
#![allow(warnings)]
#![allow(clippy::all)]

use std::{ffi::c_double, ptr::NonNull};

use objc2::{
  encode::{Encode, Encoding, RefEncode},
  extern_class, extern_methods,
  rc::{Allocated, Retained},
  runtime::{AnyObject, ProtocolObject},
  MainThreadOnly,
};
use objc2_core_foundation::*;
use objc2_foundation::*;
use objc2_ui_kit::*;
use objc2_web_kit::*;

use crate::*;

// NS_ENUM
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WKMediaPlaybackState(pub NSInteger);
impl WKMediaPlaybackState {
  #[doc(alias = "WKMediaPlaybackStateNone")]
  pub const None: Self = Self(0);
  #[doc(alias = "WKMediaPlaybackStatePlaying")]
  pub const Playing: Self = Self(1);
  #[doc(alias = "WKMediaPlaybackStatePaused")]
  pub const Paused: Self = Self(2);
  #[doc(alias = "WKMediaPlaybackStateSuspended")]
  pub const Suspended: Self = Self(3);
}

unsafe impl Encode for WKMediaPlaybackState {
  const ENCODING: Encoding = NSInteger::ENCODING;
}

unsafe impl RefEncode for WKMediaPlaybackState {
  const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

// NS_ENUM
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WKMediaCaptureState(pub NSInteger);
impl WKMediaCaptureState {
  #[doc(alias = "WKMediaCaptureStateNone")]
  pub const None: Self = Self(0);
  #[doc(alias = "WKMediaCaptureStateActive")]
  pub const Active: Self = Self(1);
  #[doc(alias = "WKMediaCaptureStateMuted")]
  pub const Muted: Self = Self(2);
}

unsafe impl Encode for WKMediaCaptureState {
  const ENCODING: Encoding = NSInteger::ENCODING;
}

unsafe impl RefEncode for WKMediaCaptureState {
  const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

// NS_ENUM
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WKFullscreenState(pub NSInteger);
impl WKFullscreenState {
  #[doc(alias = "WKFullscreenStateNotInFullscreen")]
  pub const NotInFullscreen: Self = Self(0);
  #[doc(alias = "WKFullscreenStateEnteringFullscreen")]
  pub const EnteringFullscreen: Self = Self(1);
  #[doc(alias = "WKFullscreenStateInFullscreen")]
  pub const InFullscreen: Self = Self(2);
  #[doc(alias = "WKFullscreenStateExitingFullscreen")]
  pub const ExitingFullscreen: Self = Self(3);
}

unsafe impl Encode for WKFullscreenState {
  const ENCODING: Encoding = NSInteger::ENCODING;
}

unsafe impl RefEncode for WKFullscreenState {
  const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

extern_class!(
  #[unsafe(super(UIView, UIResponder, NSObject))]
  #[thread_kind = MainThreadOnly]
  #[derive(Debug, PartialEq, Eq, Hash)]
  pub struct WKWebView;
);

#[cfg(target_os = "macos")]
unsafe impl NSAccessibility for WKWebView {}

#[cfg(target_os = "macos")]
unsafe impl NSAccessibilityElementProtocol for WKWebView {}

#[cfg(target_os = "macos")]
unsafe impl NSAnimatablePropertyContainer for WKWebView {}

#[cfg(target_os = "macos")]
unsafe impl NSAppearanceCustomization for WKWebView {}

unsafe impl NSCoding for WKWebView {}

#[cfg(target_os = "macos")]
unsafe impl NSDraggingDestination for WKWebView {}

unsafe impl NSObjectProtocol for WKWebView {}

#[cfg(target_os = "macos")]
unsafe impl NSUserInterfaceItemIdentification for WKWebView {}

impl WKWebView {
  extern_methods!(
    // #[cfg(feature = "WKWebViewConfiguration")]
    #[unsafe(method(configuration))]
    pub unsafe fn configuration(&self) -> Retained<WKWebViewConfiguration>;

    // #[cfg(feature = "WKNavigationDelegate")]
    #[unsafe(method(navigationDelegate))]
    pub unsafe fn navigationDelegate(
      &self,
    ) -> Option<Retained<ProtocolObject<dyn WKNavigationDelegate>>>;

    // #[cfg(feature = "WKNavigationDelegate")]
    #[unsafe(method(setNavigationDelegate:))]
    pub unsafe fn setNavigationDelegate(
      &self,
      navigation_delegate: Option<&ProtocolObject<dyn WKNavigationDelegate>>,
    );

    // #[cfg(feature = "WKUIDelegate")]
    #[unsafe(method(UIDelegate))]
    pub unsafe fn UIDelegate(&self) -> Option<Retained<ProtocolObject<dyn WKUIDelegate>>>;

    // #[cfg(feature = "WKUIDelegate")]
    #[unsafe(method(setUIDelegate:))]
    pub unsafe fn setUIDelegate(&self, ui_delegate: Option<&ProtocolObject<dyn WKUIDelegate>>);

    #[cfg(target_os = "macos")]
    // #[cfg(feature = "WKBackForwardList")]
    #[unsafe(method(backForwardList))]
    pub unsafe fn backForwardList(&self) -> Retained<WKBackForwardList>;

    // #[cfg(feature = "WKWebViewConfiguration")]
    #[unsafe(method(initWithFrame:configuration:))]
    pub unsafe fn initWithFrame_configuration(
      this: Allocated<Self>,
      frame: CGRect,
      configuration: &WKWebViewConfiguration,
    ) -> Retained<Self>;

    #[unsafe(method(initWithCoder:))]
    pub unsafe fn initWithCoder(this: Allocated<Self>, coder: &NSCoder) -> Option<Retained<Self>>;

    // #[cfg(feature = "WKNavigation")]
    #[unsafe(method(loadRequest:))]
    pub unsafe fn loadRequest(&self, request: &NSURLRequest) -> Option<Retained<WKNavigation>>;

    // #[cfg(feature = "WKNavigation")]
    #[unsafe(method(loadFileURL:allowingReadAccessToURL:))]
    pub unsafe fn loadFileURL_allowingReadAccessToURL(
      &self,
      url: &NSURL,
      read_access_url: &NSURL,
    ) -> Option<Retained<WKNavigation>>;

    // #[cfg(feature = "WKNavigation")]
    #[unsafe(method(loadHTMLString:baseURL:))]
    pub unsafe fn loadHTMLString_baseURL(
      &self,
      string: &NSString,
      base_url: Option<&NSURL>,
    ) -> Option<Retained<WKNavigation>>;

    // #[cfg(feature = "WKNavigation")]
    #[unsafe(method(loadData:MIMEType:characterEncodingName:baseURL:))]
    pub unsafe fn loadData_MIMEType_characterEncodingName_baseURL(
      &self,
      data: &NSData,
      mime_type: &NSString,
      character_encoding_name: &NSString,
      base_url: &NSURL,
    ) -> Option<Retained<WKNavigation>>;

    #[cfg(target_os = "macos")]
    // #[cfg(all(feature = "WKBackForwardListItem", feature = "WKNavigation"))]
    #[unsafe(method(goToBackForwardListItem:))]
    pub unsafe fn goToBackForwardListItem(
      &self,
      item: &WKBackForwardListItem,
    ) -> Option<Retained<WKNavigation>>;

    #[unsafe(method(title))]
    pub unsafe fn title(&self) -> Option<Retained<NSString>>;

    #[unsafe(method(URL))]
    pub unsafe fn URL(&self) -> Option<Retained<NSURL>>;

    #[unsafe(method(isLoading))]
    pub unsafe fn isLoading(&self) -> bool;

    #[unsafe(method(estimatedProgress))]
    pub unsafe fn estimatedProgress(&self) -> c_double;

    #[unsafe(method(hasOnlySecureContent))]
    pub unsafe fn hasOnlySecureContent(&self) -> bool;

    #[unsafe(method(canGoBack))]
    pub unsafe fn canGoBack(&self) -> bool;

    #[unsafe(method(canGoForward))]
    pub unsafe fn canGoForward(&self) -> bool;

    // #[cfg(feature = "WKNavigation")]
    #[unsafe(method(goBack))]
    pub unsafe fn goBack(&self) -> Option<Retained<WKNavigation>>;

    // #[cfg(feature = "WKNavigation")]
    #[unsafe(method(goForward))]
    pub unsafe fn goForward(&self) -> Option<Retained<WKNavigation>>;

    // #[cfg(feature = "WKNavigation")]
    #[unsafe(method(reload))]
    pub unsafe fn reload(&self) -> Option<Retained<WKNavigation>>;

    // #[cfg(feature = "WKNavigation")]
    #[unsafe(method(reloadFromOrigin))]
    pub unsafe fn reloadFromOrigin(&self) -> Option<Retained<WKNavigation>>;

    #[unsafe(method(stopLoading))]
    pub unsafe fn stopLoading(&self);

    // #[cfg(feature = "block2")]
    #[unsafe(method(evaluateJavaScript:completionHandler:))]
    pub unsafe fn evaluateJavaScript_completionHandler(
      &self,
      java_script_string: &NSString,
      completion_handler: Option<&block2::Block<dyn Fn(*mut AnyObject, *mut NSError)>>,
    );

    #[cfg(target_os = "macos")]
    // #[cfg(all(
    //   feature = "WKContentWorld",
    //   feature = "WKFrameInfo",
    //   feature = "block2"
    // ))]
    #[unsafe(method(evaluateJavaScript:inFrame:inContentWorld:completionHandler:))]
    pub unsafe fn evaluateJavaScript_inFrame_inContentWorld_completionHandler(
      &self,
      java_script_string: &NSString,
      frame: Option<&WKFrameInfo>,
      content_world: &WKContentWorld,
      completion_handler: Option<&block2::Block<dyn Fn(*mut AnyObject, *mut NSError)>>,
    );

    #[cfg(target_os = "macos")]
    // #[cfg(all(
    //   feature = "WKContentWorld",
    //   feature = "WKFrameInfo",
    //   feature = "block2"
    // ))]
    #[unsafe(method(callAsyncJavaScript:arguments:inFrame:inContentWorld:completionHandler:))]
    pub unsafe fn callAsyncJavaScript_arguments_inFrame_inContentWorld_completionHandler(
      &self,
      function_body: &NSString,
      arguments: Option<&NSDictionary<NSString, AnyObject>>,
      frame: Option<&WKFrameInfo>,
      content_world: &WKContentWorld,
      completion_handler: Option<&block2::Block<dyn Fn(*mut AnyObject, *mut NSError)>>,
    );

    // #[cfg(feature = "block2")]
    #[unsafe(method(closeAllMediaPresentationsWithCompletionHandler:))]
    pub unsafe fn closeAllMediaPresentationsWithCompletionHandler(
      &self,
      completion_handler: Option<&block2::Block<dyn Fn()>>,
    );

    #[deprecated]
    #[unsafe(method(closeAllMediaPresentations))]
    pub unsafe fn closeAllMediaPresentations(&self);

    // #[cfg(feature = "block2")]
    #[unsafe(method(pauseAllMediaPlaybackWithCompletionHandler:))]
    pub unsafe fn pauseAllMediaPlaybackWithCompletionHandler(
      &self,
      completion_handler: Option<&block2::Block<dyn Fn()>>,
    );

    // #[cfg(feature = "block2")]
    #[deprecated]
    #[unsafe(method(pauseAllMediaPlayback:))]
    pub unsafe fn pauseAllMediaPlayback(
      &self,
      completion_handler: Option<&block2::Block<dyn Fn()>>,
    );

    // #[cfg(feature = "block2")]
    #[unsafe(method(setAllMediaPlaybackSuspended:completionHandler:))]
    pub unsafe fn setAllMediaPlaybackSuspended_completionHandler(
      &self,
      suspended: bool,
      completion_handler: Option<&block2::Block<dyn Fn()>>,
    );

    // #[cfg(feature = "block2")]
    #[deprecated]
    #[unsafe(method(resumeAllMediaPlayback:))]
    pub unsafe fn resumeAllMediaPlayback(
      &self,
      completion_handler: Option<&block2::Block<dyn Fn()>>,
    );

    // #[cfg(feature = "block2")]
    #[deprecated]
    #[unsafe(method(suspendAllMediaPlayback:))]
    pub unsafe fn suspendAllMediaPlayback(
      &self,
      completion_handler: Option<&block2::Block<dyn Fn()>>,
    );

    // #[cfg(feature = "block2")]
    #[unsafe(method(requestMediaPlaybackStateWithCompletionHandler:))]
    pub unsafe fn requestMediaPlaybackStateWithCompletionHandler(
      &self,
      completion_handler: &block2::Block<dyn Fn(WKMediaPlaybackState)>,
    );

    // #[cfg(feature = "block2")]
    #[deprecated]
    #[unsafe(method(requestMediaPlaybackState:))]
    pub unsafe fn requestMediaPlaybackState(
      &self,
      completion_handler: &block2::Block<dyn Fn(WKMediaPlaybackState)>,
    );

    #[unsafe(method(cameraCaptureState))]
    pub unsafe fn cameraCaptureState(&self) -> WKMediaCaptureState;

    #[unsafe(method(microphoneCaptureState))]
    pub unsafe fn microphoneCaptureState(&self) -> WKMediaCaptureState;

    // #[cfg(feature = "block2")]
    #[unsafe(method(setCameraCaptureState:completionHandler:))]
    pub unsafe fn setCameraCaptureState_completionHandler(
      &self,
      state: WKMediaCaptureState,
      completion_handler: Option<&block2::Block<dyn Fn()>>,
    );

    // #[cfg(feature = "block2")]
    #[unsafe(method(setMicrophoneCaptureState:completionHandler:))]
    pub unsafe fn setMicrophoneCaptureState_completionHandler(
      &self,
      state: WKMediaCaptureState,
      completion_handler: Option<&block2::Block<dyn Fn()>>,
    );

    #[cfg(target_os = "macos")]
    // #[cfg(all(feature = "WKSnapshotConfiguration", feature = "block2"))]
    #[unsafe(method(takeSnapshotWithConfiguration:completionHandler:))]
    pub unsafe fn takeSnapshotWithConfiguration_completionHandler(
      &self,
      snapshot_configuration: Option<&WKSnapshotConfiguration>,
      completion_handler: &block2::Block<dyn Fn(*mut NSImage, *mut NSError)>,
    );

    #[cfg(target_os = "macos")]
    // #[cfg(all(feature = "WKPDFConfiguration", feature = "block2"))]
    #[unsafe(method(createPDFWithConfiguration:completionHandler:))]
    pub unsafe fn createPDFWithConfiguration_completionHandler(
      &self,
      pdf_configuration: Option<&WKPDFConfiguration>,
      completion_handler: &block2::Block<dyn Fn(*mut NSData, *mut NSError)>,
    );

    // #[cfg(feature = "block2")]
    #[unsafe(method(createWebArchiveDataWithCompletionHandler:))]
    pub unsafe fn createWebArchiveDataWithCompletionHandler(
      &self,
      completion_handler: &block2::Block<dyn Fn(NonNull<NSData>, NonNull<NSError>)>,
    );

    #[unsafe(method(allowsBackForwardNavigationGestures))]
    pub unsafe fn allowsBackForwardNavigationGestures(&self) -> bool;

    #[unsafe(method(setAllowsBackForwardNavigationGestures:))]
    pub unsafe fn setAllowsBackForwardNavigationGestures(
      &self,
      allows_back_forward_navigation_gestures: bool,
    );

    #[unsafe(method(customUserAgent))]
    pub unsafe fn customUserAgent(&self) -> Option<Retained<NSString>>;

    #[unsafe(method(setCustomUserAgent:))]
    pub unsafe fn setCustomUserAgent(&self, custom_user_agent: Option<&NSString>);

    #[unsafe(method(allowsLinkPreview))]
    pub unsafe fn allowsLinkPreview(&self) -> bool;

    #[unsafe(method(setAllowsLinkPreview:))]
    pub unsafe fn setAllowsLinkPreview(&self, allows_link_preview: bool);

    #[unsafe(method(allowsMagnification))]
    pub unsafe fn allowsMagnification(&self) -> bool;

    #[unsafe(method(setAllowsMagnification:))]
    pub unsafe fn setAllowsMagnification(&self, allows_magnification: bool);

    #[unsafe(method(magnification))]
    pub unsafe fn magnification(&self) -> CGFloat;

    #[unsafe(method(setMagnification:))]
    pub unsafe fn setMagnification(&self, magnification: CGFloat);

    #[unsafe(method(setMagnification:centeredAtPoint:))]
    pub unsafe fn setMagnification_centeredAtPoint(&self, magnification: CGFloat, point: CGPoint);

    #[unsafe(method(pageZoom))]
    pub unsafe fn pageZoom(&self) -> CGFloat;

    #[unsafe(method(setPageZoom:))]
    pub unsafe fn setPageZoom(&self, page_zoom: CGFloat);

    #[cfg(target_os = "macos")]
    // #[cfg(all(
    //   feature = "WKFindConfiguration",
    //   feature = "WKFindResult",
    //   feature = "block2"
    // ))]
    #[unsafe(method(findString:withConfiguration:completionHandler:))]
    pub unsafe fn findString_withConfiguration_completionHandler(
      &self,
      string: &NSString,
      configuration: Option<&WKFindConfiguration>,
      completion_handler: &block2::Block<dyn Fn(NonNull<WKFindResult>)>,
    );

    #[unsafe(method(handlesURLScheme:))]
    pub unsafe fn handlesURLScheme(url_scheme: &NSString, mtm: MainThreadMarker) -> bool;

    // #[cfg(all(feature = "WKDownload", feature = "block2"))]
    #[unsafe(method(startDownloadUsingRequest:completionHandler:))]
    pub unsafe fn startDownloadUsingRequest_completionHandler(
      &self,
      request: &NSURLRequest,
      completion_handler: &block2::Block<dyn Fn(NonNull<WKDownload>)>,
    );

    // #[cfg(all(feature = "WKDownload", feature = "block2"))]
    #[unsafe(method(resumeDownloadFromResumeData:completionHandler:))]
    pub unsafe fn resumeDownloadFromResumeData_completionHandler(
      &self,
      resume_data: &NSData,
      completion_handler: &block2::Block<dyn Fn(NonNull<WKDownload>)>,
    );

    #[unsafe(method(mediaType))]
    pub unsafe fn mediaType(&self) -> Option<Retained<NSString>>;

    #[unsafe(method(setMediaType:))]
    pub unsafe fn setMediaType(&self, media_type: Option<&NSString>);

    #[unsafe(method(interactionState))]
    pub unsafe fn interactionState(&self) -> Option<Retained<AnyObject>>;

    #[unsafe(method(setInteractionState:))]
    pub unsafe fn setInteractionState(&self, interaction_state: Option<&AnyObject>);

    // #[cfg(feature = "WKNavigation")]
    #[unsafe(method(loadSimulatedRequest:response:responseData:))]
    pub unsafe fn loadSimulatedRequest_response_responseData(
      &self,
      request: &NSURLRequest,
      response: &NSURLResponse,
      data: &NSData,
    ) -> Retained<WKNavigation>;

    // #[cfg(feature = "WKNavigation")]
    #[deprecated]
    #[unsafe(method(loadSimulatedRequest:withResponse:responseData:))]
    pub unsafe fn loadSimulatedRequest_withResponse_responseData(
      &self,
      request: &NSURLRequest,
      response: &NSURLResponse,
      data: &NSData,
    ) -> Retained<WKNavigation>;

    // #[cfg(feature = "WKNavigation")]
    #[unsafe(method(loadFileRequest:allowingReadAccessToURL:))]
    pub unsafe fn loadFileRequest_allowingReadAccessToURL(
      &self,
      request: &NSURLRequest,
      read_access_url: &NSURL,
    ) -> Retained<WKNavigation>;

    // #[cfg(feature = "WKNavigation")]
    #[unsafe(method(loadSimulatedRequest:responseHTMLString:))]
    pub unsafe fn loadSimulatedRequest_responseHTMLString(
      &self,
      request: &NSURLRequest,
      string: &NSString,
    ) -> Retained<WKNavigation>;

    // #[cfg(feature = "WKNavigation")]
    #[deprecated]
    #[unsafe(method(loadSimulatedRequest:withResponseHTMLString:))]
    pub unsafe fn loadSimulatedRequest_withResponseHTMLString(
      &self,
      request: &NSURLRequest,
      string: &NSString,
    ) -> Retained<WKNavigation>;

    #[cfg(target_os = "macos")]
    #[unsafe(method(printOperationWithPrintInfo:))]
    pub unsafe fn printOperationWithPrintInfo(
      &self,
      print_info: &NSPrintInfo,
    ) -> Retained<NSPrintOperation>;

    #[cfg(target_os = "macos")]
    #[unsafe(method(themeColor))]
    pub unsafe fn themeColor(&self) -> Option<Retained<NSColor>>;

    #[cfg(target_os = "macos")]
    #[unsafe(method(underPageBackgroundColor))]
    pub unsafe fn underPageBackgroundColor(&self) -> Retained<NSColor>;

    #[cfg(target_os = "macos")]
    #[unsafe(method(setUnderPageBackgroundColor:))]
    pub unsafe fn setUnderPageBackgroundColor(&self, under_page_background_color: Option<&NSColor>);

    #[unsafe(method(fullscreenState))]
    pub unsafe fn fullscreenState(&self) -> WKFullscreenState;

    #[unsafe(method(minimumViewportInset))]
    pub unsafe fn minimumViewportInset(&self) -> NSEdgeInsets;

    #[unsafe(method(maximumViewportInset))]
    pub unsafe fn maximumViewportInset(&self) -> NSEdgeInsets;

    #[unsafe(method(setMinimumViewportInset:maximumViewportInset:))]
    pub unsafe fn setMinimumViewportInset_maximumViewportInset(
      &self,
      minimum_viewport_inset: NSEdgeInsets,
      maximum_viewport_inset: NSEdgeInsets,
    );

    #[unsafe(method(isInspectable))]
    pub unsafe fn isInspectable(&self) -> bool;

    #[unsafe(method(setInspectable:))]
    pub unsafe fn setInspectable(&self, inspectable: bool);
  );
}

/// Methods declared on superclass `UIView`
impl WKWebView {
  extern_methods!(
    #[unsafe(method(initWithFrame:))]
    pub unsafe fn initWithFrame(this: Allocated<Self>, frame_rect: NSRect) -> Retained<Self>;
  );
}

/// Methods declared on superclass `UIResponder`
impl WKWebView {
  extern_methods!(
    #[unsafe(method(init))]
    pub unsafe fn init(this: Allocated<Self>) -> Retained<Self>;
  );
}

/// Methods declared on superclass `NSObject`
impl WKWebView {
  extern_methods!(
    #[unsafe(method(new))]
    pub unsafe fn new(mtm: MainThreadMarker) -> Retained<Self>;
  );
}

/// WKIBActions
impl WKWebView {
  extern_methods!(
    #[unsafe(method(goBack:))]
    pub unsafe fn goBack_(&self, sender: Option<&AnyObject>);

    #[unsafe(method(goForward:))]
    pub unsafe fn goForward_(&self, sender: Option<&AnyObject>);

    #[unsafe(method(reload:))]
    pub unsafe fn reload_(&self, sender: Option<&AnyObject>);

    #[unsafe(method(reloadFromOrigin:))]
    pub unsafe fn reloadFromOrigin_(&self, sender: Option<&AnyObject>);

    #[unsafe(method(stopLoading:))]
    pub unsafe fn stopLoading_(&self, sender: Option<&AnyObject>);
  );
}

#[cfg(target_os = "macos")]
unsafe impl NSUserInterfaceValidations for WKWebView {}

/// WKNSTextFinderClient
impl WKWebView {
  extern_methods!();
}

#[cfg(target_os = "macos")]
unsafe impl NSTextFinderClient for WKWebView {}

/// WKDeprecated
impl WKWebView {
  extern_methods!(
    #[deprecated]
    #[unsafe(method(certificateChain))]
    pub unsafe fn certificateChain(&self) -> Retained<NSArray>;
  );
}
