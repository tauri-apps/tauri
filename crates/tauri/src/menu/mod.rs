// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Menu types and utilities.

mod builders;
mod check;
mod icon;
#[allow(clippy::module_inception)]
mod menu;
mod normal;
pub(crate) mod plugin;
mod predefined;
mod submenu;
use std::sync::Arc;

pub use builders::*;
pub use menu::{HELP_SUBMENU_ID, WINDOW_SUBMENU_ID};
use serde::{Deserialize, Serialize};

use crate::{image::Image, AppHandle, Runtime};
pub use muda::MenuId;

macro_rules! run_item_main_thread {
  ($self:ident, $ex:expr) => {{
    use std::sync::mpsc::channel;
    let (tx, rx) = channel();
    let self_ = $self.clone();
    let task = move || {
      let f = $ex;
      let _ = tx.send(f(self_));
    };
    $self
      .app_handle()
      .run_on_main_thread(task)
      .and_then(|_| rx.recv().map_err(|_| crate::Error::FailedToReceiveMessage))
  }};
}

pub(crate) use run_item_main_thread;

/// Describes a menu event emitted when a menu item is activated
#[derive(Debug, Clone, Serialize)]
pub struct MenuEvent {
  /// Id of the menu item which triggered this event
  pub id: MenuId,
}

impl MenuEvent {
  /// Returns the id of the menu item which triggered this event
  pub fn id(&self) -> &MenuId {
    &self.id
  }
}

impl From<muda::MenuEvent> for MenuEvent {
  fn from(value: muda::MenuEvent) -> Self {
    Self { id: value.id }
  }
}

macro_rules! gen_wrappers {
  (
    $(
      $(#[$attr:meta])*
      $type:ident($inner:ident$(, $kind:ident)?)
    ),*
  ) => {
    $(
      #[tauri_macros::default_runtime(crate::Wry, wry)]
      pub(crate) struct $inner<R: $crate::Runtime> {
        id: $crate::menu::MenuId,
        inner: ::std::option::Option<::muda::$type>,
        app_handle: $crate::AppHandle<R>,
      }

      /// # Safety
      ///
      /// We make sure it always runs on the main thread.
      unsafe impl<R: $crate::Runtime> Sync for $inner<R> {}
      unsafe impl<R: $crate::Runtime> Send for $inner<R> {}

      impl<R: Runtime> $crate::Resource for $type<R> {}

      impl<R: $crate::Runtime> Clone for $inner<R> {
        fn clone(&self) -> Self {
          Self {
            id: self.id.clone(),
            inner: self.inner.clone(),
            app_handle: self.app_handle.clone(),
          }
        }
      }

      impl<R: Runtime> Drop for $inner<R> {
        fn drop(&mut self) {
          let inner = self.inner.take();
          // SAFETY: inner was created on main thread and is being dropped on main thread
          let inner = $crate::UnsafeSend(inner);
          let _ = self.app_handle.run_on_main_thread(move || {
            drop(inner.take());
          });
        }
      }

      impl<R: Runtime> AsRef<::muda::$type> for $inner<R> {
        fn as_ref(&self) -> &::muda::$type {
          self.inner.as_ref().unwrap()
        }
      }


      $(#[$attr])*
      pub struct $type<R: $crate::Runtime>(::std::sync::Arc<$inner<R>>);

      impl<R: $crate::Runtime> Clone for $type<R> {
        fn clone(&self) -> Self {
          Self(self.0.clone())
        }
      }

      $(
        impl<R: $crate::Runtime> $crate::menu::sealed::IsMenuItemBase for $type<R> {
          fn inner_muda(&self) -> &dyn muda::IsMenuItem {
            (*self.0).as_ref()
          }
        }

        impl<R: $crate::Runtime> $crate::menu::IsMenuItem<R> for $type<R> {
          fn kind(&self) -> MenuItemKind<R> {
            MenuItemKind::$kind(self.clone())
          }

          fn id(&self) -> &MenuId {
            &self.0.id
          }
        }
      )*
    )*
  };
}

gen_wrappers!(
  /// A type that is either a menu bar on the window
  /// on Windows and Linux or as a global menu in the menubar on macOS.
  ///
  /// ## Platform-specific:
  ///
  /// - **macOS**: if using [`Menu`] for the global menubar, it can only contain [`Submenu`]s
  Menu(MenuInner),
  /// A menu item inside a [`Menu`] or [`Submenu`] and contains only text.
  MenuItem(MenuItemInner, MenuItem),
  /// A type that is a submenu inside a [`Menu`] or [`Submenu`]
  Submenu(SubmenuInner, Submenu),
  /// A predefined (native) menu item which has a predefined behavior by the OS or by this crate.
  PredefinedMenuItem(PredefinedMenuItemInner, Predefined),
  /// A menu item inside a [`Menu`] or [`Submenu`]
  /// and usually contains a text and a check mark or a similar toggle
  /// that corresponds to a checked and unchecked states.
  CheckMenuItem(CheckMenuItemInner, Check),
  /// A menu item inside a [`Menu`] or [`Submenu`]
  /// and usually contains an icon and a text.
  IconMenuItem(IconMenuItemInner, Icon)
);

/// Application metadata for the [`PredefinedMenuItem::about`].
#[derive(Debug, Clone, Default)]
pub struct AboutMetadata<'a> {
  /// Sets the application name.
  pub name: Option<String>,
  /// The application version.
  pub version: Option<String>,
  /// The short version, e.g. "1.0".
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Linux:** Appended to the end of `version` in parentheses.
  pub short_version: Option<String>,
  /// The authors of the application.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Unsupported.
  pub authors: Option<Vec<String>>,
  /// Application comments.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Unsupported.
  pub comments: Option<String>,
  /// The copyright of the application.
  pub copyright: Option<String>,
  /// The license of the application.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Unsupported.
  pub license: Option<String>,
  /// The application website.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Unsupported.
  pub website: Option<String>,
  /// The website label.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Unsupported.
  pub website_label: Option<String>,
  /// The credits.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Linux:** Unsupported.
  pub credits: Option<String>,
  /// The application icon.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows:** Unsupported.
  pub icon: Option<Image<'a>>,
}

/// A builder type for [`AboutMetadata`].
#[derive(Clone, Debug, Default)]
pub struct AboutMetadataBuilder<'a>(AboutMetadata<'a>);

