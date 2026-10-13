// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Functionality related to certificates.

use {
  crate::{apple_certificates::KnownCertificate, error::AppleCodesignError},
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

/// All OIDs known for extended key usage.
const ALL_OID_EKUS: &[&ConstOid; 4] = &[
  &OID_EKU_PURPOSE_CODE_SIGNING,
  &OID_EKU_PURPOSE_SAFARI_DEVELOPER,
  &OID_EKU_PURPOSE_3RD_PARTY_MAC_DEVELOPER_INSTALLER,
  &OID_EKU_PURPOSE_DEVELOPER_ID_INSTALLER,
];

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

/// All OIDs associated with non Certificate Authority extensions.
const ALL_OID_NON_CA_EXTENSIONS: &[&ConstOid; 18] = &[
  &OID_EXTENSION_APPLE_SIGNING,
  &OID_EXTENSION_IPHONE_DEVELOPER,
  &OID_EXTENSION_IPHONE_OS_APPLICATION_SIGNING,
  &OID_EXTENSION_APPLE_DEVELOPER_CERTIFICATE_SUBMISSION,
  &OID_EXTENSION_SAFARI_DEVELOPER,
  &OID_EXTENSION_IPHONE_OS_VPN_SIGNING,
  &OID_EXTENSION_APPLE_MAC_APP_SIGNING_DEVELOPMENT,
  &OID_EXTENSION_APPLE_MAC_APP_SIGNING_SUBMISSION,
  &OID_EXTENSION_APPLE_MAC_APP_STORE_CODE_SIGNING,
  &OID_EXTENSION_APPLE_MAC_APP_STORE_INSTALLER_SIGNING,
  &OID_EXTENSION_MAC_DEVELOPER,
  &OID_EXTENSION_DEVELOPER_ID_APPLICATION,
  &OID_EXTENSION_DEVELOPER_ID_INSTALLER,
  &OID_EXTENSION_PASSBOOK_SIGNING,
  &OID_EXTENSION_WEBSITE_PUSH_NOTIFICATION_SIGNING,
  &OID_EXTENSION_DEVELOPER_ID_KERNEL,
  &OID_EXTENSION_DEVELOPER_ID_DATE,
  &OID_EXTENSION_TEST_FLIGHT,
];

/// UserID.
///
/// 0.9.2342.19200300.100.1.1
pub const OID_USER_ID: ConstOid = Oid(&[9, 146, 38, 137, 147, 242, 44, 100, 1, 1]);

/// OID used for email address in RDN in Apple generated code signing certificates.
const OID_EMAIL_ADDRESS: ConstOid = Oid(&[42, 134, 72, 134, 247, 13, 1, 9, 1]);

/// Apple Worldwide Developer Relations.
///
/// 1.2.840.113635.100.6.2.1
const OID_CA_EXTENSION_APPLE_WORLDWIDE_DEVELOPER_RELATIONS: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 2, 1]);

/// Apple Application Integration.
///
/// 1.2.840.113635.100.6.2.3
const OID_CA_EXTENSION_APPLE_APPLICATION_INTEGRATION: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 2, 3]);

/// Developer ID Certification Authority
///
/// 1.2.840.113635.100.6.2.6
const OID_CA_EXTENSION_DEVELOPER_ID: ConstOid = Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 2, 6]);

/// Apple Timestamp.
///
/// 1.2.840.113635.100.6.2.9
const OID_CA_EXTENSION_APPLE_TIMESTAMP: ConstOid = Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 2, 9]);

/// Developer Authentication Certification Authority.
///
/// 1.2.840.113635.100.6.2.11
const OID_CA_EXTENSION_DEVELOPER_AUTHENTICATION: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 2, 11]);

/// Apple Application Integration CA - G3
///
/// 1.2.840.113635.100.6.2.14
const OID_CA_EXTENSION_APPLE_APPLICATION_INTEGRATION_G3: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 2, 14]);

/// Apple Worldwide Developer Relations CA - G2
///
/// 1.2.840.113635.100.6.2.15
const OID_CA_EXTENSION_APPLE_WORLDWIDE_DEVELOPER_RELATIONS_G2: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 2, 15]);

/// Apple Software Update Certification.
///
/// 1.2.840.113635.100.6.2.19
const OID_CA_EXTENSION_APPLE_SOFTWARE_UPDATE_CERTIFICATION: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 2, 19]);

