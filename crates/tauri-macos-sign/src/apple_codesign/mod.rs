// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod certificate;
mod error;

pub use certificate::{CertificateProfile, create_self_signed_code_signing_certificate};
pub use error::AppleCodesignError;