impl<'a> AboutMetadataBuilder<'a> {
  /// Create a new about metadata builder.
  pub fn new() -> Self {
    Default::default()
  }

  /// Sets the application name.
  pub fn name<S: Into<String>>(mut self, name: Option<S>) -> Self {
    self.0.name = name.map(|s| s.into());
    self
  }
  /// Sets the application version.
  pub fn version<S: Into<String>>(mut self, version: Option<S>) -> Self {
    self.0.version = version.map(|s| s.into());
    self
  }
  /// Sets the short version, e.g. "1.0".
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Linux:** Appended to the end of `version` in parentheses.
  pub fn short_version<S: Into<String>>(mut self, short_version: Option<S>) -> Self {
    self.0.short_version = short_version.map(|s| s.into());
    self
  }
  /// Sets the authors of the application.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Unsupported.
  pub fn authors(mut self, authors: Option<Vec<String>>) -> Self {
    self.0.authors = authors;
    self
  }
  /// Application comments.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Unsupported.
  pub fn comments<S: Into<String>>(mut self, comments: Option<S>) -> Self {
    self.0.comments = comments.map(|s| s.into());
    self
  }
  /// Sets the copyright of the application.
  pub fn copyright<S: Into<String>>(mut self, copyright: Option<S>) -> Self {
    self.0.copyright = copyright.map(|s| s.into());
    self
  }
  /// Sets the license of the application.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Unsupported.
  pub fn license<S: Into<String>>(mut self, license: Option<S>) -> Self {
    self.0.license = license.map(|s| s.into());
    self
  }
  /// Sets the application website.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Unsupported.
  pub fn website<S: Into<String>>(mut self, website: Option<S>) -> Self {
    self.0.website = website.map(|s| s.into());
    self
  }
  /// Sets the website label.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Unsupported.
  pub fn website_label<S: Into<String>>(mut self, website_label: Option<S>) -> Self {
    self.0.website_label = website_label.map(|s| s.into());
    self
  }
  /// Sets the credits.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn credits<S: Into<String>>(mut self, credits: Option<S>) -> Self {
    self.0.credits = credits.map(|s| s.into());
    self
  }
  /// Sets the application icon.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows:** Unsupported.
  pub fn icon(mut self, icon: Option<Image<'a>>) -> Self {
    self.0.icon = icon;
    self
  }

  /// Construct the final [`AboutMetadata`]
  pub fn build(self) -> AboutMetadata<'a> {
    self.0
  }
}

