// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Apple X.509 certificates.
//!
//! This module defines well-known Apple X.509 certificates.
//!
//! The canonical source of this data is <https://www.apple.com/certificateauthority/>.
//!
//! Note that some certificates are commented out and not available
//! because the official DER-encoded certificates provided by Apple
//! do not conform to the encoding standards in RFC 5280.

use {once_cell::sync::Lazy, std::ops::Deref, x509_certificate::CapturedX509Certificate};

/// Apple Inc. Root Certificate
static APPLE_INC_ROOT_CERTIFICATE: Lazy<CapturedX509Certificate> = Lazy::new(|| {
    CapturedX509Certificate::from_der(
        include_bytes!("apple-certs/AppleIncRootCertificate.cer").to_vec(),
    )
    .unwrap()
});

/// Apple Computer, Inc. Root Certificate.
static APPLE_COMPUTER_INC_ROOT_CERTIFICATE: Lazy<CapturedX509Certificate> = Lazy::new(|| {
    CapturedX509Certificate::from_der(
        include_bytes!("apple-certs/AppleComputerRootCertificate.cer").to_vec(),
    )
    .unwrap()
});

/// Apple Root CA - G2 Root Certificate
static APPLE_ROOT_CA_G2_ROOT_CERTIFICATE: Lazy<CapturedX509Certificate> = Lazy::new(|| {
    CapturedX509Certificate::from_der(include_bytes!("apple-certs/AppleRootCA-G2.cer").to_vec())
        .unwrap()
});

/// Apple Root CA - G3 Root Certificate
static APPLE_ROOT_CA_G3_ROOT_CERTIFICATE: Lazy<CapturedX509Certificate> = Lazy::new(|| {
    CapturedX509Certificate::from_der(include_bytes!("apple-certs/AppleRootCA-G3.cer").to_vec())
        .unwrap()
});

/// Apple IST CA 2 - G1 Certificate
static APPLE_IST_CA_2_G1_CERTIFICATE: Lazy<CapturedX509Certificate> = Lazy::new(|| {
    CapturedX509Certificate::from_der(include_bytes!("apple-certs/AppleISTCA2G1.cer").to_vec())
        .unwrap()
});

/// Apple IST CA 8 - G1 Certificate
static APPLE_IST_CA_8_G1_CERTIFICATE: Lazy<CapturedX509Certificate> = Lazy::new(|| {
    CapturedX509Certificate::from_der(include_bytes!("apple-certs/AppleISTCA8G1.cer").to_vec())
        .unwrap()
});

/// Application Integration Certificate
static APPLICATION_INTEGRATION_CERTIFICATE: Lazy<CapturedX509Certificate> = Lazy::new(|| {
    CapturedX509Certificate::from_der(include_bytes!("apple-certs/AppleAAICA.cer").to_vec())
        .unwrap()
});

/// Application Integration 2 Certificate
static APPLICATION_INTEGRATION_2_CERTIFICATE: Lazy<CapturedX509Certificate> = Lazy::new(|| {
    CapturedX509Certificate::from_der(include_bytes!("apple-certs/AppleAAI2CA.cer").to_vec())
        .unwrap()
});

/// Application Integration - G3 Certificate
static APPLICATION_INTEGRATION_G3_CERTIFICATE: Lazy<CapturedX509Certificate> = Lazy::new(|| {
    CapturedX509Certificate::from_der(include_bytes!("apple-certs/AppleAAICAG3.cer").to_vec())
        .unwrap()
});

/// Apple Application Integration CA 5 - G1 Certificate
static APPLE_APPLICATION_INTEGRATION_CA_5_G1_CERTIFICATE: Lazy<CapturedX509Certificate> =
    Lazy::new(|| {
        CapturedX509Certificate::from_der(
            include_bytes!("apple-certs/AppleApplicationIntegrationCA5G1.cer").to_vec(),
        )
        .unwrap()
    });

