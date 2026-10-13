// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0
//
// Vendored from apple-codesign 0.29.0 <https://github.com/indygreg/apple-platform-rs>,
// Copyright Gregory Szorc and contributors. Modified by the Tauri Programme
// within The Commons Conservancy. See README.md in this directory.

//! Functionality related to certificates.

use {
  super::error::AppleCodesignError,
  bcder::{
    ConstOid, Oid,
    encode::{PrimitiveContent, Values},
  },
  bytes::Bytes,
  pkcs8::EncodePrivateKey,
  std::{
    fmt::{Display, Formatter},
    str::FromStr,
  },
  x509_certificate::{
    CapturedX509Certificate, InMemorySigningKeyPair, KeyAlgorithm, X509CertificateBuilder,
    certificate::KeyUsage, rfc4519::OID_COUNTRY_NAME,
  },
};

/// Extended Key Usage extension.
///
/// 2.5.29.37
const OID_EXTENDED_KEY_USAGE: ConstOid = Oid(&[85, 29, 37]);

/// Extended Key Usage purpose for code signing.
///
/// 1.3.6.1.5.5.7.3.3
const OID_EKU_PURPOSE_CODE_SIGNING: ConstOid = Oid(&[43, 6, 1, 5, 5, 7, 3, 3]);

/// Extended Key Usage for purpose of `Safari Developer`.
///
/// 1.2.840.113635.100.4.8
const OID_EKU_PURPOSE_SAFARI_DEVELOPER: ConstOid = Oid(&[42, 134, 72, 134, 247, 99, 100, 4, 8]);

/// Extended Key Usage for purpose of `3rd Party Mac Developer Installer`.
///
/// 1.2.840.113635.100.4.9
const OID_EKU_PURPOSE_3RD_PARTY_MAC_DEVELOPER_INSTALLER: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 4, 9]);

/// Extended Key Usage for purpose of `Developer ID Installer`.
///
/// 1.2.840.113635.100.4.13
const OID_EKU_PURPOSE_DEVELOPER_ID_INSTALLER: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 4, 13]);

/// Extension for `Apple Signing`.
///
/// 1.2.840.113635.100.6.1.1
const OID_EXTENSION_APPLE_SIGNING: ConstOid = Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 1]);

/// Extension for `iPhone Developer`.
///
/// 1.2.840.113635.100.6.1.2
const OID_EXTENSION_IPHONE_DEVELOPER: ConstOid = Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 2]);

/// Extension for `Apple iPhone OS Application Signing`
///
/// 1.2.840.113635.100.6.1.3
const OID_EXTENSION_IPHONE_OS_APPLICATION_SIGNING: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 3]);

/// Extension for `Apple Developer Certificate (Submission)`.
///
/// May also be referred to as `iPhone Distribution`.
///
/// 1.2.840.113635.100.6.1.4
const OID_EXTENSION_APPLE_DEVELOPER_CERTIFICATE_SUBMISSION: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 4]);

/// Extension for `Safari Developer`.
///
/// 1.2.840.113635.100.6.1.5
const OID_EXTENSION_SAFARI_DEVELOPER: ConstOid = Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 5]);

/// Extension for `Apple iPhone OS VPN Signing`
///
/// 1.2.840.113635.100.6.1.6
const OID_EXTENSION_IPHONE_OS_VPN_SIGNING: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 6]);

/// Extension for `Apple Mac App Signing (Development)`.
///
/// May also appear as `3rd Party Mac Developer Application`.
///
/// 1.2.840.113635.100.6.1.7
const OID_EXTENSION_APPLE_MAC_APP_SIGNING_DEVELOPMENT: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 7]);

/// Extension for `Apple Mac App Signing Submission`.
///
/// 1.2.840.113635.100.6.1.8
const OID_EXTENSION_APPLE_MAC_APP_SIGNING_SUBMISSION: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 8]);

/// Extension for `Mac App Store Code Signing`.
///
/// 1.2.840.113635.100.6.1.9
const OID_EXTENSION_APPLE_MAC_APP_STORE_CODE_SIGNING: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 9]);

/// Extension for `Mac App Store Installer Signing`.
///
/// 1.2.840.113635.100.6.1.10
const OID_EXTENSION_APPLE_MAC_APP_STORE_INSTALLER_SIGNING: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 10]);

// 1.2.840.113635.100.6.1.11 is unknown.

/// Extension for `Mac Developer`.
///
/// 1.2.840.113635.100.6.1.12
const OID_EXTENSION_MAC_DEVELOPER: ConstOid = Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 12]);