/// Apple Application Integration CA - G1.
///
/// This was introduced in `Apple Application Integration CA 7 - G1`
/// The previous `Apple Application Integration CA 5 - G1` certificate
/// had the legacy 1.2.840.113635.100.6.2.3 extension.
///
/// 1.2.840.113635.100.6.2.31
const OID_CA_EXTENSION_APPLE_APPLICATION_INTEGRATION_G1: ConstOid =
  Oid(&[42, 134, 72, 134, 247, 99, 100, 6, 2, 31]);

const ALL_OID_CA_EXTENSIONS: &[&ConstOid; 9] = &[
  &OID_CA_EXTENSION_APPLE_WORLDWIDE_DEVELOPER_RELATIONS,
  &OID_CA_EXTENSION_APPLE_APPLICATION_INTEGRATION,
  &OID_CA_EXTENSION_DEVELOPER_ID,
  &OID_CA_EXTENSION_APPLE_TIMESTAMP,
  &OID_CA_EXTENSION_DEVELOPER_AUTHENTICATION,
  &OID_CA_EXTENSION_APPLE_APPLICATION_INTEGRATION_G3,
  &OID_CA_EXTENSION_APPLE_WORLDWIDE_DEVELOPER_RELATIONS_G2,
  &OID_CA_EXTENSION_APPLE_SOFTWARE_UPDATE_CERTIFICATION,
  &OID_CA_EXTENSION_APPLE_APPLICATION_INTEGRATION_G1,
];

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
  /// Obtain all variants of this enumeration.
  pub fn all() -> Vec<Self> {
    vec![
      Self::CodeSigning,
      Self::SafariDeveloper,
      Self::ThirdPartyMacDeveloperInstaller,
      Self::DeveloperIdInstaller,
    ]
  }

  pub fn all_oids() -> &'static [&'static ConstOid] {
    ALL_OID_EKUS
  }

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
  /// Obtain all variants of this enumeration.
  pub fn all() -> Vec<Self> {
    vec![
      Self::AppleSigning,
      Self::IPhoneDeveloper,
      Self::IPhoneOsApplicationSigning,
      Self::AppleDeveloperCertificateSubmission,
      Self::SafariDeveloper,
      Self::IPhoneOsVpnSigning,
      Self::AppleMacAppSigningDevelopment,
      Self::AppleMacAppSigningSubmission,
      Self::AppleMacAppStoreCodeSigning,
      Self::AppleMacAppStoreInstallerSigning,
      Self::MacDeveloper,
      Self::DeveloperIdApplication,
      Self::DeveloperIdDate,
      Self::DeveloperIdInstaller,
      Self::ApplePayPassbookSigning,
      Self::WebsitePushNotificationSigning,
      Self::DeveloperIdKernel,
      Self::TestFlight,
    ]
  }

  /// All OIDs known to be extensions in code signing certificates.
  pub fn all_oids() -> &'static [&'static ConstOid] {
    ALL_OID_NON_CA_EXTENSIONS
  }

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

/// Denotes specific certificate extensions on Apple certificate authority certificates.
///
/// Apple's CA certificates have extensions that appear to identify the role of
/// that CA. This enumeration defines those.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertificateAuthorityExtension {
  /// Apple Worldwide Developer Relations.
  ///
  /// An intermediate CA.
  AppleWorldwideDeveloperRelations,

  /// Apple Application Integration.
  AppleApplicationIntegration,

  /// Developer ID Certification Authority.
  DeveloperId,

  /// Apple Timestamp.
  AppleTimestamp,

  /// Developer Authentication Certification Authority.
  DeveloperAuthentication,

  /// Application Application Integration CA - G3.
  AppleApplicationIntegrationG3,

  /// Apple Worldwide Developer Relations CA - G2.
  AppleWorldwideDeveloperRelationsG2,

  /// Apple Software Update Certification.
  AppleSoftwareUpdateCertification,

  /// Apple Application Integration CA - G1.
  AppleApplicationIntegrationG1,
}

impl CertificateAuthorityExtension {
  /// Obtain all variants of this enumeration.
  pub fn all() -> Vec<Self> {
    vec![
      Self::AppleWorldwideDeveloperRelations,
      Self::AppleApplicationIntegration,
      Self::DeveloperId,
      Self::AppleTimestamp,
      Self::DeveloperAuthentication,
      Self::AppleApplicationIntegrationG3,
      Self::AppleWorldwideDeveloperRelationsG2,
      Self::AppleSoftwareUpdateCertification,
      Self::AppleApplicationIntegrationG1,
    ]
  }