/// Apple Application Integration CA 7 - G1 Certificate
static APPLE_APPLICATION_INTEGRATION_CA_7_G1_CERTIFICATE: Lazy<CapturedX509Certificate> =
    Lazy::new(|| {
        CapturedX509Certificate::from_der(
            include_bytes!("apple-certs/AppleApplicationIntegrationCA7G1.cer").to_vec(),
        )
        .unwrap()
    });

/// Developer Authentication Certificate
static DEVELOPER_AUTHENTICATION_CERTIFICATE: Lazy<CapturedX509Certificate> = Lazy::new(|| {
    CapturedX509Certificate::from_der(include_bytes!("apple-certs/DevAuthCA.cer").to_vec()).unwrap()
});

/// Developer ID - G1 (Expiring 02/01/2027 22:12:15 UTC) Certificate
static DEVELOPER_ID_G1_CERTIFICATE: Lazy<CapturedX509Certificate> = Lazy::new(|| {
    CapturedX509Certificate::from_der(include_bytes!("apple-certs/DeveloperIDCA.cer").to_vec())
        .unwrap()
});

/// Developer ID - G2 (Expiring 09/17/2031 00:00:00 UTC) Certificate
static DEVELOPER_ID_G2_CERTIFICATE: Lazy<CapturedX509Certificate> = Lazy::new(|| {
    CapturedX509Certificate::from_der(include_bytes!("apple-certs/DeveloperIDG2CA.cer").to_vec())
        .unwrap()
});

/// Software Update Certificate
static SOFTWARE_UPDATE_CERTIFICATE: Lazy<CapturedX509Certificate> = Lazy::new(|| {
    CapturedX509Certificate::from_der(
        include_bytes!("apple-certs/AppleSoftwareUpdateCertificationAuthority.cer").to_vec(),
    )
    .unwrap()
});

/// Timestamp Certificate
static TIMESTAMP_CERTIFICATE: Lazy<CapturedX509Certificate> = Lazy::new(|| {
    CapturedX509Certificate::from_der(include_bytes!("apple-certs/AppleTimestampCA.cer").to_vec())
        .unwrap()
});

/// Worldwide Developer Relations - G1 (Expiring 02/07/2023 21:48:47 UTC) Certificate
static WORLD_WIDE_DEVELOPER_RELATIONS_G1_CERTIFICATE: Lazy<CapturedX509Certificate> =
    Lazy::new(|| {
        CapturedX509Certificate::from_der(include_bytes!("apple-certs/AppleWWDRCA.cer").to_vec())
            .unwrap()
    });

/// Worldwide Developer Relations - G2 (Expiring 05/06/2029 23:43:24 UTC) Certificate
static WORLD_WIDE_DEVELOPER_RELATIONS_G2_CERTIFICATE: Lazy<CapturedX509Certificate> =
    Lazy::new(|| {
        CapturedX509Certificate::from_der(include_bytes!("apple-certs/AppleWWDRCAG2.cer").to_vec())
            .unwrap()
    });

/// Worldwide Developer Relations - G3 (Expiring 02/20/2030 00:00:00 UTC) Certificate
static WORLD_WIDE_DEVELOPER_RELATIONS_G3_CERTIFICATE: Lazy<CapturedX509Certificate> =
    Lazy::new(|| {
        CapturedX509Certificate::from_der(include_bytes!("apple-certs/AppleWWDRCAG3.cer").to_vec())
            .unwrap()
    });

/// Worldwide Developer Relations - G4 (Expiring 12/10/2030 00:00:00 UTC) Certificate
static WORLD_WIDE_DEVELOPER_RELATIONS_G4_CERTIFICATE: Lazy<CapturedX509Certificate> =
    Lazy::new(|| {
        CapturedX509Certificate::from_der(include_bytes!("apple-certs/AppleWWDRCAG4.cer").to_vec())
            .unwrap()
    });

