// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use once_cell::sync::OnceCell;
use regex::Regex;
use std::{collections::BTreeSet, path::Path, process::Command};
use x509_certificate::certificate::X509Certificate;

use crate::{Error, Result};

fn get_pem_list(keychain_path: &Path, name_substr: &str) -> Result<std::process::Output> {
  Command::new("security")
    .arg("find-certificate")
    .args(["-p", "-a"])
    .arg("-c")
    .arg(name_substr)
    .arg(keychain_path)
    .stdin(os_pipe::dup_stdin().unwrap())
    .stderr(os_pipe::dup_stderr().unwrap())
    .output()
    .map_err(|error| Error::CommandFailed {
      command: "security find-certificate".to_string(),
      error,
    })
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Team {
  pub name: String,
  pub certificate_name: String,
  pub id: String,
  pub cert_prefix: &'static str,
}

impl Team {
  fn from_x509(cert_prefix: &'static str, cert: X509Certificate) -> Result<Self> {
    let common_name = cert
      .subject_common_name()
      .ok_or(Error::CertificateMissingCommonName)?;

    let organization = cert
      .subject_name()
      .iter_organization()
      .next()
      .and_then(|v| v.to_string().ok());

    let name = if let Some(organization) = organization {
      println!("found cert {common_name:?} with organization {organization:?}");
      organization
    } else {
      println!(
                "found cert {common_name:?} but failed to get organization; falling back to displaying common name"
            );
      static APPLE_DEV: OnceCell<Regex> = OnceCell::new();
      APPLE_DEV.get_or_init(|| Regex::new(r"Apple Develop\w+: (.*) \(.+\)").unwrap())
                .captures(&common_name)
                .map(|caps| caps[1].to_owned())
                .unwrap_or_else(|| {
                    println!("regex failed to capture nice part of name in cert {common_name:?}; falling back to displaying full name");
                    common_name.clone()
                })
    };

    let id = cert
      .subject_name()
      .iter_organizational_unit()
      .next()
      .and_then(|v| v.to_string().ok())
      .ok_or_else(|| Error::CertificateMissingOrganizationUnit {
        common_name: common_name.clone(),
      })?;

    Ok(Self {
      name,
      certificate_name: common_name,
      id,
      cert_prefix,
    })
  }

  pub fn certificate_name(&self) -> String {
    self.certificate_name.clone()
  }
}

pub fn list(keychain_path: &Path) -> Result<Vec<Team>> {
  let certs = {
    let mut certs = Vec::new();
    for cert_prefix in [
      "iOS Distribution:",
      "Apple Distribution:",
      "Developer ID Application:",
      "Mac App Distribution:",
      "Apple Development:",
      "iOS App Development:",
      "Mac Development:",
    ] {
      let pem_list_out = get_pem_list(keychain_path, cert_prefix)?;
      let cert_list = X509Certificate::from_pem_multiple(pem_list_out.stdout)
        .map_err(|error| Error::X509Certificate { error })?;
      certs.extend(cert_list.into_iter().map(|cert| (cert_prefix, cert)));
    }
    certs
  };
  Ok(
    certs
      .into_iter()
      .flat_map(|(cert_prefix, cert)| {
        Team::from_x509(cert_prefix, cert).map_err(|err| {
          log::error!("{err}");
          err
        })
      })
      // Silly way to sort this and ensure no dupes
      .collect::<BTreeSet<_>>()
      .into_iter()
      .collect(),
  )
}
