// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! A dumb of all wrapper types

use std::borrow::Cow;
use std::path::PathBuf;

use tauri_runtime::dpi::{
  LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Rect, Size,
};
use tauri_runtime::http::header::CONTENT_TYPE;
use tauri_runtime::http::{Request as HttpRequest, RequestParts, Response as HttpResponse};
use tauri_runtime::menu::{AboutMetadata, CustomMenuItem, MenuItem};
use tauri_runtime::monitor::Monitor;
use tauri_runtime::window::{CursorIcon, FileDropEvent, UserAttentionType, WindowEvent};
use tauri_runtime::{DeviceEventFilter, Error, Icon};
use tauri_utils::Theme;
#[cfg(target_os = "macos")]
use tauri_utils::TitleBarStyle;
use wry::application::dpi::{
  LogicalPosition as WryLogicalPosition, LogicalSize as WryLogicalSize,
  PhysicalPosition as WryPhysicalPosition, PhysicalSize as WryPhysicalSize,
  Position as WryPosition, Size as WrySize,
};
use wry::application::event::{Rectangle as WryRectangle, WindowEvent as WryWindowEvent};
use wry::application::event_loop::DeviceEventFilter as WryDeviceEventFilter;
use wry::application::menu::{
  AboutMetadata as WryAboutMetadata, MenuId as WryMenuId, MenuItem as WryMenuItem,
  MenuItemAttributes as WryMenuItemAttributes,
};
use wry::application::monitor::MonitorHandle;

use wry::application::window::{
  CursorIcon as WryCursorIcon, Icon as WryWindowIcon, Theme as WryTheme,
  UserAttentionType as WryUserAttentionType,
};
use wry::http::{Request as WryRequest, Response as WryResponse};
use wry::webview::FileDropEvent as WryFileDropEvent;

use crate::window::WindowHandle;
use crate::WebviewEvent;

pub(crate) struct HttpRequestWrapper(pub(crate) HttpRequest);

impl From<&WryRequest<Vec<u8>>> for HttpRequestWrapper {
  fn from(req: &WryRequest<Vec<u8>>) -> Self {
    let parts = RequestParts {
      uri: req.uri().to_string(),
      method: req.method().clone(),
      headers: req.headers().clone(),
    };
    Self(HttpRequest::new_internal(parts, req.body().clone()))
  }
}

// response
pub struct HttpResponseWrapper(pub(crate) WryResponse<Cow<'static, [u8]>>);
impl From<HttpResponse> for HttpResponseWrapper {
  fn from(response: HttpResponse) -> Self {
    let (parts, body) = response.into_parts();
    let mut res_builder = WryResponse::builder()
      .status(parts.status)
      .version(parts.version);
    if let Some(mime) = parts.mimetype {
      res_builder = res_builder.header(CONTENT_TYPE, mime);
    }
    for (name, val) in parts.headers.iter() {
      res_builder = res_builder.header(name, val);
    }

    let res = res_builder.body(body).unwrap();
    Self(res)
  }
}

pub struct MenuItemAttributesWrapper<'a>(pub WryMenuItemAttributes<'a>);

impl<'a> From<&'a CustomMenuItem> for MenuItemAttributesWrapper<'a> {
  fn from(item: &'a CustomMenuItem) -> Self {
    let mut attributes = WryMenuItemAttributes::new(&item.title)
      .with_enabled(item.enabled)
      .with_selected(item.selected)
      .with_id(WryMenuId(item.id));
    if let Some(accelerator) = item.keyboard_accelerator.as_ref() {
      attributes = attributes.with_accelerators(&accelerator.parse().expect("invalid accelerator"));
    }
    Self(attributes)
  }
}

pub struct AboutMetadataWrapper(pub WryAboutMetadata);

impl From<AboutMetadata> for AboutMetadataWrapper {
  fn from(metadata: AboutMetadata) -> Self {
    Self(WryAboutMetadata {
      version: metadata.version,
      authors: metadata.authors,
      comments: metadata.comments,
      copyright: metadata.copyright,
      license: metadata.license,
      website: metadata.website,
      website_label: metadata.website_label,
    })
  }
}

pub struct MenuItemWrapper(pub WryMenuItem);

