mod certificate;
mod error;

pub use certificate::{CertificateProfile, create_self_signed_code_signing_certificate};
pub use error::AppleCodesignError;
