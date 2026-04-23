// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Image types used by this crate and also referenced by the JavaScript API layer.

pub(crate) mod plugin;

use std::borrow::Cow;
use std::sync::Arc;

#[cfg(windows)]
use windows::{
  core::{Owned, PCWSTR},
  Win32::{
    Foundation::GetLastError,
    Graphics::Gdi::{
      CreateCompatibleDC, DeleteDC, GetDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
      GetIconInfo, LoadImageW, HICON, ICONINFO, IMAGE_ICON, LR_DEFAULTCOLOR,
    },
  },
};

use crate::{Resource, ResourceId, ResourceTable};

/// An RGBA Image in row-major order from top to bottom.
#[derive(Clone)]
pub struct Image<'a> {
  rgba: Cow<'a, [u8]>,
  width: u32,
  height: u32,
}

impl std::fmt::Debug for Image<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Image")
      .field(
        "rgba",
        // Reduces the debug size compared to the derived default, as the default
        // would format the raw bytes as numbers `[0, 0, 0, 0]` for 1 pixel.
        // The custom format doesn't grow as much with larger images:
        // `Image { rgba: Cow::Borrowed([u8; 4096]), width: 32, height: 32 }`
        &format_args!(
          "Cow::{}([u8; {}])",
          match &self.rgba {
            Cow::Borrowed(_) => "Borrowed",
            Cow::Owned(_) => "Owned",
          },
          self.rgba.len()
        ),
      )
      .field("width", &self.width)
      .field("height", &self.height)
      .finish()
  }
}

impl Resource for Image<'static> {}

impl Image<'static> {
  /// Creates a new Image using RGBA data, in row-major order from top to bottom, and with specified width and height.
  ///
  /// Similar to [`Self::new`] but avoids cloning the rgba data to get an owned Image.
  pub const fn new_owned(rgba: Vec<u8>, width: u32, height: u32) -> Self {
    Self {
      rgba: Cow::Owned(rgba),
      width,
      height,
    }
  }
}

impl<'a> Image<'a> {
  /// Creates a new Image using RGBA data, in row-major order from top to bottom, and with specified width and height.
  pub const fn new(rgba: &'a [u8], width: u32, height: u32) -> Self {
    Self {
      rgba: Cow::Borrowed(rgba),
      width,
      height,
    }
  }

  /// Creates a new image using the provided bytes.
  ///
  /// Only `ico` and `png` are supported (based on activated feature flag).
  #[cfg(any(feature = "image-ico", feature = "image-png"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "image-ico", feature = "image-png"))))]
  pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
    use image::GenericImageView;

    let img = image::load_from_memory(bytes)?;
    let pixels = img
      .pixels()
      .flat_map(|(_, _, pixel)| pixel.0)
      .collect::<Vec<_>>();
    Ok(Self {
      rgba: Cow::Owned(pixels),
      width: img.width(),
      height: img.height(),
    })
  }

  /// Creates a new image using the provided path.
  ///
  /// Only `ico` and `png` are supported (based on activated feature flag).
  #[cfg(any(feature = "image-ico", feature = "image-png"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "image-ico", feature = "image-png"))))]
  pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> crate::Result<Self> {
    let bytes = std::fs::read(path)?;
    Self::from_bytes(&bytes)
  }

  /// Creates a new image from the application icon embedded in this executable or library.
  ///
  /// The application icon is currently the icon with `nameID 32512` we embedded through `tauri-build`,
  /// this could change in the future.
  #[cfg(windows)]
  pub fn from_app_icon_resource(size: u32) -> crate::Result<Self> {
    // Make sure we keep this `resource_id` in sync with the one in `tauri-build`
    Image::from_icon_resource(PCWSTR(32512 as _), size, size)
  }

  /// Create a new image from an icon resource embedded in this executable or library.
  ///
  /// **Note**: This might take ~2ms for [`LoadImageW`] to load the image for the first time.
  ///
  /// ## Examples
  ///
  /// The `resource_id` can be an `u16` wrapped as `PCWSTR(1 as _)` or a wide string like `w!("icon")`
  ///
  /// ```
  /// # use tauri::image::Image;
  /// # use windows::core::{w, PCWSTR};
  /// Image::from_icon_resource(PCWSTR(1 as _), 32, 32);
  /// Image::from_icon_resource(w!("icon"), 32, 32);
  /// ```
  #[cfg(windows)]
  pub fn from_icon_resource(resource_id: PCWSTR, width: u32, height: u32) -> crate::Result<Self> {
    let width_i32 = width as i32;
    let height_i32 = height as i32;
    let color_depth_bytes = 4;

    let hicon = unsafe {
      Owned::new(HICON(
        LoadImageW(
          Some(
            GetModuleHandleW(PCWSTR::null())
              .map_err(crate::Error::ImageFromResource)?
              .into(),
          ),
          resource_id,
          IMAGE_ICON,
          width_i32,
          height_i32,
          LR_DEFAULTCOLOR,
        )
        .map_err(crate::Error::ImageFromResource)?
        .0,
      ))
    };

    let mut icon_info = ICONINFO::default();
    unsafe { GetIconInfo(*hicon, &mut icon_info).map_err(crate::Error::ImageFromResource)? };

    let image_bytes = (width_i32 * height_i32 * color_depth_bytes as i32) as usize;
    let mut bgra: Vec<u8> = Vec::with_capacity(image_bytes);

    let mut bitmap_info = BITMAPINFO::default();
    bitmap_info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as _;
    bitmap_info.bmiHeader.biWidth = width_i32;
    // nagative value for top-down
    bitmap_info.bmiHeader.biHeight = -height_i32;
    bitmap_info.bmiHeader.biBitCount = color_depth_bytes * 8;
    bitmap_info.bmiHeader.biPlanes = 1;
    bitmap_info.bmiHeader.biCompression = BI_RGB.0;

    unsafe {
      let hdc = CreateCompatibleDC(None);
      let result = GetDIBits(
        hdc,
        icon_info.hbmColor,
        0,
        height,
        Some(bgra.as_mut_ptr() as _),
        &mut bitmap_info,
        DIB_RGB_COLORS,
      );
      let _ = DeleteDC(hdc);
      if result == 0 {
        return Err(crate::Error::ImageFromResource(GetLastError().into()));
      }
      bgra.set_len(image_bytes);
    }

    let rgba = {
      for px in bgra.chunks_exact_mut(color_depth_bytes as usize) {
        // Swap Blue and Red channels
        px.swap(0, 2);
      }
      bgra
    };

    Ok(Image::new_owned(rgba, width, height))
  }

  /// Returns the RGBA data for this image, in row-major order from top to bottom.
  pub fn rgba(&'a self) -> &'a [u8] {
    &self.rgba
  }

  /// Returns the width of this image.
  pub fn width(&self) -> u32 {
    self.width
  }

  /// Returns the height of this image.
  pub fn height(&self) -> u32 {
    self.height
  }

  /// Convert into a 'static owned [`Image`].
  /// This will allocate.
  pub fn to_owned(self) -> Image<'static> {
    Image {
      rgba: match self.rgba {
        Cow::Owned(v) => Cow::Owned(v),
        Cow::Borrowed(v) => Cow::Owned(v.to_vec()),
      },
      height: self.height,
      width: self.width,
    }
  }
}