impl From<MenuItem> for MenuItemWrapper {
  fn from(item: MenuItem) -> Self {
    match item {
      MenuItem::About(name, metadata) => Self(WryMenuItem::About(
        name,
        AboutMetadataWrapper::from(metadata).0,
      )),
      MenuItem::Hide => Self(WryMenuItem::Hide),
      MenuItem::Services => Self(WryMenuItem::Services),
      MenuItem::HideOthers => Self(WryMenuItem::HideOthers),
      MenuItem::ShowAll => Self(WryMenuItem::ShowAll),
      MenuItem::CloseWindow => Self(WryMenuItem::CloseWindow),
      MenuItem::Quit => Self(WryMenuItem::Quit),
      MenuItem::Copy => Self(WryMenuItem::Copy),
      MenuItem::Cut => Self(WryMenuItem::Cut),
      MenuItem::Undo => Self(WryMenuItem::Undo),
      MenuItem::Redo => Self(WryMenuItem::Redo),
      MenuItem::SelectAll => Self(WryMenuItem::SelectAll),
      MenuItem::Paste => Self(WryMenuItem::Paste),
      MenuItem::EnterFullScreen => Self(WryMenuItem::EnterFullScreen),
      MenuItem::Minimize => Self(WryMenuItem::Minimize),
      MenuItem::Zoom => Self(WryMenuItem::Zoom),
      MenuItem::Separator => Self(WryMenuItem::Separator),
      _ => unimplemented!(),
    }
  }
}

pub struct DeviceEventFilterWrapper(pub WryDeviceEventFilter);

impl From<DeviceEventFilter> for DeviceEventFilterWrapper {
  fn from(item: DeviceEventFilter) -> Self {
    match item {
      DeviceEventFilter::Always => Self(WryDeviceEventFilter::Always),
      DeviceEventFilter::Never => Self(WryDeviceEventFilter::Never),
      DeviceEventFilter::Unfocused => Self(WryDeviceEventFilter::Unfocused),
    }
  }
}

#[cfg(target_os = "macos")]
pub struct NativeImageWrapper(pub WryNativeImage);

#[cfg(target_os = "macos")]
impl std::fmt::Debug for NativeImageWrapper {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("NativeImageWrapper").finish()
  }
}