impl TryFrom<AboutMetadata<'_>> for muda::AboutMetadata {
  type Error = crate::Error;

  fn try_from(value: AboutMetadata<'_>) -> Result<Self, Self::Error> {
    let icon = match value.icon {
      Some(i) => Some(i.try_into()?),
      None => None,
    };

    Ok(Self {
      authors: value.authors,
      name: value.name,
      version: value.version,
      short_version: value.short_version,
      comments: value.comments,
      copyright: value.copyright,
      license: value.license,
      website: value.website,
      website_label: value.website_label,
      credits: value.credits,
      icon,
    })
  }
}

/// A native Icon to be used for the menu item
///
/// ## Platform-specific:
///
/// - **Windows / Linux**: Unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum NativeIcon {
  /// An add item template image.
  Add,
  /// Advanced preferences toolbar icon for the preferences window.
  Advanced,
  /// A Bluetooth template image.
  Bluetooth,
  /// Bookmarks image suitable for a template.
  Bookmarks,
  /// A caution image.
  Caution,
  /// A color panel toolbar icon.
  ColorPanel,
  /// A column view mode template image.
  ColumnView,
  /// A computer icon.
  Computer,
  /// An enter full-screen mode template image.
  EnterFullScreen,
  /// Permissions for all users.
  Everyone,
  /// An exit full-screen mode template image.
  ExitFullScreen,
  /// A cover flow view mode template image.
  FlowView,
  /// A folder image.
  Folder,
  /// A burnable folder icon.
  FolderBurnable,
  /// A smart folder icon.
  FolderSmart,
  /// A link template image.
  FollowLinkFreestanding,
  /// A font panel toolbar icon.
  FontPanel,
  /// A `go back` template image.
  GoLeft,
  /// A `go forward` template image.
  GoRight,
  /// Home image suitable for a template.
  Home,
  /// An iChat Theater template image.
  IChatTheater,
  /// An icon view mode template image.
  IconView,
  /// An information toolbar icon.
  Info,
  /// A template image used to denote invalid data.
  InvalidDataFreestanding,
  /// A generic left-facing triangle template image.
  LeftFacingTriangle,
  /// A list view mode template image.
  ListView,
  /// A locked padlock template image.
  LockLocked,
  /// An unlocked padlock template image.
  LockUnlocked,
  /// A horizontal dash, for use in menus.
  MenuMixedState,
  /// A check mark template image, for use in menus.
  MenuOnState,
  /// A MobileMe icon.
  MobileMe,
  /// A drag image for multiple items.
  MultipleDocuments,
  /// A network icon.
  Network,
  /// A path button template image.
  Path,
  /// General preferences toolbar icon for the preferences window.
  PreferencesGeneral,
  /// A Quick Look template image.
  QuickLook,
  /// A refresh template image.
  RefreshFreestanding,
  /// A refresh template image.
  Refresh,
  /// A remove item template image.
  Remove,
  /// A reveal contents template image.
  RevealFreestanding,
  /// A generic right-facing triangle template image.
  RightFacingTriangle,
  /// A share view template image.
  Share,
  /// A slideshow template image.
  Slideshow,
  /// A badge for a `smart` item.
  SmartBadge,
  /// Small green indicator, similar to iChat's available image.
  StatusAvailable,
  /// Small clear indicator.
  StatusNone,
  /// Small yellow indicator, similar to iChat's idle image.
  StatusPartiallyAvailable,
  /// Small red indicator, similar to iChat's unavailable image.
  StatusUnavailable,
  /// A stop progress template image.
  StopProgressFreestanding,
  /// A stop progress button template image.
  StopProgress,
  /// An image of the empty trash can.
  TrashEmpty,
  /// An image of the full trash can.
  TrashFull,
  /// Permissions for a single user.
  User,
  /// User account toolbar icon for the preferences window.
  UserAccounts,
  /// Permissions for a group of users.
  UserGroup,
  /// Permissions for guests.
  UserGuest,
}