/// Extension for `Developer ID Application`.
///
/// 1.2.840.113635.100.6.1.13
const OID_EXTENSION_DEVELOPER_ID_APPLICATION: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 13]);

/// Extension for `Developer ID Installer`.
///
/// 1.2.840.113635.100.6.1.14
const OID_EXTENSION_DEVELOPER_ID_INSTALLER: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 14]);

// 1.2.840.113635.100.6.1.15 looks to have something to do with core OS functionality,
// as it appears in search results for hacking Apple OS booting.

/// Extension for `Apple Pay Passbook Signing`
///
/// 1.2.840.113635.100.6.1.16
const OID_EXTENSION_PASSBOOK_SIGNING: ConstOid = Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 16]);

/// Extension for `Web Site Push Notifications Signing`
///
/// 1.2.840.113635.100.6.1.17
const OID_EXTENSION_WEBSITE_PUSH_NOTIFICATION_SIGNING: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 17]);

/// Extension for `Developer ID Kernel`.
///
/// 1.2.840.113635.100.6.1.18
const OID_EXTENSION_DEVELOPER_ID_KERNEL: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 18]);

/// Extension for `Developer ID Date`.
///
/// This OID doesn't have a description in Apple tooling. But it
/// holds a UtcDate (with hours, minutes, and seconds all set to 0) and seems to
/// denote a date constraint to apply to validation. This is likely used
/// to validating timestamping constrains for certificate validity.
///
/// 1.2.840.113635.100.6.1.33
const OID_EXTENSION_DEVELOPER_ID_DATE: ConstOid = Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 33]);

/// Extension for `TestFlight`.
///
/// 1.2.840.113635.100.6.1.25.1
const OID_EXTENSION_TEST_FLIGHT: ConstOid = Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 1, 25, 1]);

/// UserID.
///
/// 0.9.2342.19200300.100.1.1
pub const OID_USER_ID: ConstOid = Oid(&[9, 146, 38, 137, 147, 242, 44, 100, 1, 1]);

/// Describes the type of code signing that a certificate is authorized to perform.
///
/// Code signing certificates are issued with extended key usage (EKU) attributes
/// denoting what that certificate will be used for. They basically say *I'm authorized
/// to sign X*.
///
/// This type describes the different code signing key usages defined on Apple
/// platforms.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtendedKeyUsagePurpose {
  /// Code signing.
  CodeSigning,

  /// Safari Developer.
  SafariDeveloper,

  /// 3rd Party Mac Developer Installer Packaging Signing.
  ///
  /// The certificate can be used to sign Mac installer packages.
  ThirdPartyMacDeveloperInstaller,

  /// Developer ID Installer.
  DeveloperIdInstaller,
}

impl ExtendedKeyUsagePurpose {
  pub fn as_oid(&self) -> ConstOid {
    match self {
      Self::CodeSigning => OID_EKU_PURPOSE_CODE_SIGNING,
      Self::SafariDeveloper => OID_EKU_PURPOSE_SAFARI_DEVELOPER,
      Self::ThirdPartyMacDeveloperInstaller => OID_EKU_PURPOSE_3RD_PARTY_MAC_DEVELOPER_INSTALLER,
      Self::DeveloperIdInstaller => OID_EKU_PURPOSE_DEVELOPER_ID_INSTALLER,
    }
  }
}

impl Display for ExtendedKeyUsagePurpose {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      ExtendedKeyUsagePurpose::CodeSigning => f.write_str("Code Signing"),
      ExtendedKeyUsagePurpose::SafariDeveloper => f.write_str("Safari Developer"),
      ExtendedKeyUsagePurpose::ThirdPartyMacDeveloperInstaller => {
        f.write_str("3rd Party Mac Developer Installer Packaging Signing")
      }
      ExtendedKeyUsagePurpose::DeveloperIdInstaller => f.write_str("Developer ID Installer"),
    }
  }
}

impl TryFrom<&Oid> for ExtendedKeyUsagePurpose {
  type Error = AppleCodesignError;