  /// All the known OIDs constituting Apple CA extensions.
  pub fn all_oids() -> &'static [&'static ConstOid] {
    ALL_OID_CA_EXTENSIONS
  }

  pub fn as_oid(&self) -> ConstOid {
    match self {
      Self::AppleWorldwideDeveloperRelations => {
        OID_CA_EXTENSION_APPLE_WORLDWIDE_DEVELOPER_RELATIONS
      }
      Self::AppleApplicationIntegration => OID_CA_EXTENSION_APPLE_APPLICATION_INTEGRATION,
      Self::DeveloperId => OID_CA_EXTENSION_DEVELOPER_ID,
      Self::AppleTimestamp => OID_CA_EXTENSION_APPLE_TIMESTAMP,
      Self::DeveloperAuthentication => OID_CA_EXTENSION_DEVELOPER_AUTHENTICATION,
      Self::AppleApplicationIntegrationG3 => OID_CA_EXTENSION_APPLE_APPLICATION_INTEGRATION_G3,
      Self::AppleWorldwideDeveloperRelationsG2 => {
        OID_CA_EXTENSION_APPLE_WORLDWIDE_DEVELOPER_RELATIONS_G2
      }
      Self::AppleSoftwareUpdateCertification => {
        OID_CA_EXTENSION_APPLE_SOFTWARE_UPDATE_CERTIFICATION
      }
      Self::AppleApplicationIntegrationG1 => OID_CA_EXTENSION_APPLE_APPLICATION_INTEGRATION_G1,
    }
  }
}

impl Display for CertificateAuthorityExtension {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      CertificateAuthorityExtension::AppleWorldwideDeveloperRelations => {
        f.write_str("Apple Worldwide Developer Relations")
      }
      CertificateAuthorityExtension::AppleApplicationIntegration => {
        f.write_str("Apple Application Integration")
      }
      CertificateAuthorityExtension::DeveloperId => {
        f.write_str("Developer ID Certification Authority")
      }
      CertificateAuthorityExtension::AppleTimestamp => f.write_str("Apple Timestamp"),
      CertificateAuthorityExtension::DeveloperAuthentication => {
        f.write_str("Developer Authentication Certification Authority")
      }
      CertificateAuthorityExtension::AppleApplicationIntegrationG3 => {
        f.write_str("Apple Application Integration CA - G3")
      }
      CertificateAuthorityExtension::AppleWorldwideDeveloperRelationsG2 => {
        f.write_str("Apple Worldwide Developer Relations CA - G2")
      }
      CertificateAuthorityExtension::AppleSoftwareUpdateCertification => {
        f.write_str("Apple Software Update Certification")
      }
      CertificateAuthorityExtension::AppleApplicationIntegrationG1 => {
        f.write_str("Apple Application Integration CA - G1")
      }
    }
  }
}

impl TryFrom<&Oid> for CertificateAuthorityExtension {
  type Error = AppleCodesignError;