/// Worldwide Developer Relations - G5 (Expiring 12/10/2030 00:00:00 UTC) Certificate
static WORLD_WIDE_DEVELOPER_RELATIONS_G5_CERTIFICATE: Lazy<CapturedX509Certificate> =
    Lazy::new(|| {
        CapturedX509Certificate::from_der(include_bytes!("apple-certs/AppleWWDRCAG5.cer").to_vec())
            .unwrap()
    });

/// Worldwide Developer Relations - G6 (Expiring 03/19/2036 00:00:00 UTC) Certificate
static WORLD_WIDE_DEVELOPER_RELATIONS_G6_CERTIFICATE: Lazy<CapturedX509Certificate> =
    Lazy::new(|| {
        CapturedX509Certificate::from_der(include_bytes!("apple-certs/AppleWWDRCAG6.cer").to_vec())
            .unwrap()
    });

/// Worldwide Developer Relations - G7 (Expiring 11/17/2023 20:40:52 UTC) Certificate
static WORLD_WIDE_DEVELOPER_RELATIONS_G7_CERTIFICATE: Lazy<CapturedX509Certificate> =
    Lazy::new(|| {
        CapturedX509Certificate::from_der(include_bytes!("apple-certs/AppleWWDRCAG7.cer").to_vec())
            .unwrap()
    });

/// Worldwide Developer Relations - G8 (Expiring 01/24/2025 00:00:00 UTC) Certificate
static WORLD_WIDE_DEVELOPER_RELATIONS_G8_CERTIFICATE: Lazy<CapturedX509Certificate> =
    Lazy::new(|| {
        CapturedX509Certificate::from_der(include_bytes!("apple-certs/AppleWWDRCAG8.cer").to_vec())
            .unwrap()
    });

/// All known Apple certificates.
static KNOWN_CERTIFICATES: Lazy<Vec<&CapturedX509Certificate>> = Lazy::new(|| {
    vec![
        // We put the 4 roots first, newest to oldest.
        APPLE_ROOT_CA_G3_ROOT_CERTIFICATE.deref(),
        APPLE_ROOT_CA_G2_ROOT_CERTIFICATE.deref(),
        APPLE_INC_ROOT_CERTIFICATE.deref(),
        APPLE_COMPUTER_INC_ROOT_CERTIFICATE.deref(),
        APPLE_IST_CA_2_G1_CERTIFICATE.deref(),
        APPLE_IST_CA_8_G1_CERTIFICATE.deref(),
        APPLICATION_INTEGRATION_CERTIFICATE.deref(),
        APPLICATION_INTEGRATION_2_CERTIFICATE.deref(),
        APPLICATION_INTEGRATION_G3_CERTIFICATE.deref(),
        APPLE_APPLICATION_INTEGRATION_CA_5_G1_CERTIFICATE.deref(),
        APPLE_APPLICATION_INTEGRATION_CA_7_G1_CERTIFICATE.deref(),
        DEVELOPER_AUTHENTICATION_CERTIFICATE.deref(),
        DEVELOPER_ID_G1_CERTIFICATE.deref(),
        DEVELOPER_ID_G2_CERTIFICATE.deref(),
        SOFTWARE_UPDATE_CERTIFICATE.deref(),
        TIMESTAMP_CERTIFICATE.deref(),
        WORLD_WIDE_DEVELOPER_RELATIONS_G1_CERTIFICATE.deref(),
        WORLD_WIDE_DEVELOPER_RELATIONS_G2_CERTIFICATE.deref(),
        WORLD_WIDE_DEVELOPER_RELATIONS_G3_CERTIFICATE.deref(),
        WORLD_WIDE_DEVELOPER_RELATIONS_G4_CERTIFICATE.deref(),
        WORLD_WIDE_DEVELOPER_RELATIONS_G5_CERTIFICATE.deref(),
        WORLD_WIDE_DEVELOPER_RELATIONS_G6_CERTIFICATE.deref(),
        WORLD_WIDE_DEVELOPER_RELATIONS_G7_CERTIFICATE.deref(),
        WORLD_WIDE_DEVELOPER_RELATIONS_G8_CERTIFICATE.deref(),
    ]
});