#[cfg(target_os = "macos")]
impl From<NativeImage> for NativeImageWrapper {
  fn from(image: NativeImage) -> NativeImageWrapper {
    let wry_image = match image {
      NativeImage::Add => WryNativeImage::Add,
      NativeImage::Advanced => WryNativeImage::Advanced,
      NativeImage::Bluetooth => WryNativeImage::Bluetooth,
      NativeImage::Bookmarks => WryNativeImage::Bookmarks,
      NativeImage::Caution => WryNativeImage::Caution,
      NativeImage::ColorPanel => WryNativeImage::ColorPanel,
      NativeImage::ColumnView => WryNativeImage::ColumnView,
      NativeImage::Computer => WryNativeImage::Computer,
      NativeImage::EnterFullScreen => WryNativeImage::EnterFullScreen,
      NativeImage::Everyone => WryNativeImage::Everyone,
      NativeImage::ExitFullScreen => WryNativeImage::ExitFullScreen,
      NativeImage::FlowView => WryNativeImage::FlowView,
      NativeImage::Folder => WryNativeImage::Folder,
      NativeImage::FolderBurnable => WryNativeImage::FolderBurnable,
      NativeImage::FolderSmart => WryNativeImage::FolderSmart,
      NativeImage::FollowLinkFreestanding => WryNativeImage::FollowLinkFreestanding,
      NativeImage::FontPanel => WryNativeImage::FontPanel,
      NativeImage::GoLeft => WryNativeImage::GoLeft,
      NativeImage::GoRight => WryNativeImage::GoRight,
      NativeImage::Home => WryNativeImage::Home,
      NativeImage::IChatTheater => WryNativeImage::IChatTheater,
      NativeImage::IconView => WryNativeImage::IconView,
      NativeImage::Info => WryNativeImage::Info,
      NativeImage::InvalidDataFreestanding => WryNativeImage::InvalidDataFreestanding,
      NativeImage::LeftFacingTriangle => WryNativeImage::LeftFacingTriangle,
      NativeImage::ListView => WryNativeImage::ListView,
      NativeImage::LockLocked => WryNativeImage::LockLocked,
      NativeImage::LockUnlocked => WryNativeImage::LockUnlocked,
      NativeImage::MenuMixedState => WryNativeImage::MenuMixedState,
      NativeImage::MenuOnState => WryNativeImage::MenuOnState,
      NativeImage::MobileMe => WryNativeImage::MobileMe,
      NativeImage::MultipleDocuments => WryNativeImage::MultipleDocuments,
      NativeImage::Network => WryNativeImage::Network,
      NativeImage::Path => WryNativeImage::Path,
      NativeImage::PreferencesGeneral => WryNativeImage::PreferencesGeneral,
      NativeImage::QuickLook => WryNativeImage::QuickLook,
      NativeImage::RefreshFreestanding => WryNativeImage::RefreshFreestanding,
      NativeImage::Refresh => WryNativeImage::Refresh,
      NativeImage::Remove => WryNativeImage::Remove,
      NativeImage::RevealFreestanding => WryNativeImage::RevealFreestanding,
      NativeImage::RightFacingTriangle => WryNativeImage::RightFacingTriangle,
      NativeImage::Share => WryNativeImage::Share,
      NativeImage::Slideshow => WryNativeImage::Slideshow,
      NativeImage::SmartBadge => WryNativeImage::SmartBadge,
      NativeImage::StatusAvailable => WryNativeImage::StatusAvailable,
      NativeImage::StatusNone => WryNativeImage::StatusNone,
      NativeImage::StatusPartiallyAvailable => WryNativeImage::StatusPartiallyAvailable,
      NativeImage::StatusUnavailable => WryNativeImage::StatusUnavailable,
      NativeImage::StopProgressFreestanding => WryNativeImage::StopProgressFreestanding,
      NativeImage::StopProgress => WryNativeImage::StopProgress,

      NativeImage::TrashEmpty => WryNativeImage::TrashEmpty,
      NativeImage::TrashFull => WryNativeImage::TrashFull,
      NativeImage::User => WryNativeImage::User,
      NativeImage::UserAccounts => WryNativeImage::UserAccounts,
      NativeImage::UserGroup => WryNativeImage::UserGroup,
      NativeImage::UserGuest => WryNativeImage::UserGuest,
    };
    Self(wry_image)
  }
}

/// Wrapper around a [`wry::application::window::Icon`] that can be created from an [`Icon`].
pub struct WryIcon(pub WryWindowIcon);

pub(crate) fn icon_err<E: std::error::Error + Send + Sync + 'static>(e: E) -> Error {
  Error::InvalidIcon(Box::new(e))
}

impl TryFrom<Icon> for WryIcon {
  type Error = Error;
  fn try_from(icon: Icon) -> std::result::Result<Self, Self::Error> {
    WryWindowIcon::from_rgba(icon.rgba, icon.width, icon.height)
      .map(Self)
      .map_err(icon_err)
  }
}

pub struct WindowEventWrapper(pub Option<WindowEvent>);

impl WindowEventWrapper {
  pub(crate) fn parse(webview: &Option<WindowHandle>, event: &WryWindowEvent<'_>) -> Self {
    match event {
      // resized event from tao doesn't include a reliable size on macOS
      // because wry replaces the NSView
      WryWindowEvent::Resized(_) => {
        if let Some(webview) = webview {
          Self(Some(WindowEvent::Resized(
            PhysicalSizeWrapper(webview.inner_size()).into(),
          )))
        } else {
          Self(None)
        }
      }
      e => e.into(),
    }
  }
}

pub fn map_theme(theme: &WryTheme) -> Theme {
  match theme {
    WryTheme::Light => Theme::Light,
    WryTheme::Dark => Theme::Dark,
    _ => Theme::Light,
  }
}