  fn try_from(oid: &Oid) -> Result<Self, Self::Error> {
    // Surely there is a way to use `match`. But the `Oid` type is a bit wonky.
    if oid.as_ref() == OID_EKU_PURPOSE_CODE_SIGNING.as_ref() {
      Ok(Self::CodeSigning)
    } else if oid.as_ref() == OID_EKU_PURPOSE_SAFARI_DEVELOPER.as_ref() {
      Ok(Self::SafariDeveloper)
    } else if oid.as_ref() == OID_EKU_PURPOSE_3RD_PARTY_MAC_DEVELOPER_INSTALLER.as_ref() {
      Ok(Self::ThirdPartyMacDeveloperInstaller)
    } else if oid.as_ref() == OID_EKU_PURPOSE_DEVELOPER_ID_INSTALLER.as_ref() {
      Ok(Self::DeveloperIdInstaller)
    } else {
      Err(AppleCodesignError::OidIsntCertificateAuthority)
    }
  }
}

/// Describes one of the many X.509 certificate extensions found on Apple code signing certificates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodeSigningCertificateExtension {
  /// Apple Signing.
  ///
  /// (Appears to be deprecated).
  AppleSigning,

  /// iPhone Developer.
  IPhoneDeveloper,

  /// Apple iPhone OS Application Signing.
  IPhoneOsApplicationSigning,

  /// Apple Developer Certificate (Submission).
  ///
  /// May also be referred to as `iPhone Distribution`.
  AppleDeveloperCertificateSubmission,

  /// Safari Developer.
  SafariDeveloper,

  /// Apple iPhone OS VPN Signing.
  IPhoneOsVpnSigning,

  /// Apple Mac App Signing (Development).
  ///
  /// Also known as `3rd Party Mac Developer Application`.
  AppleMacAppSigningDevelopment,

  /// Apple Mac App Signing Submission.
  AppleMacAppSigningSubmission,

  /// Mac App Store Code Signing.
  AppleMacAppStoreCodeSigning,

  /// Mac App Store Installer Signing.
  AppleMacAppStoreInstallerSigning,

  /// Mac Developer.
  MacDeveloper,

  /// Developer ID Application.
  DeveloperIdApplication,

  /// Developer ID Date.
  DeveloperIdDate,

  /// Developer ID Installer.
  DeveloperIdInstaller,

  /// Apple Pay Passbook Signing.
  ApplePayPassbookSigning,

  /// Web Site Push Notifications Signing.
  WebsitePushNotificationSigning,

  /// Developer ID Kernel.
  DeveloperIdKernel,

  /// TestFlight.
  TestFlight,
}

impl CodeSigningCertificateExtension {
  pub fn as_oid(&self) -> ConstOid {
    match self {
      Self::AppleSigning => OID_EXTENSION_APPLE_SIGNING,
      Self::IPhoneDeveloper => OID_EXTENSION_IPHONE_DEVELOPER,
      Self::IPhoneOsApplicationSigning => OID_EXTENSION_IPHONE_OS_APPLICATION_SIGNING,
      Self::AppleDeveloperCertificateSubmission => {
        OID_EXTENSION_APPLE_DEVELOPER_CERTIFICATE_SUBMISSION
      }
      Self::SafariDeveloper => OID_EXTENSION_SAFARI_DEVELOPER,
      Self::IPhoneOsVpnSigning => OID_EXTENSION_IPHONE_OS_VPN_SIGNING,
      Self::AppleMacAppSigningDevelopment => OID_EXTENSION_APPLE_MAC_APP_SIGNING_DEVELOPMENT,
      Self::AppleMacAppSigningSubmission => OID_EXTENSION_APPLE_MAC_APP_SIGNING_SUBMISSION,
      Self::AppleMacAppStoreCodeSigning => OID_EXTENSION_APPLE_MAC_APP_STORE_CODE_SIGNING,
      Self::AppleMacAppStoreInstallerSigning => OID_EXTENSION_APPLE_MAC_APP_STORE_INSTALLER_SIGNING,
      Self::MacDeveloper => OID_EXTENSION_MAC_DEVELOPER,
      Self::DeveloperIdApplication => OID_EXTENSION_DEVELOPER_ID_APPLICATION,
      Self::DeveloperIdDate => OID_EXTENSION_DEVELOPER_ID_DATE,
      Self::DeveloperIdInstaller => OID_EXTENSION_DEVELOPER_ID_INSTALLER,
      Self::ApplePayPassbookSigning => OID_EXTENSION_PASSBOOK_SIGNING,
      Self::WebsitePushNotificationSigning => OID_EXTENSION_WEBSITE_PUSH_NOTIFICATION_SIGNING,
      Self::DeveloperIdKernel => OID_EXTENSION_DEVELOPER_ID_KERNEL,
      Self::TestFlight => OID_EXTENSION_TEST_FLIGHT,
    }
  }
}