static KNOWN_ROOTS: Lazy<Vec<&CapturedX509Certificate>> = Lazy::new(|| {
    vec![
        APPLE_ROOT_CA_G3_ROOT_CERTIFICATE.deref(),
        APPLE_ROOT_CA_G2_ROOT_CERTIFICATE.deref(),
        APPLE_INC_ROOT_CERTIFICATE.deref(),
        APPLE_COMPUTER_INC_ROOT_CERTIFICATE.deref(),
    ]
});

/// Defines all known Apple certificates.
///
/// This crate embeds the raw certificate data for the various known
/// Apple certificate authorities, as advertised at
/// <https://www.apple.com/certificateauthority/>.
///
/// This enumeration defines all the ones we know about. Instances can
/// be dereferenced into concrete [CapturedX509Certificate] to get at the underlying
/// certificate and access its metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KnownCertificate {
    /// Apple Computer, Inc. Root Certificate.
    ///
    /// C = US, O = "Apple Computer, Inc.", OU = Apple Computer Certificate Authority, CN = Apple Root Certificate Authority
    AppleComputerIncRoot,

    /// Apple Inc. Root Certificate
    ///
    /// C = US, O = Apple Inc., OU = Apple Certification Authority, CN = Apple Root CA
    AppleRootCa,

    /// Apple Root CA - G2 Root Certificate
    ///
    /// CN = Apple Root CA - G2, OU = Apple Certification Authority, O = Apple Inc., C = US
    AppleRootCaG2Root,

    /// Apple Root CA - G3 Root Certificate
    ///
    /// CN = Apple Root CA - G3, OU = Apple Certification Authority, O = Apple Inc., C = US
    AppleRootCaG3Root,

    /// Apple IST CA 2 - G1 Certificate
    ///
    /// CN = Apple IST CA 2 - G1, OU = Certification Authority, O = Apple Inc., C = US
    AppleIstCa2G1,

    /// Apple IST CA 8 - G1 Certificate
    ///
    /// CN = Apple IST CA 8 - G1, OU = Certification Authority, O = Apple Inc., C = US
    AppleIstCa8G1,

    /// Application Integration Certificate
    ///
    /// C = US, O = Apple Inc., OU = Apple Certification Authority, CN = Apple Application Integration Certification Authority
    ApplicationIntegration,

    /// Application Integration 2 Certificate
    ///
    /// CN = Apple Application Integration 2 Certification Authority, OU = Apple Certification Authority, O = Apple Inc., C = US
    ApplicationIntegration2,

    /// Application Integration - G3 Certificate
    ///
    /// CN = Apple Application Integration CA - G3, OU = Apple Certification Authority, O = Apple Inc., C = US
    ApplicationIntegrationG3,

    /// Apple Application Integration CA 5 - G1 Certificate
    ///
    /// CN = Apple Application Integration CA 5 - G1, OU = Apple Certification Authority, O = Apple Inc., C = US
    AppleApplicationIntegrationCa5G1,

    /// Apple Application Integration CA 7 - G1 Certificate
    ///
    /// CN=Apple Application Integration CA 7 - G1, OU=Apple Certification Authority, O=Apple Inc., C=US
    AppleApplicationIntegrationCa7G1,

    /// Developer Authentication Certificate
    ///
    /// CN = Developer Authentication Certification Authority, OU = Apple Worldwide Developer Relations, O = Apple Inc., C = US
    DeveloperAuthentication,

    /// Developer ID - G1  Certificate
    ///
    /// CN = Developer ID Certification Authority, OU = Apple Certification Authority, O = Apple Inc., C = US
    DeveloperIdG1,

    /// Developer ID - G2 Certificate.
    ///
    /// CN = Developer ID Certification Authority, OU = G2, O = Apple Inc., C = US
    DeveloperIdG2,

    /// Software Update Certificate
    ///
    /// CN = Apple Software Update Certification Authority, OU = Certification Authority, O = Apple Inc., C = US
    SoftwareUpdate,

    /// Timestamp Certificate
    ///
    /// CN = Apple Timestamp Certification Authority, OU = Apple Certification Authority, O = Apple Inc., C = US
    Timestamp,

    /// Worldwide Developer Relations - G1 (Expiring 02/07/2023 21:48:47 UTC) Certificate
    ///
    /// C = US, O = Apple Inc., OU = Apple Worldwide Developer Relations, CN = Apple Worldwide Developer Relations Certification Authority
    WwdrG1,

    /// Worldwide Developer Relations - G2 (Expiring 05/06/2029 23:43:24 UTC) Certificate
    ///
    /// CN = Apple Worldwide Developer Relations CA - G2, OU = Apple Certification Authority, O = Apple Inc., C = US
    WwdrG2,

    /// Worldwide Developer Relations - G3 (Expiring 02/20/2030 00:00:00 UTC) Certificate
    ///
    /// CN = Apple Worldwide Developer Relations Certification Authority, OU = G3, O = Apple Inc., C = US
    WwdrG3,

    /// Worldwide Developer Relations - G4 (Expiring 12/10/2030 00:00:00 UTC) Certificate
    ///
    /// CN = Apple Worldwide Developer Relations Certification Authority, OU = G4, O = Apple Inc., C = US
    WwdrG4,

    /// Worldwide Developer Relations - G5 (Expiring 12/10/2030 00:00:00 UTC) Certificate
    ///
    /// CN = Apple Worldwide Developer Relations Certification Authority, OU = G5, O = Apple Inc., C = US
    WwdrG5,

    /// Worldwide Developer Relations - G6 (Expiring 03/19/2036 00:00:00 UTC) Certificate
    ///
    /// CN = Apple Worldwide Developer Relations Certification Authority, OU = G6, O = Apple Inc., C = US
    WwdrG6,

    /// Worldwide Developer Relations - G7 (Expiring 11/17/2023 20:40:52 UTC)
    ///
    /// C=US, O=Apple Inc., OU=G7, CN=Apple Worldwide Developer Relations Certification Authority
    WwdrG7,

    /// Worldwide Developer Relations - G8 (Expiring 01/24/2025 00:00:00 UTC)
    ///
    /// C=US, O=Apple Inc., OU=G8, CN=Apple Worldwide Developer Relations Certification Authority
    WwdrG8,
}