  fn try_from(oid: &Oid) -> Result<Self, Self::Error> {
    // Surely there is a way to use `match`. But the `Oid` type is a bit wonky.
    if oid.as_ref() == OID_CA_EXTENSION_APPLE_WORLDWIDE_DEVELOPER_RELATIONS.as_ref() {
      Ok(Self::AppleWorldwideDeveloperRelations)
    } else if oid.as_ref() == OID_CA_EXTENSION_APPLE_APPLICATION_INTEGRATION.as_ref() {
      Ok(Self::AppleApplicationIntegration)
    } else if oid.as_ref() == OID_CA_EXTENSION_DEVELOPER_ID.as_ref() {
      Ok(Self::DeveloperId)
    } else if oid.as_ref() == OID_CA_EXTENSION_APPLE_TIMESTAMP.as_ref() {
      Ok(Self::AppleTimestamp)
    } else if oid.as_ref() == OID_CA_EXTENSION_DEVELOPER_AUTHENTICATION.as_ref() {
      Ok(Self::DeveloperAuthentication)
    } else if oid.as_ref() == OID_CA_EXTENSION_APPLE_APPLICATION_INTEGRATION_G3.as_ref() {
      Ok(Self::AppleApplicationIntegrationG3)
    } else if oid.as_ref() == OID_CA_EXTENSION_APPLE_WORLDWIDE_DEVELOPER_RELATIONS_G2.as_ref() {
      Ok(Self::AppleWorldwideDeveloperRelationsG2)
    } else if oid.as_ref() == OID_CA_EXTENSION_APPLE_SOFTWARE_UPDATE_CERTIFICATION.as_ref() {
      Ok(Self::AppleSoftwareUpdateCertification)
    } else if oid.as_ref() == OID_CA_EXTENSION_APPLE_APPLICATION_INTEGRATION_G1.as_ref() {
      Ok(Self::AppleApplicationIntegrationG1)
    } else {
      Err(AppleCodesignError::OidIsntCertificateAuthority)
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

/// Extends functionality of [CapturedX509Certificate] with Apple specific certificate knowledge.
pub trait AppleCertificate: Sized {
  /// Whether this is a known Apple root certificate authority.
  ///
  /// We define this criteria as a certificate in our built-in list of known
  /// Apple certificates that has the same subject and issuer Names.
  fn is_apple_root_ca(&self) -> bool;

  /// Whether this is a known Apple intermediate certificate authority.
  ///
  /// This is similar to [Self::is_apple_root_ca] except it doesn't match against
  /// known self-signed Apple certificates.
  fn is_apple_intermediate_ca(&self) -> bool;

  /// Find [CertificateAuthorityExtension] present on this certificate.
  ///
  /// If this is non-empty, the certificate says it is an Apple certificate
  /// whose role is issuing other certificates using for signing things.
  ///
  /// This function does not perform trust validation that the underlying
  /// certificate is a legitimate Apple issued certificate: just that it has
  /// the desired property.
  fn apple_ca_extensions(&self) -> Vec<CertificateAuthorityExtension>;

  /// Obtain all of Apple's [ExtendedKeyUsagePurpose] in this certificate.
  fn apple_extended_key_usage_purposes(&self) -> Vec<ExtendedKeyUsagePurpose>;

  /// Obtain all of Apple's [CodeSigningCertificateExtension] in this certificate.
  fn apple_code_signing_extensions(&self) -> Vec<CodeSigningCertificateExtension>;

  /// Attempt to guess the [CertificateProfile] associated with this certificate.
  ///
  /// This keys off present certificate extensions to guess which profile it
  /// belongs to. Incorrect guesses are possible, which is why *guess* is in the
  /// function name.
  ///
  /// Returns `None` if we don't think a [CertificateProfile] is associated with
  /// this extension.
  fn apple_guess_profile(&self) -> Option<CertificateProfile>;

  /// Attempt to resolve the certificate issuer chain back to [AppleCertificate].
  ///
  /// This is a glorified wrapper around [CapturedX509Certificate::resolve_signing_chain]
  /// that filters matches against certificates in our known set of Apple
  /// certificates and maps them back to our [KnownCertificate] Rust enumeration.
  ///
  /// False negatives (read: missing certificates) can be encountered if
  /// we don't know about an Apple CA certificate.
  fn apple_issuing_chain(&self) -> Vec<KnownCertificate>;

  /// Whether this certificate chains back to a known Apple root certificate authority.
  ///
  /// This is true if the resolved certificate issuance chain (which is
  /// confirmed via verifying the cryptographic signatures on certificates)
  /// ands in a certificate that is known to be an Apple root CA.
  fn chains_to_apple_root_ca(&self) -> bool;

  /// Obtain the chain of issuing certificates, back to a known Apple root.
  ///
  /// The returned chain starts with this certificate and ends with a known
  /// Apple root certificate authority. None is returned if this certificate
  /// doesn't appear to chain to a known Apple root CA.
  fn apple_root_certificate_chain(&self) -> Option<Vec<CapturedX509Certificate>>;

  /// Attempt to resolve the *team id* of an Apple issued certificate.
  ///
  /// The *team id* is a value like `AB42XYZ789` that is attached to your
  /// Apple Developer account. It seems to always be embedded in signing
  /// certificates as the Organizational Unit field of the subject. So this
  /// function is just a shortcut for retrieving that.
  fn apple_team_id(&self) -> Option<String>;

  /// Whether this is a certificate pretending to be signed by an Apple CA but isn't really.
  fn is_test_apple_signed_certificate(&self) -> bool;
}

impl AppleCertificate for CapturedX509Certificate {
  fn is_apple_root_ca(&self) -> bool {
    KnownCertificate::all_roots().contains(&self)
  }

  fn is_apple_intermediate_ca(&self) -> bool {
    KnownCertificate::all().contains(&self) && !KnownCertificate::all_roots().contains(&self)
  }

  fn apple_ca_extensions(&self) -> Vec<CertificateAuthorityExtension> {
    let cert: &x509_certificate::rfc5280::Certificate = self.as_ref();

    cert
      .iter_extensions()
      .filter_map(|extension| CertificateAuthorityExtension::try_from(&extension.id).ok())
      .collect::<Vec<_>>()
  }

  fn apple_extended_key_usage_purposes(&self) -> Vec<ExtendedKeyUsagePurpose> {
    let cert: &x509_certificate::rfc5280::Certificate = self.as_ref();

    cert
      .iter_extensions()
      .filter_map(|extension| {
        if extension.id.as_ref() == OID_EXTENDED_KEY_USAGE.as_ref() {
          if let Some(oid) = extension.try_decode_sequence_single_oid() {
            ExtendedKeyUsagePurpose::try_from(&oid).ok()
          } else {
            None
          }
        } else {
          None
        }
      })
      .collect::<Vec<_>>()
  }

  fn apple_code_signing_extensions(&self) -> Vec<CodeSigningCertificateExtension> {
    let cert: &x509_certificate::rfc5280::Certificate = self.as_ref();

    cert
      .iter_extensions()
      .filter_map(|extension| CodeSigningCertificateExtension::try_from(&extension.id).ok())
      .collect::<Vec<_>>()
  }

  fn apple_guess_profile(&self) -> Option<CertificateProfile> {
    let ekus = self.apple_extended_key_usage_purposes();
    let signing = self.apple_code_signing_extensions();

    // Some EKUs uniquely identify the certificate profile. We don't yet handle
    // all EKUs because we don't have profiles defined for them.
    //
    // Ideally this logic stays in sync with apple_certificate_profile().
    if ekus.contains(&ExtendedKeyUsagePurpose::DeveloperIdInstaller) {
      Some(CertificateProfile::DeveloperIdInstaller)
    } else if ekus.contains(&ExtendedKeyUsagePurpose::ThirdPartyMacDeveloperInstaller) {
      Some(CertificateProfile::MacInstallerDistribution)
      // That's all the EKUs that have a 1:1 to CertificateProfile. Now look at
      // code signing extensions.
    } else if signing.contains(&CodeSigningCertificateExtension::DeveloperIdApplication) {
      Some(CertificateProfile::DeveloperIdApplication)
    } else if signing.contains(&CodeSigningCertificateExtension::IPhoneDeveloper)
      && signing.contains(&CodeSigningCertificateExtension::MacDeveloper)
    {
      Some(CertificateProfile::AppleDevelopment)
    } else if signing.contains(&CodeSigningCertificateExtension::AppleMacAppSigningDevelopment)
      && signing.contains(&CodeSigningCertificateExtension::AppleDeveloperCertificateSubmission)
    {
      Some(CertificateProfile::AppleDistribution)
    } else {
      None
    }
  }

  fn apple_issuing_chain(&self) -> Vec<KnownCertificate> {
    self
      .resolve_signing_chain(KnownCertificate::all().iter().copied())
      .into_iter()
      .filter_map(|cert| KnownCertificate::try_from(cert).ok())
      .collect::<Vec<_>>()
  }

  fn chains_to_apple_root_ca(&self) -> bool {
    if self.is_apple_root_ca() {
      true
    } else {
      self
        .resolve_signing_chain(KnownCertificate::all().iter().copied())
        .into_iter()
        .any(|cert| cert.is_apple_root_ca())
    }
  }

  fn apple_root_certificate_chain(&self) -> Option<Vec<CapturedX509Certificate>> {
    let mut chain = vec![self.clone()];

    for cert in self.resolve_signing_chain(KnownCertificate::all().iter().copied()) {
      chain.push(cert.clone());

      if cert.is_apple_root_ca() {
        break;
      }
    }

    if chain.last().unwrap().is_apple_root_ca() {
      Some(chain)
    } else {
      None
    }
  }

  fn apple_team_id(&self) -> Option<String> {
    self
      .subject_name()
      .find_first_attribute_string(Oid(
        x509_certificate::rfc4519::OID_ORGANIZATIONAL_UNIT_NAME
          .as_ref()
          .into(),
      ))
      .unwrap_or(None)
  }

  fn is_test_apple_signed_certificate(&self) -> bool {
    if let Ok(digest) = self.sha256_fingerprint() {
      hex::encode(digest) == "5939ad5770d8b977b38d07533754371314744e87a8d606433f689e9bc6b980a0"
    } else {
      false
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

  /// Add an email address to the certificate's subject name.
  fn apple_email_address(&mut self, address: &str) -> Result<(), AppleCodesignError>;

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

  fn apple_email_address(&mut self, address: &str) -> Result<(), AppleCodesignError> {
    self
      .subject()
      .append_utf8_string(Oid(OID_EMAIL_ADDRESS.as_ref().into()), address)
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