impl Display for CodeSigningCertificateExtension {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      CodeSigningCertificateExtension::AppleSigning => f.write_str("Apple Signing"),
      CodeSigningCertificateExtension::IPhoneDeveloper => f.write_str("iPhone Developer"),
      CodeSigningCertificateExtension::IPhoneOsApplicationSigning => {
        f.write_str("Apple iPhone OS Application Signing")
      }
      CodeSigningCertificateExtension::AppleDeveloperCertificateSubmission => {
        f.write_str("Apple Developer Certificate (Submission)")
      }
      CodeSigningCertificateExtension::SafariDeveloper => f.write_str("Safari Developer"),
      CodeSigningCertificateExtension::IPhoneOsVpnSigning => {
        f.write_str("Apple iPhone OS VPN Signing")
      }
      CodeSigningCertificateExtension::AppleMacAppSigningDevelopment => {
        f.write_str("Apple Mac App Signing (Development)")
      }
      CodeSigningCertificateExtension::AppleMacAppSigningSubmission => {
        f.write_str("Apple Mac App Signing Submission")
      }
      CodeSigningCertificateExtension::AppleMacAppStoreCodeSigning => {
        f.write_str("Mac App Store Code Signing")
      }
      CodeSigningCertificateExtension::AppleMacAppStoreInstallerSigning => {
        f.write_str("Mac App Store Installer Signing")
      }
      CodeSigningCertificateExtension::MacDeveloper => f.write_str("Mac Developer"),
      CodeSigningCertificateExtension::DeveloperIdApplication => {
        f.write_str("Developer ID Application")
      }
      CodeSigningCertificateExtension::DeveloperIdDate => f.write_str("Developer ID Date"),
      CodeSigningCertificateExtension::DeveloperIdInstaller => {
        f.write_str("Developer ID Installer")
      }
      CodeSigningCertificateExtension::ApplePayPassbookSigning => {
        f.write_str("Apple Pay Passbook Signing")
      }
      CodeSigningCertificateExtension::WebsitePushNotificationSigning => {
        f.write_str("Web Site Push Notifications Signing")
      }
      CodeSigningCertificateExtension::DeveloperIdKernel => f.write_str("Developer ID Kernel"),
      CodeSigningCertificateExtension::TestFlight => f.write_str("TestFlight"),
    }
  }
}

impl TryFrom<&Oid> for CodeSigningCertificateExtension {
  type Error = AppleCodesignError;

  fn try_from(oid: &Oid) -> Result<Self, Self::Error> {
    // Surely there is a way to use `match`. But the `Oid` type is a bit wonky.
    let o = oid.as_ref();

    if o == OID_EXTENSION_APPLE_SIGNING.as_ref() {
      Ok(Self::AppleSigning)
    } else if o == OID_EXTENSION_IPHONE_DEVELOPER.as_ref() {
      Ok(Self::IPhoneDeveloper)
    } else if o == OID_EXTENSION_IPHONE_OS_APPLICATION_SIGNING.as_ref() {
      Ok(Self::IPhoneOsApplicationSigning)
    } else if o == OID_EXTENSION_APPLE_DEVELOPER_CERTIFICATE_SUBMISSION.as_ref() {
      Ok(Self::AppleDeveloperCertificateSubmission)
    } else if o == OID_EXTENSION_SAFARI_DEVELOPER.as_ref() {
      Ok(Self::SafariDeveloper)
    } else if o == OID_EXTENSION_IPHONE_OS_VPN_SIGNING.as_ref() {
      Ok(Self::IPhoneOsVpnSigning)
    } else if o == OID_EXTENSION_APPLE_MAC_APP_SIGNING_DEVELOPMENT.as_ref() {
      Ok(Self::AppleMacAppSigningDevelopment)
    } else if o == OID_EXTENSION_APPLE_MAC_APP_SIGNING_SUBMISSION.as_ref() {
      Ok(Self::AppleMacAppSigningSubmission)
    } else if o == OID_EXTENSION_APPLE_MAC_APP_STORE_CODE_SIGNING.as_ref() {
      Ok(Self::AppleMacAppStoreCodeSigning)
    } else if o == OID_EXTENSION_APPLE_MAC_APP_STORE_INSTALLER_SIGNING.as_ref() {
      Ok(Self::AppleMacAppStoreInstallerSigning)
    } else if o == OID_EXTENSION_MAC_DEVELOPER.as_ref() {
      Ok(Self::MacDeveloper)
    } else if o == OID_EXTENSION_DEVELOPER_ID_APPLICATION.as_ref() {
      Ok(Self::DeveloperIdApplication)
    } else if o == OID_EXTENSION_DEVELOPER_ID_INSTALLER.as_ref() {
      Ok(Self::DeveloperIdInstaller)
    } else if o == OID_EXTENSION_PASSBOOK_SIGNING.as_ref() {
      Ok(Self::ApplePayPassbookSigning)
    } else if o == OID_EXTENSION_WEBSITE_PUSH_NOTIFICATION_SIGNING.as_ref() {
      Ok(Self::WebsitePushNotificationSigning)
    } else if o == OID_EXTENSION_DEVELOPER_ID_KERNEL.as_ref() {
      Ok(Self::DeveloperIdKernel)
    } else if o == OID_EXTENSION_DEVELOPER_ID_DATE.as_ref() {
      Ok(Self::DeveloperIdDate)
    } else if o == OID_EXTENSION_TEST_FLIGHT.as_ref() {
      Ok(Self::TestFlight)
    } else {
      Err(AppleCodesignError::OidIsntCodeSigningExtension)
    }
  }
}

