// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use {
    crate::{macho_universal::UniversalMachOError, remote_signing::RemoteSignError},
    cryptographic_message_syntax::CmsError,
    std::path::PathBuf,
    thiserror::Error,
    x509_certificate::{KeyAlgorithm, X509CertificateError},
};

/// Unified error type for Apple code signing.
#[derive(Debug, Error)]
pub enum AppleCodesignError {
    #[error("unknown command")]
    CliUnknownCommand,

    #[error("bad argument")]
    CliBadArgument,

    #[error("{0}")]
    CliGeneralError(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CLI error: {0}")]
    CliDialoguer(#[from] dialoguer::Error),

    #[error("binary parsing error: {0}")]
    Goblin(#[from] goblin::error::Error),

    #[error("invalid Mach-O binary: {0}")]
    InvalidBinary(String),

    #[error("invalid binary index within Mach-O: {0}")]
    InvalidMachOIndex(usize),

    #[error("binary does not have code signature data")]
    BinaryNoCodeSignature,

    #[error("binary does not have code directory blob")]
    BinaryNoCodeDirectory,

    #[error("X.509 certificate handler error: {0}")]
    X509(#[from] X509CertificateError),

    #[error("CMS error: {0}")]
    Cms(#[from] CmsError),

    #[error("JSON serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("YAML serialization error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),

    #[error("glob error: {0}")]
    GlobPattern(#[from] glob::PatternError),

    #[error("problems reported during verification")]
    VerificationProblems,

    #[error("certificate error: {0}")]
    CertificateGeneric(String),

    #[error("certificate decode error: {0}")]
    CertificateDecode(bcder::decode::DecodeError<std::convert::Infallible>),

    #[error("PEM error: {0}")]
    CertificatePem(pem::PemError),

    #[error("X.509 certificate parsing error: {0}")]
    X509Parse(String),

    #[error("unsupported key algorithm in certificate: {0:?}")]
    CertificateUnsupportedKeyAlgorithm(KeyAlgorithm),

    #[error("unspecified cryptography error in certificate")]
    CertificateRing(ring::error::Unspecified),

    #[error("bad string value in certificate: {0:?}")]
    CertificateCharset(bcder::string::CharSetError),

    #[error("DER: {0}")]
    Der(#[from] der::Error),

    #[error("error parsing version string: {0}")]
    VersionParse(#[from] semver::Error),

    #[error("XAR error: {0}")]
    Xar(#[from] apple_xar::Error),

    #[error("Apple flat package error: {0}")]
    FlatPackage(#[from] apple_flat_package::Error),

    #[error("unable to locate __TEXT segment")]
    MissingText,

    #[error("unable to locate __LINKEDIT segment")]
    MissingLinkedit,

    #[error("bad header magic in {0}")]
    BadMagic(&'static str),

    #[error("data structure parse error: {0}")]
    Scroll(#[from] scroll::Error),

    #[error("error parsing plist XML: {0}")]
    PlistParseXml(plist::Error),

    #[error("error serializing plist to XML: {0}")]
    PlistSerializeXml(plist::Error),

    #[error("malformed identifier string in code directory")]
    CodeDirectoryMalformedIdentifier,

    #[error("malformed team name string in code directory")]
    CodeDirectoryMalformedTeam,

    #[error("plist error in code directory: {0}")]
    CodeDirectoryPlist(plist::Error),

    #[error("SuperBlob data is malformed")]
    SuperblobMalformed,

    #[error("specified path is not of a recognized type")]
    UnrecognizedPathType,

    #[error("functionality not implemented: {0}")]
    Unimplemented(&'static str),

    #[error("unknown code signature flag: {0}")]
    CodeSignatureUnknownFlag(String),

    #[error("entitlements data not valid UTF-8: {0}")]
    EntitlementsBadUtf8(std::str::Utf8Error),

    #[error("error with plist DER encoding: {0}")]
    PlistDer(String),

    #[error("unknown executable segment flag: {0}")]
    ExecutableSegmentUnknownFlag(String),

    #[error("unknown code requirement opcode: {0}")]
    RequirementUnknownOpcode(u32),

    #[error("unknown code requirement match expression: {0}")]
    RequirementUnknownMatchExpression(u32),

    #[error("code requirement data malformed: {0}")]
    RequirementMalformed(&'static str),

    #[error("plist error in code resources: {0}")]
    ResourcesPlist(plist::Error),

    #[error("base64 error in code resources: {0}")]
    ResourcesBase64(base64::DecodeError),

    #[error("plist parse error in code resources: {0}")]
    ResourcesPlistParse(String),

    #[error("bad regular expression in code resources: {0}; {1}")]
    ResourcesBadRegex(String, regex::Error),

    #[error("__LINKEDIT isn't final Mach-O segment")]
    LinkeditNotLast,

    #[error("__LINKEDIT segment contains data after signature")]
    DataAfterSignature,

    #[error("insufficient room to write code signature load command")]
    LoadCommandNoRoom,

    #[error("error writing Mach-O: {0}")]
    MachOWrite(String),

    #[error("no identifier string provided")]
    NoIdentifier,

    #[error("no signing certificate")]
    NoSigningCertificate,

    #[error("signature data too large (please report this issue)")]
    SignatureDataTooLarge,

    #[error("invalid builder operation: {0}")]
    SignatureBuilder(&'static str),

    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("unknown digest algorithm")]
    DigestUnknownAlgorithm,

    #[error("unsupported digest algorithm")]
    DigestUnsupportedAlgorithm,

    #[error("unspecified digest error")]
    DigestUnspecified,

    #[error("deriving identifier from path: {0}")]
    PathIdentifier(String),

    #[error("error interfacing with directory-based bundle: {0}")]
    DirectoryBundle(anyhow::Error),

    #[error("nested bundle does not exist: {0}")]
    BundleUnknown(String),

    #[error("bundle Info.plist does not define CFBundleIdentifier: {0}")]
    BundleNoIdentifier(PathBuf),

    #[error("bundle Info.plist does not define CFBundleExecutable: {0}")]
    BundleNoMainExecutable(PathBuf),

    #[error(
        "unexpected resource rule evaluation when signing nested bundle (please report this issue)"
    )]
    BundleUnexpectedResourceRuleResult,

    #[error("unable to parse settings scope: {0}")]
    ParseSettingsScope(String),

    #[error("incorrect password given when decrypting PFX data")]
    PfxBadPassword,

    #[error("error parsing PFX data: {0}")]
    PfxParseError(String),

    #[cfg(target_os = "macos")]
    #[error("SecurityFramework error: {0}")]
    SecurityFramework(#[from] security_framework::base::Error),

    #[error("error interfacing with macOS keychain: {0}")]
    KeychainError(String),

    #[error("error interfacing with Windows certificate store: {0}")]
    WindowsStoreError(String),

    #[error("failed to find certificate satisfying requirements: {0}")]
    CertificateNotFound(String),

    #[error("the given OID does not match a recognized Apple certificate authority extension")]
    OidIsntCertificateAuthority,

    #[error("the given OID does not match a recognized Apple extended key usage extension")]
    OidIsntExtendedKeyUsage,

    #[error("the given OID does not match a recognized Apple code signing extension")]
    OidIsntCodeSigningExtension,

    #[error("error building certificate: {0}")]
    CertificateBuildError(String),

    #[error("unknown certificate profile: {0}")]
    UnknownCertificateProfile(String),

    #[error("unknown code execution policy: {0}")]
    UnknownPolicy(String),

    #[error("unable to generate code requirement policy: {0}")]
    PolicyFormulationError(String),

    #[error("error producing universal Mach-O binary: {0}")]
    UniversalMachO(#[from] UniversalMachOError),

    #[error("walkdir error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("zip error: {0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("error writing app metadata XML: {0}")]
    AppMetadataXml(xml::writer::Error),

    #[error("error writing XML: {0}")]
    XmlWrite(xml::writer::Error),

    #[error("signing XAR archives requires a signing certificate")]
    XarNoAdhoc,

    #[error("App Store Connect API Key error: {0}")]
    AppStoreConnectApiKey(String),

    #[error("Could not find App Store Connect API key in default search locations")]
    AppStoreConnectApiKeyNotFound,

    #[error("signing settings are not compatible with notarization")]
    ForNotarizationInvalidSettings,

    #[error("do not know how to notarize {0}")]
    NotarizeUnsupportedPath(PathBuf),

    #[error("no authentication credentials to perform notarization request")]
    NotarizeNoAuthCredentials,

    #[error("reached time limit waiting for notarization to complete")]
    NotarizeWaitLimitReached,

    #[error("error interacting with Notary API")]
    NotarizeServerError,

    #[error("notarization rejected: StatusCode={0}; StatusMessage={1}")]
    NotarizeRejected(i64, String),

    #[error("notarization is incomplete (no status code and message)")]
    NotarizeIncomplete,

    #[error("notarization package is invalid")]
    NotarizeInvalid,

    #[error("notarization record not in response: {0}")]
    NotarizationRecordNotInResponse(String),

    #[error("signed ticket data not found in ticket lookup response (this should not happen)")]
    NotarizationRecordNoSignedTicket,

    #[error("signedTicket in notarization ticket lookup response is not BYTES: {0}")]
    NotarizationRecordSignedTicketNotBytes(String),

    #[error("notarization ticket lookup failure: {0}: {1}")]
    NotarizationLookupFailure(String, String),

    #[error("error decoding base64 in notarization ticket: {0}")]
    NotarizationRecordDecodeFailure(base64::DecodeError),

    #[error("unable to determine app platform from bundle")]
    BundleUnknownAppPlatform,

    #[error("do not support stapling {0:?} bundles")]
    StapleUnsupportedBundleType(apple_bundles::BundlePackageType),

    #[error("XAR file is malformed; cannot staple")]
    StapleMalformedXar,

    #[error("failed to find main executable in bundle")]
    StapleMainExecutableNotFound,

    #[error("do not know how to staple {0}")]
    StapleUnsupportedPath(PathBuf),

    #[error("bad header magic in DMG; not a DMG file?")]
    DmgBadMagic,

    #[error("cannot notarize DMG without an embedded signature")]
    DmgNotarizeNoSignature,

    #[error("cannot staple DMG without an embedded signature")]
    DmgStapleNoSignature,

    #[error("failed to find certificate in smartcard slot {0}")]
    SmartcardNoCertificate(String),

    #[error("failed to authenticate with smartcard device")]
    SmartcardFailedAuthentication,

    #[cfg(feature = "yubikey")]
    #[error("YubiKey error: {0}")]
    YubiKey(#[from] yubikey::Error),

    #[error("poisoned lock")]
    PoisonedLock,

    #[error("internal API / logic error: {0}")]
    LogicError(String),

    #[error("zip structs error: {0}")]
    ZipStructs(#[from] zip_structs::zip_error::ZipReadError),

    #[error("PKCS11 error: {0}")]
    Pkcs11Error(String),

    #[error("remote signing error: {0}")]
    RemoteSign(#[from] RemoteSignError),

    #[cfg(feature = "notarize")]
    #[error("bytestream creation error: {0}")]
    AwsByteStream(#[from] aws_smithy_types::byte_stream::error::Error),

    #[cfg(feature = "notarize")]
    #[error("s3 upload error: {0}")]
    AwsS3PutObject(
        aws_smithy_types::error::display::DisplayErrorContext<
            aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::put_object::PutObjectError>,
        >,
    ),

    #[error("bad time value")]
    BadTime,

    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("plist: {0}")]
    Plist(#[from] plist::Error),

    #[error("config error: {0:?}")]
    Figment(#[from] figment::Error),

    #[error("environment constraints: {0}")]
    EnvironmentConstraint(String),
}

/// Result type for this library.
pub type Result<T, E = AppleCodesignError> = std::result::Result<T, E>;
