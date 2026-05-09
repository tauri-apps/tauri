// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// taken from https://github.com/rust-windowing/winit/blob/92fdf5ba85f920262a61cee4590f4a11ad5738d1/src/icon.rs

use crate::platform_impl::PlatformIcon;
use std::{error::Error, fmt, io, mem};

#[repr(C)]
#[derive(Debug)]
pub(crate) struct Pixel {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
    pub(crate) a: u8,
}

pub(crate) const PIXEL_SIZE: usize = mem::size_of::<Pixel>();

#[derive(Debug)]
/// An error produced when using [`Icon::from_rgba`] with invalid arguments.
pub enum BadIcon {
    /// Produced when the length of the `rgba` argument isn't divisible by 4, thus `rgba` can't be
    /// safely interpreted as 32bpp RGBA pixels.
    ByteCountNotDivisibleBy4 { byte_count: usize },
    /// Produced when the number of pixels (`rgba.len() / 4`) isn't equal to `width * height`.
    /// At least one of your arguments is incorrect.
    DimensionsVsPixelCount {
        width: u32,
        height: u32,
        width_x_height: usize,
        pixel_count: usize,
    },
    /// Produced when underlying OS functionality failed to create the icon
    OsError(io::Error),
}

impl fmt::Display for BadIcon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BadIcon::ByteCountNotDivisibleBy4 { byte_count } => write!(f,
                "The length of the `rgba` argument ({:?}) isn't divisible by 4, making it impossible to interpret as 32bpp RGBA pixels.",
                byte_count,
            ),
            BadIcon::DimensionsVsPixelCount {
                width,
                height,
                width_x_height,
                pixel_count,
            } => write!(f,
                "The specified dimensions ({:?}x{:?}) don't match the number of pixels supplied by the `rgba` argument ({:?}). For those dimensions, the expected pixel count is {:?}.",
                width, height, pixel_count, width_x_height,
            ),
            BadIcon::OsError(e) => write!(f, "OS error when instantiating the icon: {:?}", e),
        }
    }
}

impl Error for BadIcon {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RgbaIcon {
    pub(crate) rgba: Vec<u8>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

/// For platforms which don't have window icons (e.g. web)
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NoIcon;

#[allow(dead_code)] // These are not used on every platform
mod constructors {
    use super::*;

    impl RgbaIcon {
        pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, BadIcon> {
            if rgba.len() % PIXEL_SIZE != 0 {
                return Err(BadIcon::ByteCountNotDivisibleBy4 {
                    byte_count: rgba.len(),
                });
            }
            let pixel_count = rgba.len() / PIXEL_SIZE;
            if pixel_count != (width * height) as usize {
                Err(BadIcon::DimensionsVsPixelCount {
                    width,
                    height,
                    width_x_height: (width * height) as usize,
                    pixel_count,
                })
            } else {
                Ok(RgbaIcon {
                    rgba,
                    width,
                    height,
                })
            }
        }
    }

    impl NoIcon {
        pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, BadIcon> {
            // Create the rgba icon anyway to validate the input
            let _ = RgbaIcon::from_rgba(rgba, width, height)?;
            Ok(NoIcon)
        }
    }
}

/// An icon used for the window titlebar, taskbar, etc.
#[derive(Clone)]
pub struct Icon {
    pub(crate) inner: PlatformIcon,
}

impl fmt::Debug for Icon {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(&self.inner, formatter)
    }
}

impl Icon {
    /// Creates an icon from 32bpp RGBA data.
    ///
    /// The length of `rgba` must be divisible by 4, and `width * height` must equal
    /// `rgba.len() / 4`. Otherwise, this will return a `BadIcon` error.
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, BadIcon> {
        Ok(Icon {
            inner: PlatformIcon::from_rgba(rgba, width, height)?,
        })
    }

    /// Create an icon from a file path.
    ///
    /// Specify `size` to load a specific icon size from the file, or `None` to load the default
    /// icon size from the file.
    ///
    /// In cases where the specified size does not exist in the file, Windows may perform scaling
    /// to get an icon of the desired size.
    #[cfg(windows)]
    pub fn from_path<P: AsRef<std::path::Path>>(
        path: P,
        size: Option<(u32, u32)>,
    ) -> Result<Self, BadIcon> {
        let win_icon = PlatformIcon::from_path(path, size)?;
        Ok(Icon { inner: win_icon })
    }

    /// Create an icon from a resource embedded in this executable or library.
    ///
    /// Specify `size` to load a specific icon size from the file, or `None` to load the default
    /// icon size from the file.
    ///
    /// In cases where the specified size does not exist in the file, Windows may perform scaling
    /// to get an icon of the desired size.
    #[cfg(windows)]
    pub fn from_resource(ordinal: u16, size: Option<(u32, u32)>) -> Result<Self, BadIcon> {
        let win_icon = PlatformIcon::from_resource(ordinal, size)?;
        Ok(Icon { inner: win_icon })
    }
}

/// A native Icon to be used for the menu item
///
/// ## Platform-specific:
///
/// - **Windows / Linux**: Unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Small green indicator, similar to iChat’s available image.
    StatusAvailable,
    /// Small clear indicator.
    StatusNone,
    /// Small yellow indicator, similar to iChat’s idle image.
    StatusPartiallyAvailable,
    /// Small red indicator, similar to iChat’s unavailable image.
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