/// Describes combinations of certificate extensions for Apple code signing certificates.
///
/// Code signing certificates contain various X.509 extensions denoting them for
/// code signing.
///
/// This type represents various common extensions as used on Apple platforms.
///
/// Typically, you'll want to apply at most one of these extensions to a
/// new certificate in order to mark it as compatible for code signing.
///
/// This type essentially encapsulates the logic for handling of different
/// "profiles" attached to the different code signing certificates that Apple
/// issues.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertificateProfile {
  /// Mac Installer Distribution.
  ///
  /// In `Keychain Access.app`, this might render as `3rd Party Mac Developer Installer`.
  ///
  /// Certificates are marked for EKU with `3rd Party Developer Installer Package
  /// Signing`.
  ///
  /// They also have the `Apple Mac App Signing (Submission)` extension.
  ///
  /// Typically issued by `Apple Worldwide Developer Relations Certificate
  /// Authority`.
  MacInstallerDistribution,

  /// Apple Distribution.
  ///
  /// Certificates are marked for EKU with `Code Signing`. They also have
  /// extensions `Apple Mac App Signing (Development)` and
  /// `Apple Developer Certificate (Submission)`.
  ///
  /// Typically issued by `Apple Worldwide Developer Relations Certificate Authority`.
  AppleDistribution,

  /// Apple Development.
  ///
  /// Certificates are marked for EKU with `Code Signing`. They also have
  /// extensions `Apple Developer Certificate (Development)` and
  /// `Mac Developer`.
  ///
  /// Typically issued by `Apple Worldwide Developer Relations Certificate
  /// Authority`.
  AppleDevelopment,

  /// Developer ID Application.
  ///
  /// Certificates are marked for EKU with `Code Signing`. They also have
  /// extensions for `Developer ID Application` and `Developer ID Date`.
  DeveloperIdApplication,

  /// Developer ID Installer.
  ///
  /// Certificates are marked for EKU with `Developer ID Application`. They also
  /// have extensions `Developer ID Installer` and `Developer ID Date`.
  DeveloperIdInstaller,
}

impl CertificateProfile {
  pub fn all() -> &'static [Self] {
    &[
      Self::MacInstallerDistribution,
      Self::AppleDistribution,
      Self::AppleDevelopment,
      Self::DeveloperIdApplication,
      Self::DeveloperIdInstaller,
    ]
  }

  /// Obtain the string values that variants are recognized as.
  pub fn str_names() -> [&'static str; 5] {
    [
      "mac-installer-distribution",
      "apple-distribution",
      "apple-development",
      "developer-id-application",
      "developer-id-installer",
    ]
  }
}

impl Display for CertificateProfile {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      CertificateProfile::MacInstallerDistribution => f.write_str("mac-installer-distribution"),
      CertificateProfile::AppleDistribution => f.write_str("apple-distribution"),
      CertificateProfile::AppleDevelopment => f.write_str("apple-development"),
      CertificateProfile::DeveloperIdApplication => f.write_str("developer-id-application"),
      CertificateProfile::DeveloperIdInstaller => f.write_str("developer-id-installer"),
    }
  }
}