impl From<NativeIcon> for muda::NativeIcon {
  fn from(value: NativeIcon) -> Self {
    match value {
      NativeIcon::Add => muda::NativeIcon::Add,
      NativeIcon::Advanced => muda::NativeIcon::Advanced,
      NativeIcon::Bluetooth => muda::NativeIcon::Bluetooth,
      NativeIcon::Bookmarks => muda::NativeIcon::Bookmarks,
      NativeIcon::Caution => muda::NativeIcon::Caution,
      NativeIcon::ColorPanel => muda::NativeIcon::ColorPanel,
      NativeIcon::ColumnView => muda::NativeIcon::ColumnView,
      NativeIcon::Computer => muda::NativeIcon::Computer,
      NativeIcon::EnterFullScreen => muda::NativeIcon::EnterFullScreen,
      NativeIcon::Everyone => muda::NativeIcon::Everyone,
      NativeIcon::ExitFullScreen => muda::NativeIcon::ExitFullScreen,
      NativeIcon::FlowView => muda::NativeIcon::FlowView,
      NativeIcon::Folder => muda::NativeIcon::Folder,
      NativeIcon::FolderBurnable => muda::NativeIcon::FolderBurnable,
      NativeIcon::FolderSmart => muda::NativeIcon::FolderSmart,
      NativeIcon::FollowLinkFreestanding => muda::NativeIcon::FollowLinkFreestanding,
      NativeIcon::FontPanel => muda::NativeIcon::FontPanel,
      NativeIcon::GoLeft => muda::NativeIcon::GoLeft,
      NativeIcon::GoRight => muda::NativeIcon::GoRight,
      NativeIcon::Home => muda::NativeIcon::Home,
      NativeIcon::IChatTheater => muda::NativeIcon::IChatTheater,
      NativeIcon::IconView => muda::NativeIcon::IconView,
      NativeIcon::Info => muda::NativeIcon::Info,
      NativeIcon::InvalidDataFreestanding => muda::NativeIcon::InvalidDataFreestanding,
      NativeIcon::LeftFacingTriangle => muda::NativeIcon::LeftFacingTriangle,
      NativeIcon::ListView => muda::NativeIcon::ListView,
      NativeIcon::LockLocked => muda::NativeIcon::LockLocked,
      NativeIcon::LockUnlocked => muda::NativeIcon::LockUnlocked,
      NativeIcon::MenuMixedState => muda::NativeIcon::MenuMixedState,
      NativeIcon::MenuOnState => muda::NativeIcon::MenuOnState,
      NativeIcon::MobileMe => muda::NativeIcon::MobileMe,
      NativeIcon::MultipleDocuments => muda::NativeIcon::MultipleDocuments,
      NativeIcon::Network => muda::NativeIcon::Network,
      NativeIcon::Path => muda::NativeIcon::Path,
      NativeIcon::PreferencesGeneral => muda::NativeIcon::PreferencesGeneral,
      NativeIcon::QuickLook => muda::NativeIcon::QuickLook,
      NativeIcon::RefreshFreestanding => muda::NativeIcon::RefreshFreestanding,
      NativeIcon::Refresh => muda::NativeIcon::Refresh,
      NativeIcon::Remove => muda::NativeIcon::Remove,
      NativeIcon::RevealFreestanding => muda::NativeIcon::RevealFreestanding,
      NativeIcon::RightFacingTriangle => muda::NativeIcon::RightFacingTriangle,
      NativeIcon::Share => muda::NativeIcon::Share,
      NativeIcon::Slideshow => muda::NativeIcon::Slideshow,
      NativeIcon::SmartBadge => muda::NativeIcon::SmartBadge,
      NativeIcon::StatusAvailable => muda::NativeIcon::StatusAvailable,
      NativeIcon::StatusNone => muda::NativeIcon::StatusNone,
      NativeIcon::StatusPartiallyAvailable => muda::NativeIcon::StatusPartiallyAvailable,
      NativeIcon::StatusUnavailable => muda::NativeIcon::StatusUnavailable,
      NativeIcon::StopProgressFreestanding => muda::NativeIcon::StopProgressFreestanding,
      NativeIcon::StopProgress => muda::NativeIcon::StopProgress,
      NativeIcon::TrashEmpty => muda::NativeIcon::TrashEmpty,
      NativeIcon::TrashFull => muda::NativeIcon::TrashFull,
      NativeIcon::User => muda::NativeIcon::User,
      NativeIcon::UserAccounts => muda::NativeIcon::UserAccounts,
      NativeIcon::UserGroup => muda::NativeIcon::UserGroup,
      NativeIcon::UserGuest => muda::NativeIcon::UserGuest,
    }
  }
}