impl Deref for KnownCertificate {
    type Target = CapturedX509Certificate;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::AppleComputerIncRoot => APPLE_COMPUTER_INC_ROOT_CERTIFICATE.deref(),
            Self::AppleRootCa => APPLE_INC_ROOT_CERTIFICATE.deref(),
            Self::AppleRootCaG2Root => APPLE_ROOT_CA_G2_ROOT_CERTIFICATE.deref(),
            Self::AppleRootCaG3Root => APPLE_ROOT_CA_G3_ROOT_CERTIFICATE.deref(),
            Self::AppleIstCa2G1 => APPLE_IST_CA_2_G1_CERTIFICATE.deref(),
            Self::AppleIstCa8G1 => APPLE_IST_CA_8_G1_CERTIFICATE.deref(),
            Self::ApplicationIntegration => APPLICATION_INTEGRATION_CERTIFICATE.deref(),
            Self::ApplicationIntegration2 => APPLICATION_INTEGRATION_2_CERTIFICATE.deref(),
            Self::ApplicationIntegrationG3 => APPLICATION_INTEGRATION_G3_CERTIFICATE.deref(),
            Self::AppleApplicationIntegrationCa5G1 => {
                APPLE_APPLICATION_INTEGRATION_CA_5_G1_CERTIFICATE.deref()
            }
            Self::AppleApplicationIntegrationCa7G1 => {
                APPLE_APPLICATION_INTEGRATION_CA_7_G1_CERTIFICATE.deref()
            }
            Self::DeveloperAuthentication => DEVELOPER_AUTHENTICATION_CERTIFICATE.deref(),
            Self::DeveloperIdG1 => DEVELOPER_ID_G1_CERTIFICATE.deref(),
            Self::DeveloperIdG2 => DEVELOPER_ID_G2_CERTIFICATE.deref(),
            Self::SoftwareUpdate => SOFTWARE_UPDATE_CERTIFICATE.deref(),
            Self::Timestamp => TIMESTAMP_CERTIFICATE.deref(),
            Self::WwdrG1 => WORLD_WIDE_DEVELOPER_RELATIONS_G1_CERTIFICATE.deref(),
            Self::WwdrG2 => WORLD_WIDE_DEVELOPER_RELATIONS_G2_CERTIFICATE.deref(),
            Self::WwdrG3 => WORLD_WIDE_DEVELOPER_RELATIONS_G3_CERTIFICATE.deref(),
            Self::WwdrG4 => WORLD_WIDE_DEVELOPER_RELATIONS_G4_CERTIFICATE.deref(),
            Self::WwdrG5 => WORLD_WIDE_DEVELOPER_RELATIONS_G5_CERTIFICATE.deref(),
            Self::WwdrG6 => WORLD_WIDE_DEVELOPER_RELATIONS_G6_CERTIFICATE.deref(),
            Self::WwdrG7 => WORLD_WIDE_DEVELOPER_RELATIONS_G7_CERTIFICATE.deref(),
            Self::WwdrG8 => WORLD_WIDE_DEVELOPER_RELATIONS_G8_CERTIFICATE.deref(),
        }
    }
}