impl<'a> From<&WryWindowEvent<'a>> for WindowEventWrapper {
  fn from(event: &WryWindowEvent<'a>) -> Self {
    let event = match event {
      WryWindowEvent::Resized(size) => WindowEvent::Resized(PhysicalSizeWrapper(*size).into()),
      WryWindowEvent::Moved(position) => {
        WindowEvent::Moved(PhysicalPositionWrapper(*position).into())
      }
      WryWindowEvent::Destroyed => WindowEvent::Destroyed,
      WryWindowEvent::ScaleFactorChanged {
        scale_factor,
        new_inner_size,
      } => WindowEvent::ScaleFactorChanged {
        scale_factor: *scale_factor,
        new_inner_size: PhysicalSizeWrapper(**new_inner_size).into(),
      },
      #[cfg(any(linuxy, target_os = "macos"))]
      WryWindowEvent::Focused(focused) => WindowEvent::Focused(*focused),
      WryWindowEvent::ThemeChanged(theme) => WindowEvent::ThemeChanged(map_theme(theme)),
      _ => return Self(None),
    };
    Self(Some(event))
  }
}

impl From<&WebviewEvent> for WindowEventWrapper {
  fn from(event: &WebviewEvent) -> Self {
    let event = match event {
      WebviewEvent::Focused(focused) => WindowEvent::Focused(*focused),
    };
    Self(Some(event))
  }
}

pub struct MonitorHandleWrapper(pub MonitorHandle);

impl From<MonitorHandleWrapper> for Monitor {
  fn from(monitor: MonitorHandleWrapper) -> Monitor {
    Self {
      name: monitor.0.name(),
      position: PhysicalPositionWrapper(monitor.0.position()).into(),
      size: PhysicalSizeWrapper(monitor.0.size()).into(),
      scale_factor: monitor.0.scale_factor(),
    }
  }
}

pub struct PhysicalPositionWrapper<T>(pub WryPhysicalPosition<T>);

impl<T> From<PhysicalPositionWrapper<T>> for PhysicalPosition<T> {
  fn from(position: PhysicalPositionWrapper<T>) -> Self {
    Self {
      x: position.0.x,
      y: position.0.y,
    }
  }
}

impl<T> From<PhysicalPosition<T>> for PhysicalPositionWrapper<T> {
  fn from(position: PhysicalPosition<T>) -> Self {
    Self(WryPhysicalPosition {
      x: position.x,
      y: position.y,
    })
  }
}

struct LogicalPositionWrapper<T>(WryLogicalPosition<T>);

impl<T> From<LogicalPosition<T>> for LogicalPositionWrapper<T> {
  fn from(position: LogicalPosition<T>) -> Self {
    Self(WryLogicalPosition {
      x: position.x,
      y: position.y,
    })
  }
}

pub struct PhysicalSizeWrapper<T>(pub WryPhysicalSize<T>);

impl<T> From<PhysicalSizeWrapper<T>> for PhysicalSize<T> {
  fn from(size: PhysicalSizeWrapper<T>) -> Self {
    Self {
      width: size.0.width,
      height: size.0.height,
    }
  }
}

impl<T> From<PhysicalSize<T>> for PhysicalSizeWrapper<T> {
  fn from(size: PhysicalSize<T>) -> Self {
    Self(WryPhysicalSize {
      width: size.width,
      height: size.height,
    })
  }
}

struct LogicalSizeWrapper<T>(WryLogicalSize<T>);

impl<T> From<LogicalSize<T>> for LogicalSizeWrapper<T> {
  fn from(size: LogicalSize<T>) -> Self {
    Self(WryLogicalSize {
      width: size.width,
      height: size.height,
    })
  }
}

pub struct SizeWrapper(pub WrySize);

impl From<Size> for SizeWrapper {
  fn from(size: Size) -> Self {
    match size {
      Size::Logical(s) => Self(WrySize::Logical(LogicalSizeWrapper::from(s).0)),
      Size::Physical(s) => Self(WrySize::Physical(PhysicalSizeWrapper::from(s).0)),
    }
  }
}

pub struct PositionWrapper(pub WryPosition);

impl From<Position> for PositionWrapper {
  fn from(position: Position) -> Self {
    match position {
      Position::Logical(s) => Self(WryPosition::Logical(LogicalPositionWrapper::from(s).0)),
      Position::Physical(s) => Self(WryPosition::Physical(PhysicalPositionWrapper::from(s).0)),
    }
  }
}

#[derive(Debug, Clone)]
pub struct UserAttentionTypeWrapper(pub WryUserAttentionType);

