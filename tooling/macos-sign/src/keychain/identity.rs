// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use anyhow::Context;
use once_cell_regex::regex;
use std::{collections::BTreeSet, path::Path, process::Command};
use x509_certificate::certificate::X509Certificate;

use crate::Result;

fn get_pem_list(keychain_path: &Path, name_substr: &str) -> std::io::Result<std::process::Output> {
  Command::new("security")
    .arg("find-certificate")
    .args(["-p", "-a"])
    .arg("-c")
    .arg(name_substr)
    .arg(keychain_path)
    .stdin(os_pipe::dup_stdin().unwrap())
    .stderr(os_pipe::dup_stderr().unwrap())
    .output()
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
      .ok_or_else(|| anyhow::anyhow!("skipping cert, missing common name"))?;

    let organization = cert
      .subject_name()
      .iter_organization()
      .next()
      .and_then(|v| v.to_string().ok());

    let name = if let Some(organization) = organization {
      println!(
        "found cert {:?} with organization {:?}",
        common_name, organization
      );
      organization
    } else {
      println!(
        "found cert {:?} but failed to get organization; falling back to displaying common name",
        common_name
      );
      regex!(r"Apple Develop\w+: (.*) \(.+\)")
                .captures(&common_name)
                .map(|caps| caps[1].to_owned())
                .unwrap_or_else(|| {
                    println!("regex failed to capture nice part of name in cert {:?}; falling back to displaying full name", common_name);
                    common_name.clone()
                })
    };

    let id = cert
      .subject_name()
      .iter_organizational_unit()
      .next()
      .and_then(|v| v.to_string().ok())
      .ok_or_else(|| anyhow::anyhow!("skipping cert {common_name}: missing Organization Unit"))?;

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
      let pem_list_out =
        get_pem_list(keychain_path, cert_prefix).context("Failed to call `security` command")?;
      let cert_list = X509Certificate::from_pem_multiple(pem_list_out.stdout)
        .context("Failed to parse X509 cert")?;
      certs.extend(cert_list.into_iter().map(|cert| (cert_prefix, cert)));
    }
    certs
  };
  Ok(
    certs
      .into_iter()
      .flat_map(|(cert_prefix, cert)| {
        Team::from_x509(cert_prefix, cert).map_err(|err| {
          eprintln!("{}", err);
          err
        })
      })
      // Silly way to sort this and ensure no dupes
      .collect::<BTreeSet<_>>()
      .into_iter()
      .collect(),
  )
}