/// An enumeration of all menu item kinds that could be added to
/// a [`Menu`] or [`Submenu`]
pub enum MenuItemKind<R: Runtime> {
  /// Normal menu item
  MenuItem(MenuItem<R>),
  /// Submenu menu item
  Submenu(Submenu<R>),
  /// Predefined menu item
  Predefined(PredefinedMenuItem<R>),
  /// Check menu item
  Check(CheckMenuItem<R>),
  /// Icon menu item
  Icon(IconMenuItem<R>),
}

impl<R: Runtime> MenuItemKind<R> {
  /// Returns a unique identifier associated with this menu item.
  pub fn id(&self) -> &MenuId {
    match self {
      MenuItemKind::MenuItem(i) => i.id(),
      MenuItemKind::Submenu(i) => i.id(),
      MenuItemKind::Predefined(i) => i.id(),
      MenuItemKind::Check(i) => i.id(),
      MenuItemKind::Icon(i) => i.id(),
    }
  }

  pub(crate) fn inner(&self) -> &dyn IsMenuItem<R> {
    match self {
      MenuItemKind::MenuItem(i) => i,
      MenuItemKind::Submenu(i) => i,
      MenuItemKind::Predefined(i) => i,
      MenuItemKind::Check(i) => i,
      MenuItemKind::Icon(i) => i,
    }
  }

  pub(crate) fn from_muda(app_handle: AppHandle<R>, i: muda::MenuItemKind) -> Self {
    match i {
      muda::MenuItemKind::MenuItem(i) => Self::MenuItem(MenuItem(Arc::new(MenuItemInner {
        id: i.id().clone(),
        inner: i.into(),
        app_handle,
      }))),
      muda::MenuItemKind::Submenu(i) => Self::Submenu(Submenu(Arc::new(SubmenuInner {
        id: i.id().clone(),
        inner: i.into(),
        app_handle,
      }))),
      muda::MenuItemKind::Predefined(i) => {
        Self::Predefined(PredefinedMenuItem(Arc::new(PredefinedMenuItemInner {
          id: i.id().clone(),
          inner: i.into(),
          app_handle,
        })))
      }
      muda::MenuItemKind::Check(i) => Self::Check(CheckMenuItem(Arc::new(CheckMenuItemInner {
        id: i.id().clone(),
        inner: i.into(),
        app_handle,
      }))),
      muda::MenuItemKind::Icon(i) => Self::Icon(IconMenuItem(Arc::new(IconMenuItemInner {
        id: i.id().clone(),
        inner: i.into(),
        app_handle,
      }))),
    }
  }

  /// Casts this item to a [`MenuItem`], and returns `None` if it wasn't.
  pub fn as_menuitem(&self) -> Option<&MenuItem<R>> {
    match self {
      MenuItemKind::MenuItem(i) => Some(i),
      _ => None,
    }
  }

  /// Casts this item to a [`MenuItem`], and panics if it wasn't.
  pub fn as_menuitem_unchecked(&self) -> &MenuItem<R> {
    match self {
      MenuItemKind::MenuItem(i) => i,
      _ => panic!("Not a MenuItem"),
    }
  }

  /// Casts this item to a [`Submenu`], and returns `None` if it wasn't.
  pub fn as_submenu(&self) -> Option<&Submenu<R>> {
    match self {
      MenuItemKind::Submenu(i) => Some(i),
      _ => None,
    }
  }

  /// Casts this item to a [`Submenu`], and panics if it wasn't.
  pub fn as_submenu_unchecked(&self) -> &Submenu<R> {
    match self {
      MenuItemKind::Submenu(i) => i,
      _ => panic!("Not a Submenu"),
    }
  }

  /// Casts this item to a [`PredefinedMenuItem`], and returns `None` if it wasn't.
  pub fn as_predefined_menuitem(&self) -> Option<&PredefinedMenuItem<R>> {
    match self {
      MenuItemKind::Predefined(i) => Some(i),
      _ => None,
    }
  }

  /// Casts this item to a [`PredefinedMenuItem`], and panics if it wasn't.
  pub fn as_predefined_menuitem_unchecked(&self) -> &PredefinedMenuItem<R> {
    match self {
      MenuItemKind::Predefined(i) => i,
      _ => panic!("Not a PredefinedMenuItem"),
    }
  }

  /// Casts this item to a [`CheckMenuItem`], and returns `None` if it wasn't.
  pub fn as_check_menuitem(&self) -> Option<&CheckMenuItem<R>> {
    match self {
      MenuItemKind::Check(i) => Some(i),
      _ => None,
    }
  }