impl From<UserAttentionType> for UserAttentionTypeWrapper {
  fn from(request_type: UserAttentionType) -> Self {
    let o = match request_type {
      UserAttentionType::Critical => WryUserAttentionType::Critical,
      UserAttentionType::Informational => WryUserAttentionType::Informational,
    };
    Self(o)
  }
}

#[derive(Debug)]
pub struct CursorIconWrapper(pub WryCursorIcon);

impl From<CursorIcon> for CursorIconWrapper {
  fn from(icon: CursorIcon) -> Self {
    use CursorIcon::*;
    let i = match icon {
      Default => WryCursorIcon::Default,
      Crosshair => WryCursorIcon::Crosshair,
      Hand => WryCursorIcon::Hand,
      Arrow => WryCursorIcon::Arrow,
      Move => WryCursorIcon::Move,
      Text => WryCursorIcon::Text,
      Wait => WryCursorIcon::Wait,
      Help => WryCursorIcon::Help,
      Progress => WryCursorIcon::Progress,
      NotAllowed => WryCursorIcon::NotAllowed,
      ContextMenu => WryCursorIcon::ContextMenu,
      Cell => WryCursorIcon::Cell,
      VerticalText => WryCursorIcon::VerticalText,
      Alias => WryCursorIcon::Alias,
      Copy => WryCursorIcon::Copy,
      NoDrop => WryCursorIcon::NoDrop,
      Grab => WryCursorIcon::Grab,
      Grabbing => WryCursorIcon::Grabbing,
      AllScroll => WryCursorIcon::AllScroll,
      ZoomIn => WryCursorIcon::ZoomIn,
      ZoomOut => WryCursorIcon::ZoomOut,
      EResize => WryCursorIcon::EResize,
      NResize => WryCursorIcon::NResize,
      NeResize => WryCursorIcon::NeResize,
      NwResize => WryCursorIcon::NwResize,
      SResize => WryCursorIcon::SResize,
      SeResize => WryCursorIcon::SeResize,
      SwResize => WryCursorIcon::SwResize,
      WResize => WryCursorIcon::WResize,
      EwResize => WryCursorIcon::EwResize,
      NsResize => WryCursorIcon::NsResize,
      NeswResize => WryCursorIcon::NeswResize,
      NwseResize => WryCursorIcon::NwseResize,
      ColResize => WryCursorIcon::ColResize,
      RowResize => WryCursorIcon::RowResize,
      _ => WryCursorIcon::Default,
    };
    Self(i)
  }
}

pub struct FileDropEventWrapper(pub(crate) WryFileDropEvent);

// on Linux, the paths are percent-encoded
#[cfg(linuxy)]
fn decode_path(path: PathBuf) -> PathBuf {
  percent_encoding::percent_decode(path.display().to_string().as_bytes())
    .decode_utf8_lossy()
    .into_owned()
    .into()
}

// on Windows and macOS, we do not need to decode the path
#[cfg(not(linuxy))]
fn decode_path(path: PathBuf) -> PathBuf {
  path
}

impl From<FileDropEventWrapper> for FileDropEvent {
  fn from(event: FileDropEventWrapper) -> Self {
    match event.0 {
      WryFileDropEvent::Hovered { paths, position: _ } => {
        FileDropEvent::Hovered(paths.into_iter().map(decode_path).collect())
      }
      WryFileDropEvent::Dropped { paths, position: _ } => {
        FileDropEvent::Dropped(paths.into_iter().map(decode_path).collect())
      }
      // default to cancelled
      // FIXME(maybe): Add `FileDropEvent::Unknown` event?
      _ => FileDropEvent::Cancelled,
    }
  }
}

#[cfg(linuxy)]
pub struct GtkWindow(pub gtk::ApplicationWindow);
#[cfg(linuxy)]
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for GtkWindow {}

pub struct RawWindowHandle(pub raw_window_handle::RawWindowHandle);
unsafe impl Send for RawWindowHandle {}

#[derive(Debug)]
pub struct RectWrapper(pub(crate) WryRectangle);
impl From<RectWrapper> for Rect {
  fn from(value: RectWrapper) -> Self {
    Rect {
      position: PhysicalPositionWrapper(value.0.position).into(),
      size: PhysicalSizeWrapper(value.0.size).into(),
    }
  }
}
