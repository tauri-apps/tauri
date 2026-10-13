// Copyright 2018-2026 apple-codesign-rs developers <https://github.com/indygreg/apple-platform-rs>
// SPDX-License-Identifier: MPL-2.0

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use thiserror::Error;
use x509_certificate::X509CertificateError;

/// Unified error type for Apple code signing.
#[derive(Debug, Error)]
pub enum AppleCodesignError {
  #[error("X.509 certificate handler error: {0}")]
  X509(#[from] X509CertificateError),

  #[error("certificate error: {0}")]
  CertificateGeneric(String),

  #[error("the given OID does not match a recognized Apple certificate authority extension")]
  OidIsntCertificateAuthority,

  #[error("the given OID does not match a recognized Apple code signing extension")]
  OidIsntCodeSigningExtension,

  #[error("error building certificate: {0}")]
  CertificateBuildError(String),

  #[error("unknown certificate profile: {0}")]
  UnknownCertificateProfile(String),
}