impl<'a> From<Image<'a>> for crate::runtime::Icon<'a> {
  fn from(img: Image<'a>) -> Self {
    Self {
      rgba: img.rgba,
      width: img.width,
      height: img.height,
    }
  }
}

#[cfg(desktop)]
impl TryFrom<Image<'_>> for muda::Icon {
  type Error = crate::Error;

  fn try_from(img: Image<'_>) -> Result<Self, Self::Error> {
    muda::Icon::from_rgba(img.rgba.to_vec(), img.width, img.height).map_err(Into::into)
  }
}

#[cfg(all(desktop, feature = "tray-icon"))]
impl TryFrom<Image<'_>> for tray_icon::Icon {
  type Error = crate::Error;

  fn try_from(img: Image<'_>) -> Result<Self, Self::Error> {
    tray_icon::Icon::from_rgba(img.rgba.to_vec(), img.width, img.height).map_err(Into::into)
  }
}

/// An image type that accepts file paths, raw bytes, previously loaded images and image objects.
///
/// This type is meant to be used along the [transformImage](https://v2.tauri.app/reference/javascript/api/namespaceimage/#transformimage) API.
///
/// # Stability
///
/// The stability of the variants are not guaranteed, and matching against them is not recommended.
/// Use [`JsImage::into_img`] instead.
#[derive(serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum JsImage {
  /// A reference to a image in the filesystem.
  #[non_exhaustive]
  Path(std::path::PathBuf),
  /// Image from raw bytes.
  #[non_exhaustive]
  Bytes(Vec<u8>),
  /// An image that was previously loaded with the API and is stored in the resource table.
  #[non_exhaustive]
  Resource(ResourceId),
  /// Raw RGBA definition of an image.
  #[non_exhaustive]
  Rgba {
    /// Image bytes.
    rgba: Vec<u8>,
    /// Image width.
    width: u32,
    /// Image height.
    height: u32,
  },
}

impl JsImage {
  /// Converts this intermediate image format into an actual [`Image`].
  ///
  /// This will retrieve the image from the passed [`ResourceTable`] if it is [`JsImage::Resource`]
  /// and will return an error if it doesn't exist in the passed [`ResourceTable`] so make sure
  /// the passed [`ResourceTable`] is the same one used to store the image, usually this should be
  /// the webview [resources table](crate::webview::Webview::resources_table).
  pub fn into_img(self, resources_table: &ResourceTable) -> crate::Result<Arc<Image<'_>>> {
    match self {
      Self::Resource(rid) => resources_table.get::<Image<'static>>(rid),
      #[cfg(any(feature = "image-ico", feature = "image-png"))]
      Self::Path(path) => Image::from_path(path).map(Arc::new),

      #[cfg(any(feature = "image-ico", feature = "image-png"))]
      Self::Bytes(bytes) => Image::from_bytes(&bytes).map(Arc::new),

      Self::Rgba {
        rgba,
        width,
        height,
      } => Ok(Arc::new(Image::new_owned(rgba, width, height))),

      #[cfg(not(any(feature = "image-ico", feature = "image-png")))]
      _ => Err(
        std::io::Error::new(
          std::io::ErrorKind::InvalidInput,
          format!(
            "expected RGBA image data, found {}",
            match self {
              JsImage::Path(_) => "a file path",
              JsImage::Bytes(_) => "raw bytes",
              _ => unreachable!(),
            }
          ),
        )
        .into(),
      ),
    }
  }
}