impl AsRef<CapturedX509Certificate> for KnownCertificate {
    fn as_ref(&self) -> &CapturedX509Certificate {
        self.deref()
    }
}

impl TryFrom<&CapturedX509Certificate> for KnownCertificate {
    type Error = &'static str;

    fn try_from(cert: &CapturedX509Certificate) -> Result<Self, Self::Error> {
        let want = cert.constructed_data();

        match cert.constructed_data() {
            _ if APPLE_ROOT_CA_G3_ROOT_CERTIFICATE.constructed_data() == want => {
                Ok(Self::AppleRootCaG3Root)
            }
            _ if APPLE_ROOT_CA_G2_ROOT_CERTIFICATE.constructed_data() == want => {
                Ok(Self::AppleRootCaG2Root)
            }
            _ if APPLE_INC_ROOT_CERTIFICATE.constructed_data() == want => Ok(Self::AppleRootCa),
            _ if APPLE_COMPUTER_INC_ROOT_CERTIFICATE.constructed_data() == want => {
                Ok(Self::AppleComputerIncRoot)
            }
            _ if APPLE_IST_CA_2_G1_CERTIFICATE.constructed_data() == want => {
                Ok(Self::AppleIstCa2G1)
            }
            _ if APPLE_IST_CA_8_G1_CERTIFICATE.constructed_data() == want => {
                Ok(Self::AppleIstCa8G1)
            }
            _ if APPLICATION_INTEGRATION_CERTIFICATE.constructed_data() == want => {
                Ok(Self::ApplicationIntegration)
            }
            _ if APPLICATION_INTEGRATION_2_CERTIFICATE.constructed_data() == want => {
                Ok(Self::ApplicationIntegration2)
            }
            _ if APPLICATION_INTEGRATION_G3_CERTIFICATE.constructed_data() == want => {
                Ok(Self::ApplicationIntegrationG3)
            }
            _ if APPLE_APPLICATION_INTEGRATION_CA_5_G1_CERTIFICATE.constructed_data() == want => {
                Ok(Self::AppleApplicationIntegrationCa5G1)
            }
            _ if APPLE_APPLICATION_INTEGRATION_CA_7_G1_CERTIFICATE.constructed_data() == want => {
                Ok(Self::AppleApplicationIntegrationCa7G1)
            }
            _ if DEVELOPER_AUTHENTICATION_CERTIFICATE.constructed_data() == want => {
                Ok(Self::DeveloperAuthentication)
            }
            _ if DEVELOPER_ID_G1_CERTIFICATE.constructed_data() == want => Ok(Self::DeveloperIdG1),
            _ if DEVELOPER_ID_G2_CERTIFICATE.constructed_data() == want => Ok(Self::DeveloperIdG2),
            _ if SOFTWARE_UPDATE_CERTIFICATE.constructed_data() == want => Ok(Self::SoftwareUpdate),
            _ if TIMESTAMP_CERTIFICATE.constructed_data() == want => Ok(Self::Timestamp),
            _ if WORLD_WIDE_DEVELOPER_RELATIONS_G1_CERTIFICATE.constructed_data() == want => {
                Ok(Self::WwdrG1)
            }
            _ if WORLD_WIDE_DEVELOPER_RELATIONS_G2_CERTIFICATE.constructed_data() == want => {
                Ok(Self::WwdrG2)
            }
            _ if WORLD_WIDE_DEVELOPER_RELATIONS_G3_CERTIFICATE.constructed_data() == want => {
                Ok(Self::WwdrG3)
            }
            _ if WORLD_WIDE_DEVELOPER_RELATIONS_G4_CERTIFICATE.constructed_data() == want => {
                Ok(Self::WwdrG4)
            }
            _ if WORLD_WIDE_DEVELOPER_RELATIONS_G5_CERTIFICATE.constructed_data() == want => {
                Ok(Self::WwdrG5)
            }
            _ if WORLD_WIDE_DEVELOPER_RELATIONS_G6_CERTIFICATE.constructed_data() == want => {
                Ok(Self::WwdrG6)
            }
            _ if WORLD_WIDE_DEVELOPER_RELATIONS_G7_CERTIFICATE.constructed_data() == want => {
                Ok(Self::WwdrG7)
            }
            _ if WORLD_WIDE_DEVELOPER_RELATIONS_G8_CERTIFICATE.constructed_data() == want => {
                Ok(Self::WwdrG8)
            }
            _ => Err("certificate not found"),
        }
    }
}