impl FromStr for CertificateProfile {
  type Err = AppleCodesignError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "apple-distribution" => Ok(Self::AppleDistribution),
      "apple-development" => Ok(Self::AppleDevelopment),
      "developer-id-application" => Ok(Self::DeveloperIdApplication),
      "developer-id-installer" => Ok(Self::DeveloperIdInstaller),
      "mac-installer-distribution" => Ok(Self::MacInstallerDistribution),
      _ => Err(AppleCodesignError::UnknownCertificateProfile(s.to_string())),
    }
  }
}

/// Extensions to [X509CertificateBuilder] specializing in Apple certificate behavior.
///
/// Most callers should call [Self::apple_certificate_profile] to configure
/// a preset profile for the certificate being generated. After that - and it is
/// important it is after - call [Self::apple_subject] to define the subject
/// field. If you call this after registering code signing extensions, it
/// detects the appropriate format for the Common Name field.
pub trait AppleCertificateBuilder: Sized {
  /// This functions defines common attributes on the certificate subject.
  ///
  /// `team_id` is your Apple team id. It is a short alphanumeric string. You
  /// can find this at <https://developer.apple.com/account/#/membership/>.
  fn apple_subject(
    &mut self,
    team_id: &str,
    person_name: &str,
    country: &str,
  ) -> Result<(), AppleCodesignError>;

  /// Add an [ExtendedKeyUsagePurpose] to this certificate.
  fn apple_extended_key_usage(
    &mut self,
    usage: ExtendedKeyUsagePurpose,
  ) -> Result<(), AppleCodesignError>;

  /// Add a certificate extension as defined by a [CodeSigningCertificateExtension] instance.
  fn apple_code_signing_certificate_extension(
    &mut self,
    extension: CodeSigningCertificateExtension,
  ) -> Result<(), AppleCodesignError>;

  /// Add a [CertificateProfile] to this builder.
  ///
  /// All certificate extensions relevant to this profile are added.
  ///
  /// This should be the first function you call after creating an instance
  /// because other functions rely on the state that it sets.
  fn apple_certificate_profile(
    &mut self,
    profile: CertificateProfile,
  ) -> Result<(), AppleCodesignError>;

  /// Find code signing extensions that are currently registered.
  fn apple_code_signing_extensions(&self) -> Vec<CodeSigningCertificateExtension>;
}

impl AppleCertificateBuilder for X509CertificateBuilder {
  fn apple_subject(
    &mut self,
    team_id: &str,
    person_name: &str,
    country: &str,
  ) -> Result<(), AppleCodesignError> {
    // TODO the subject schema here isn't totally accurate. While OU does always
    // appear to be the team id, the user id attribute can be something else.
    // For example, for Apple Development, there are a similarly formatted yet
    // different value. But the team id does still appear.
    self
      .subject()
      .append_utf8_string(Oid(OID_USER_ID.as_ref().into()), team_id)
      .map_err(|e| AppleCodesignError::CertificateBuildError(format!("{e:?}")))?;

    // Common Name is derived from the profile in use.

    let extensions = self.apple_code_signing_extensions();

    let common_name = if extensions
      .contains(&CodeSigningCertificateExtension::DeveloperIdApplication)
    {
      format!("Developer ID Application: {person_name} ({team_id})")
    } else if extensions.contains(&CodeSigningCertificateExtension::DeveloperIdInstaller) {
      format!("Developer ID Installer: {person_name} ({team_id})")
    } else if extensions
      .contains(&CodeSigningCertificateExtension::AppleDeveloperCertificateSubmission)
    {
      format!("Apple Distribution: {person_name} ({team_id})")
    } else if extensions.contains(&CodeSigningCertificateExtension::AppleMacAppSigningSubmission) {
      format!("3rd Party Mac Developer Installer: {person_name} ({team_id})")
    } else if extensions.contains(&CodeSigningCertificateExtension::MacDeveloper) {
      format!("Apple Development: {person_name} ({team_id})")
    } else {
      format!("{person_name} ({team_id})")
    };

    self
      .subject()
      .append_common_name_utf8_string(&common_name)
      .map_err(|e| AppleCodesignError::CertificateBuildError(format!("{e:?}")))?;

    self
      .subject()
      .append_organizational_unit_utf8_string(team_id)
      .map_err(|e| AppleCodesignError::CertificateBuildError(format!("{e:?}")))?;

    self
      .subject()
      .append_organization_utf8_string(person_name)
      .map_err(|e| AppleCodesignError::CertificateBuildError(format!("{e:?}")))?;

    self
      .subject()
      .append_printable_string(Oid(OID_COUNTRY_NAME.as_ref().into()), country)
      .map_err(|e| AppleCodesignError::CertificateBuildError(format!("{e:?}")))?;

    Ok(())
  }

