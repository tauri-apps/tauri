// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::icon::BadIcon;

#[derive(Debug, Clone)]
pub struct PlatformIcon {
    argb: Vec<u8>,
    width: i32,
    height: i32,
}

impl PlatformIcon {
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, BadIcon> {
        if !rgba.len().is_multiple_of(4) {
            return Err(BadIcon::ByteCountNotDivisibleBy4 {
                byte_count: rgba.len(),
            });
        }

        let mut bytes = rgba;
        for i in 0..(bytes.len() / 4) {
            let j = i * 4;
            let a = bytes[j + 3];
            bytes[j + 3] = bytes[j + 2];
            bytes[j + 2] = bytes[j + 1];
            bytes[j + 1] = bytes[j];
            bytes[j] = a;
        }

        Ok(Self {
            argb: bytes,
            width: width as i32,
            height: height as i32,
        })
    }

    pub fn into_rgba(self) -> (Vec<u8>, u32, u32) {
        let mut bytes = self.argb;
        for i in 0..(bytes.len() / 4) {
            let j = i * 4;
            let a = bytes[j];
            bytes[j] = bytes[j + 1];
            bytes[j + 1] = bytes[j + 2];
            bytes[j + 2] = bytes[j + 3];
            bytes[j + 3] = a;
        }

        (bytes, self.width as u32, self.height as u32)
    }
}

impl From<PlatformIcon> for ksni::Icon {
    fn from(icon: PlatformIcon) -> Self {
        ksni::Icon {
            width: icon.width,
            height: icon.height,
            data: icon.argb,
        }
    }
}