impl KnownCertificate {
    /// Obtain a slice of all known [KnownCertificate].
    ///
    /// If you want to iterate over all certificates and find one, you can use
    /// this.
    pub fn all() -> &'static [&'static CapturedX509Certificate] {
        KNOWN_CERTIFICATES.deref().as_ref()
    }

    /// All of Apple's known root certificate authority certificates.
    pub fn all_roots() -> &'static [&'static CapturedX509Certificate] {
        KNOWN_ROOTS.deref()
    }
}

#[cfg(test)]
mod test {
    use {
        super::*,
        crate::certificate::{AppleCertificate, CertificateAuthorityExtension},
    };

    #[test]
    fn all() {
        for cert in KnownCertificate::all() {
            assert!(cert.subject_common_name().is_some());
            assert!(KnownCertificate::try_from(*cert).is_ok());
        }
    }

    #[test]
    fn apple_root_ca() {
        assert!(APPLE_INC_ROOT_CERTIFICATE.is_apple_root_ca());
        assert!(!APPLE_INC_ROOT_CERTIFICATE.is_apple_intermediate_ca());
        assert!(APPLE_COMPUTER_INC_ROOT_CERTIFICATE.is_apple_root_ca());
        assert!(!APPLE_COMPUTER_INC_ROOT_CERTIFICATE.is_apple_intermediate_ca());
        assert!(APPLE_ROOT_CA_G2_ROOT_CERTIFICATE.is_apple_root_ca());
        assert!(!APPLE_ROOT_CA_G2_ROOT_CERTIFICATE.is_apple_intermediate_ca());
        assert!(APPLE_ROOT_CA_G3_ROOT_CERTIFICATE.is_apple_root_ca());
        assert!(!APPLE_ROOT_CA_G3_ROOT_CERTIFICATE.is_apple_intermediate_ca());

        assert!(!WORLD_WIDE_DEVELOPER_RELATIONS_G3_CERTIFICATE.is_apple_root_ca());
        assert!(WORLD_WIDE_DEVELOPER_RELATIONS_G3_CERTIFICATE.is_apple_intermediate_ca());

        let wanted = [
            APPLE_INC_ROOT_CERTIFICATE.deref(),
            APPLE_COMPUTER_INC_ROOT_CERTIFICATE.deref(),
            APPLE_ROOT_CA_G2_ROOT_CERTIFICATE.deref(),
            APPLE_ROOT_CA_G3_ROOT_CERTIFICATE.deref(),
        ];

        for cert in KnownCertificate::all() {
            if wanted.contains(cert) {
                continue;
            }

            assert!(!cert.is_apple_root_ca());
            assert!(cert.is_apple_intermediate_ca());
        }
    }