  fn apple_extended_key_usage(
    &mut self,
    usage: ExtendedKeyUsagePurpose,
  ) -> Result<(), AppleCodesignError> {
    let payload =
      bcder::encode::sequence(Oid(Bytes::copy_from_slice(usage.as_oid().as_ref())).encode())
        .to_captured(bcder::Mode::Der);

    self.add_extension_der_data(
      Oid(OID_EXTENDED_KEY_USAGE.as_ref().into()),
      true,
      payload.as_slice(),
    );

    Ok(())
  }

  fn apple_code_signing_certificate_extension(
    &mut self,
    extension: CodeSigningCertificateExtension,
  ) -> Result<(), AppleCodesignError> {
    let (critical, payload) = match extension {
      CodeSigningCertificateExtension::IPhoneDeveloper => {
        // SEQUENCE (3 elem)
        //   OBJECT IDENTIFIER 1.2.840.113635.100.6.1.2
        //   BOOLEAN true
        //   OCTET STRING (2 byte) 0500
        //     NULL
        (true, Bytes::copy_from_slice(&[0x05, 0x00]))
      }
      CodeSigningCertificateExtension::AppleDeveloperCertificateSubmission => {
        // SEQUENCE (3 elem)
        //   OBJECT IDENTIFIER 1.2.840.113635.100.6.1.4
        //   BOOLEAN true
        //   OCTET STRING (2 byte) 0500
        //     NULL
        (true, Bytes::copy_from_slice(&[0x05, 0x00]))
      }
      CodeSigningCertificateExtension::AppleMacAppSigningDevelopment => {
        // SEQUENCE (3 elem)
        //   OBJECT IDENTIFIER 1.2.840.113635.100.6.1.7
        //   BOOLEAN true
        //   OCTET STRING (2 byte) 0500
        //     NULL
        (true, Bytes::copy_from_slice(&[0x05, 0x00]))
      }
      CodeSigningCertificateExtension::AppleMacAppSigningSubmission => {
        // SEQUENCE (3 elem)
        //   OBJECT IDENTIFIER 1.2.840.113635.100.6.1.8
        //   BOOLEAN true
        //   OCTET STRING (2 byte) 0500
        //   NULL
        (true, Bytes::copy_from_slice(&[0x05, 0x00]))
      }
      CodeSigningCertificateExtension::MacDeveloper => {
        // SEQUENCE (3 elem)
        //   OBJECT IDENTIFIER 1.2.840.113635.100.6.1.12
        //   BOOLEAN true
        //   OCTET STRING (2 byte) 0500
        //     NULL
        (true, Bytes::copy_from_slice(&[0x05, 0x00]))
      }
      CodeSigningCertificateExtension::DeveloperIdApplication => {
        // SEQUENCE (3 elem)
        //   OBJECT IDENTIFIER 1.2.840.113635.100.6.1.13
        //   BOOLEAN true
        //   OCTET STRING (2 byte) 0500
        //     NULL
        (true, Bytes::copy_from_slice(&[0x05, 0x00]))
      }
      CodeSigningCertificateExtension::DeveloperIdInstaller => {
        // SEQUENCE (3 elem)
        //   OBJECT IDENTIFIER 1.2.840.113635.100.6.1.14
        //   BOOLEAN true
        //   OCTET STRING (2 byte) 0500
        //   NULL
        (true, Bytes::copy_from_slice(&[0x05, 0x00]))
      }

      // The rest of these probably have the same payload. But until we see
      // them, don't take chances.
      _ => {
        return Err(AppleCodesignError::CertificateBuildError(format!(
          "don't know how to handle code signing extension {extension:?}"
        )));
      }
    };

    self.add_extension_der_data(
      Oid(Bytes::copy_from_slice(extension.as_oid().as_ref())),
      critical,
      payload,
    );

    Ok(())
  }