  /// Casts this item to a [`CheckMenuItem`], and panics if it wasn't.
  pub fn as_check_menuitem_unchecked(&self) -> &CheckMenuItem<R> {
    match self {
      MenuItemKind::Check(i) => i,
      _ => panic!("Not a CheckMenuItem"),
    }
  }

  /// Casts this item to a [`IconMenuItem`], and returns `None` if it wasn't.
  pub fn as_icon_menuitem(&self) -> Option<&IconMenuItem<R>> {
    match self {
      MenuItemKind::Icon(i) => Some(i),
      _ => None,
    }
  }

  /// Casts this item to a [`IconMenuItem`], and panics if it wasn't.
  pub fn as_icon_menuitem_unchecked(&self) -> &IconMenuItem<R> {
    match self {
      MenuItemKind::Icon(i) => i,
      _ => panic!("Not an IconMenuItem"),
    }
  }
}

impl<R: Runtime> Clone for MenuItemKind<R> {
  fn clone(&self) -> Self {
    match self {
      Self::MenuItem(i) => Self::MenuItem(i.clone()),
      Self::Submenu(i) => Self::Submenu(i.clone()),
      Self::Predefined(i) => Self::Predefined(i.clone()),
      Self::Check(i) => Self::Check(i.clone()),
      Self::Icon(i) => Self::Icon(i.clone()),
    }
  }
}

impl<R: Runtime> sealed::IsMenuItemBase for MenuItemKind<R> {
  fn inner_muda(&self) -> &dyn muda::IsMenuItem {
    self.inner().inner_muda()
  }
}

impl<R: Runtime> IsMenuItem<R> for MenuItemKind<R> {
  fn kind(&self) -> MenuItemKind<R> {
    self.clone()
  }

  fn id(&self) -> &MenuId {
    self.id()
  }
}

/// A trait that defines a generic item in a menu, which may be one of [`MenuItemKind`]
///
/// # Safety
///
/// This trait is ONLY meant to be implemented internally by the crate.
pub trait IsMenuItem<R: Runtime>: sealed::IsMenuItemBase {
  /// Returns the kind of this menu item.
  fn kind(&self) -> MenuItemKind<R>;

  /// Returns a unique identifier associated with this menu.
  fn id(&self) -> &MenuId;
}

/// A helper trait with methods to help creating a context menu.
///
/// # Safety
///
/// This trait is ONLY meant to be implemented internally by the crate.
pub trait ContextMenu: sealed::ContextMenuBase + Send + Sync {
  /// Get the popup [`HMENU`] for this menu.
  ///
  /// The returned [`HMENU`] is valid as long as the [`ContextMenu`] is.
  ///
  /// [`HMENU`]: https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#HMENU
  #[cfg(windows)]
  #[cfg_attr(docsrs, doc(cfg(windows)))]
  fn hpopupmenu(&self) -> crate::Result<isize>;

  /// Popup this menu as a context menu on the specified window at the cursor position.
  fn popup<R: crate::Runtime>(&self, window: crate::Window<R>) -> crate::Result<()>;

  /// Popup this menu as a context menu on the specified window at the specified position.
  ///
  /// The position is relative to the window's top-left corner.
  fn popup_at<R: crate::Runtime, P: Into<crate::Position>>(
    &self,
    window: crate::Window<R>,
    position: P,
  ) -> crate::Result<()>;
}

pub(crate) mod sealed {

  pub trait IsMenuItemBase {
    fn inner_muda(&self) -> &dyn muda::IsMenuItem;
  }

  pub trait ContextMenuBase {
    fn inner_context(&self) -> &dyn muda::ContextMenu;
    fn inner_context_owned(&self) -> Box<dyn muda::ContextMenu>;
    fn popup_inner<R: crate::Runtime, P: Into<crate::Position>>(
      &self,
      window: crate::Window<R>,
      position: Option<P>,
    ) -> crate::Result<()>;
  }
}

#[cfg(windows)]
pub(crate) fn map_to_menu_theme(theme: tauri_utils::Theme) -> muda::MenuTheme {
  match theme {
    tauri_utils::Theme::Light => muda::MenuTheme::Light,
    tauri_utils::Theme::Dark => muda::MenuTheme::Dark,
    _ => muda::MenuTheme::Auto,
  }
}