    #[test]
    fn intermediate_have_apple_ca_extension() {
        // All intermediate certs should have OIDs identifying them as such.
        for cert in KnownCertificate::all()
            .iter()
            .filter(|cert| !cert.is_apple_root_ca())
            // There are some intermediate certificates signed by GeoTrust. Filter them out
            // as well.
            .filter(|cert| {
                cert.issuer_name()
                    .iter_common_name()
                    .all(|atv| !atv.to_string().unwrap().contains("GeoTrust"))
            })
        {
            assert!(!cert.apple_ca_extensions().is_empty());
        }

        // Let's spot check a few.
        assert_eq!(
            KnownCertificate::DeveloperIdG1
                .apple_ca_extensions()
                .first(),
            Some(&CertificateAuthorityExtension::DeveloperId)
        );
        assert_eq!(
            KnownCertificate::DeveloperIdG2
                .apple_ca_extensions()
                .first(),
            Some(&CertificateAuthorityExtension::DeveloperId)
        );
        assert_eq!(
            KnownCertificate::WwdrG1.apple_ca_extensions().first(),
            Some(&CertificateAuthorityExtension::AppleWorldwideDeveloperRelations)
        );
        assert_eq!(
            KnownCertificate::WwdrG2.apple_ca_extensions().first(),
            Some(&CertificateAuthorityExtension::AppleWorldwideDeveloperRelationsG2)
        );
        assert_eq!(
            KnownCertificate::WwdrG3.apple_ca_extensions().first(),
            Some(&CertificateAuthorityExtension::AppleWorldwideDeveloperRelations)
        );
        assert_eq!(
            KnownCertificate::WwdrG4.apple_ca_extensions().first(),
            Some(&CertificateAuthorityExtension::AppleWorldwideDeveloperRelations)
        );
        assert_eq!(
            KnownCertificate::WwdrG5.apple_ca_extensions().first(),
            Some(&CertificateAuthorityExtension::AppleWorldwideDeveloperRelations)
        );
        assert_eq!(
            KnownCertificate::WwdrG6.apple_ca_extensions().first(),
            Some(&CertificateAuthorityExtension::AppleWorldwideDeveloperRelations)
        );
        assert_eq!(
            KnownCertificate::WwdrG7.apple_ca_extensions().first(),
            Some(&CertificateAuthorityExtension::AppleWorldwideDeveloperRelations)
        );
        assert_eq!(
            KnownCertificate::WwdrG8.apple_ca_extensions().first(),
            Some(&CertificateAuthorityExtension::AppleWorldwideDeveloperRelations)
        );
    }

    #[test]
    fn chaining() {
        let relevant = KnownCertificate::all()
            .iter()
            .filter(|cert| {
                cert.issuer_name()
                    .iter_common_name()
                    .all(|atv| !atv.to_string().unwrap().contains("GeoTrust"))
            })
            .filter(|cert| {
                cert.constructed_data() != APPLICATION_INTEGRATION_G3_CERTIFICATE.constructed_data()
                    && cert.constructed_data()
                        != APPLE_APPLICATION_INTEGRATION_CA_5_G1_CERTIFICATE.constructed_data()
            });

        for cert in relevant {
            let chain = cert.resolve_signing_chain(KnownCertificate::all().iter().copied());
            let apple_chain = cert.apple_issuing_chain();
            assert_eq!(chain.len(), apple_chain.len());
        }
    }
}