  fn apple_certificate_profile(
    &mut self,
    profile: CertificateProfile,
  ) -> Result<(), AppleCodesignError> {
    // Try to keep this logic in sync with apple_guess_profile().
    match profile {
      CertificateProfile::DeveloperIdApplication => {
        self.constraint_not_ca();
        self.apple_extended_key_usage(ExtendedKeyUsagePurpose::CodeSigning)?;
        self.key_usage(KeyUsage::DigitalSignature);

        // OID_EXTENSION_DEVELOPER_ID_DATE comes next. But we don't know what
        // that should be. It is a UTF8String instead of an ASN.1 time type
        // because who knows.

        self.apple_code_signing_certificate_extension(
          CodeSigningCertificateExtension::DeveloperIdApplication,
        )?;
      }
      CertificateProfile::DeveloperIdInstaller => {
        self.constraint_not_ca();
        self.apple_extended_key_usage(ExtendedKeyUsagePurpose::DeveloperIdInstaller)?;
        self.key_usage(KeyUsage::DigitalSignature);

        // OID_EXTENSION_DEVELOPER_ID_DATE comes next.

        self.apple_code_signing_certificate_extension(
          CodeSigningCertificateExtension::DeveloperIdInstaller,
        )?;
      }
      CertificateProfile::AppleDevelopment => {
        self.constraint_not_ca();
        self.apple_extended_key_usage(ExtendedKeyUsagePurpose::CodeSigning)?;
        self.key_usage(KeyUsage::DigitalSignature);
        self.apple_code_signing_certificate_extension(
          CodeSigningCertificateExtension::IPhoneDeveloper,
        )?;
        self.apple_code_signing_certificate_extension(
          CodeSigningCertificateExtension::MacDeveloper,
        )?;
      }
      CertificateProfile::AppleDistribution => {
        self.constraint_not_ca();
        self.apple_extended_key_usage(ExtendedKeyUsagePurpose::CodeSigning)?;
        self.key_usage(KeyUsage::DigitalSignature);

        // OID_EXTENSION_DEVELOPER_ID_DATE comes next.

        self.apple_code_signing_certificate_extension(
          CodeSigningCertificateExtension::AppleMacAppSigningDevelopment,
        )?;
        self.apple_code_signing_certificate_extension(
          CodeSigningCertificateExtension::AppleDeveloperCertificateSubmission,
        )?;
      }
      CertificateProfile::MacInstallerDistribution => {
        self.constraint_not_ca();
        self.apple_extended_key_usage(ExtendedKeyUsagePurpose::ThirdPartyMacDeveloperInstaller)?;
        self.key_usage(KeyUsage::DigitalSignature);

        self.apple_code_signing_certificate_extension(
          CodeSigningCertificateExtension::AppleMacAppSigningSubmission,
        )?;
      }
    }

    Ok(())
  }

  fn apple_code_signing_extensions(&self) -> Vec<CodeSigningCertificateExtension> {
    self
      .extensions()
      .iter()
      .filter_map(|ext| CodeSigningCertificateExtension::try_from(&ext.id).ok())
      .collect::<Vec<_>>()
  }
}

/// Create a new self-signed X.509 certificate suitable for signing code.
///
/// The created certificate contains all the extensions needed to convey
/// that it is used for code signing and should resemble certificates.
///
/// However, because the certificate isn't signed by Apple or another
/// trusted certificate authority, binaries signed with the certificate
/// may not pass Apple's verification requirements and the OS may refuse
/// to proceed. Needless to say, only use certificates generated with this
/// function for testing purposes only.
pub fn create_self_signed_code_signing_certificate(
  algorithm: KeyAlgorithm,
  profile: CertificateProfile,
  team_id: &str,
  person_name: &str,
  country: &str,
  validity_duration: chrono::Duration,
) -> Result<(CapturedX509Certificate, InMemorySigningKeyPair), AppleCodesignError> {
  let mut builder = X509CertificateBuilder::default();

  builder.apple_certificate_profile(profile)?;
  builder.apple_subject(team_id, person_name, country)?;
  builder.validity_duration(validity_duration);

  // x509-certificate crate doesn't support RSA key generation. So do
  // that ourselves.
  if matches!(algorithm, KeyAlgorithm::Rsa) {
    let private_key = rsa::RsaPrivateKey::new(&mut rand08::thread_rng(), 2048).map_err(|e| {
      AppleCodesignError::CertificateBuildError(format!("error generating RSA key: {}", e))
    })?;
    let key_pair = InMemorySigningKeyPair::from_pkcs8_der(
      private_key
        .to_pkcs8_der()
        .map_err(|e| {
          AppleCodesignError::CertificateGeneric(format!("error converting RSA key to DER: {}", e))
        })?
        .as_bytes(),
    )?;

    let cert = builder.create_with_key_pair(&key_pair)?;

    Ok((cert, key_pair))
  } else {
    Ok(builder.create_with_random_keypair(algorithm)?)
  }
}
