use std::{
  fs::File,
  io::prelude::*,
  path::PathBuf,
  process::{Command, Stdio},
};

use crate::{bundle::common, Settings};
use regex::Regex;

// Import certificate from ENV variables.
// APPLE_CERTIFICATE is the p12 certificate base64 encoded.
// By example you can use; openssl base64 -in MyCertificate.p12 -out MyCertificate-base64.txt
// Then use the value of the base64 in APPLE_CERTIFICATE env variable.
// You need to set APPLE_CERTIFICATE_PASSWORD to the password you set when youy exported your certificate.
// https://help.apple.com/xcode/mac/current/#/dev154b28f09 see: `Export a signing certificate`
pub fn setup_keychain_if_needed() -> crate::Result<()> {
  match (
    std::env::var_os("APPLE_CERTIFICATE"),
    std::env::var_os("APPLE_CERTIFICATE_PASSWORD"),
  ) {
    (Some(certificate_encoded), Some(certificate_password)) => {
      // we delete any previous version of our keychain if present
      delete_keychain_if_needed();
      common::print_info("setup keychain from environment variables...")?;

      let key_chain_id = "tauri-build.keychain";
      let key_chain_name = "tauri-build";
      let tmp_dir = tempfile::tempdir()?;
      let cert_path = tmp_dir
        .path()
        .join("cert.p12")
        .to_string_lossy()
        .to_string();
      let cert_path_tmp = tmp_dir
        .path()
        .join("cert.p12.tmp")
        .to_string_lossy()
        .to_string();
      let certificate_encoded = certificate_encoded
        .to_str()
        .expect("failed to convert APPLE_CERTIFICATE to string")
        .as_bytes();

      let certificate_password = certificate_password
        .to_str()
        .expect("failed to convert APPLE_CERTIFICATE_PASSWORD to string")
        .to_string();

      // as certificate contain whitespace decoding may be broken
      // https://github.com/marshallpierce/rust-base64/issues/105
      // we'll use builtin base64 command from the OS
      let mut tmp_cert = File::create(cert_path_tmp.clone())?;
      tmp_cert.write_all(certificate_encoded)?;

      let decode_certificate = Command::new("base64")
        .args(vec!["--decode", "-i", &cert_path_tmp, "-o", &cert_path])
        .stderr(Stdio::piped())
        .status()?;

      if !decode_certificate.success() {
        return Err(anyhow::anyhow!("failed to decode certificate",).into());
      }

      let create_key_chain = Command::new("security")
        .args(vec!["create-keychain", "-p", key_chain_name, key_chain_id])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status()?;

      if !create_key_chain.success() {
        return Err(anyhow::anyhow!("failed to create keychain",).into());
      }

      let set_default_keychain = Command::new("security")
        .args(vec!["default-keychain", "-s", key_chain_id])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status()?;

      if !set_default_keychain.success() {
        return Err(anyhow::anyhow!("failed to set default keychain",).into());
      }

      let unlock_keychain = Command::new("security")
        .args(vec!["unlock-keychain", "-p", key_chain_name, key_chain_id])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status()?;

      if !unlock_keychain.success() {
        return Err(anyhow::anyhow!("failed to set unlock keychain",).into());
      }

      let import_certificate = Command::new("security")
        .arg("import")
        .arg(cert_path)
        .arg("-k")
        .arg(key_chain_id)
        .arg("-P")
        .arg(certificate_password)
        .arg("-T")
        .arg("/usr/bin/codesign")
        .arg("-T")
        .arg("/usr/bin/pkgbuild")
        .arg("-T")
        .arg("/usr/bin/productbuild")
        .stderr(Stdio::inherit())
        .output()?;

      if !import_certificate.status.success() {
        return Err(
          anyhow::anyhow!(format!(
            "failed to import keychain certificate {:?}",
            std::str::from_utf8(&import_certificate.stdout)
          ))
          .into(),
        );
      }

      let settings_keychain = Command::new("security")
        .args(vec![
          "set-keychain-settings",
          "-t",
          "3600",
          "-u",
          key_chain_id,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status()?;

      if !settings_keychain.success() {
        return Err(anyhow::anyhow!("failed to set keychain settings",).into());
      }

      let partition_list = Command::new("security")
        .args(vec![
          "set-key-partition-list",
          "-S",
          "apple-tool:,apple:,codesign:",
          "-s",
          "-k",
          key_chain_name,
          key_chain_id,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status()?;

      if !partition_list.success() {
        return Err(anyhow::anyhow!("failed to set keychain settings",).into());
      }

      Ok(())
    }
    // skip it
    _ => Ok(()),
  }
}

pub fn delete_keychain_if_needed() {
  if let (Some(_cert), Some(_password)) = (
    std::env::var_os("APPLE_CERTIFICATE"),
    std::env::var_os("APPLE_CERTIFICATE_PASSWORD"),
  ) {
    let key_chain_id = "tauri-build.keychain";
    // delete keychain if needed and skip any error
    let _result = Command::new("security")
      .arg("delete-keychain")
      .arg(key_chain_id)
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .status();
  }
}

pub fn sign(
  path_to_sign: PathBuf,
  identity: &str,
  settings: &Settings,
  is_an_executable: bool,
) -> crate::Result<()> {
  common::print_info(format!(r#"signing with identity "{}""#, identity).as_str())?;
  let mut args = vec!["--force", "-s", identity];
  if let Some(entitlements_path) = &settings.macos().entitlements {
    common::print_info(format!("using entitlements file at {}", entitlements_path).as_str())?;
    args.push("--entitlements");
    args.push(entitlements_path);
  }

  if is_an_executable {
    args.push("--options");
    args.push("runtime");
  }

  if path_to_sign.is_dir() {
    args.push("--deep");
  }

  let status = Command::new("codesign")
    .args(args)
    .arg(path_to_sign.to_string_lossy().to_string())
    .status()?;

  if !status.success() {
    return Err(anyhow::anyhow!("failed to sign app").into());
  }

  Ok(())
}

pub fn notarize(
  app_bundle_path: PathBuf,
  auth_args: Vec<String>,
  settings: &Settings,
) -> crate::Result<()> {
  let identifier = settings.bundle_identifier();

  let bundle_stem = app_bundle_path
    .file_stem()
    .expect("failed to get bundle filename");

  let tmp_dir = tempfile::tempdir()?;
  let zip_path = tmp_dir
    .path()
    .join(format!("{}.zip", bundle_stem.to_string_lossy()));
  let zip_args = vec![
    "-c",
    "-k",
    "--keepParent",
    "--sequesterRsrc",
    app_bundle_path
      .to_str()
      .expect("failed to convert bundle_path to string"),
    zip_path
      .to_str()
      .expect("failed to convert zip_path to string"),
  ];

  // use ditto to create a PKZip almost identical to Finder
  // this remove almost 99% of false alarm in notarization
  let zip_app = Command::new("ditto")
    .args(zip_args)
    .stderr(Stdio::inherit())
    .status()?;

  if !zip_app.success() {
    return Err(anyhow::anyhow!("failed to zip app with ditto").into());
  }

  // sign the zip file
  if let Some(identity) = &settings.macos().signing_identity {
    sign(zip_path.clone(), identity, &settings, false)?;
  };

  let notarize_args = vec![
    "altool",
    "--notarize-app",
    "-f",
    zip_path
      .to_str()
      .expect("failed to convert zip_path to string"),
    "--primary-bundle-id",
    identifier,
  ];
  common::print_info("notarizing app")?;
  let output = Command::new("xcrun")
    .args(notarize_args)
    .args(auth_args.clone())
    .stderr(Stdio::inherit())
    .output()?;

  if !output.status.success() {
    return Err(
      anyhow::anyhow!(format!(
        "failed to upload app to Apple's notarization servers. {}",
        std::str::from_utf8(&output.stdout)?
      ))
      .into(),
    );
  }

  let stdout = std::str::from_utf8(&output.stdout)?;
  if let Some(uuid) = Regex::new(r"\nRequestUUID = (.+?)\n")?
    .captures_iter(stdout)
    .next()
  {
    common::print_info("notarization started; waiting for Apple response...")?;
    let uuid = uuid[1].to_string();
    get_notarization_status(uuid, auth_args)?;
    staple_app(app_bundle_path.clone())?;
  } else {
    return Err(
      anyhow::anyhow!(format!(
        "failed to parse RequestUUID from upload output. {}",
        stdout
      ))
      .into(),
    );
  }

  Ok(())
}

fn staple_app(mut app_bundle_path: PathBuf) -> crate::Result<()> {
  let app_bundle_path_clone = app_bundle_path.clone();
  let filename = app_bundle_path_clone
    .file_name()
    .expect("failed to get bundle filename")
    .to_str()
    .expect("failed to convert bundle filename to string");

  app_bundle_path.pop();

  let output = Command::new("xcrun")
    .args(vec!["stapler", "staple", "-v", filename])
    .current_dir(app_bundle_path)
    .stderr(Stdio::inherit())
    .output()?;

  if !output.status.success() {
    Err(
      anyhow::anyhow!(format!(
        "failed to staple app. {}",
        std::str::from_utf8(&output.stdout)?
      ))
      .into(),
    )
  } else {
    Ok(())
  }
}

fn get_notarization_status(uuid: String, auth_args: Vec<String>) -> crate::Result<()> {
  std::thread::sleep(std::time::Duration::from_secs(10));
  let output = Command::new("xcrun")
    .args(vec!["altool", "--notarization-info", &uuid])
    .args(auth_args.clone())
    .stderr(Stdio::inherit())
    .output()?;

  if !output.status.success() {
    get_notarization_status(uuid, auth_args)
  } else {
    let stdout = std::str::from_utf8(&output.stdout)?;
    if let Some(status) = Regex::new(r"\n *Status: (.+?)\n")?
      .captures_iter(stdout)
      .next()
    {
      let status = status[1].to_string();
      if status == "in progress" {
        get_notarization_status(uuid, auth_args)
      } else if status == "invalid" {
        Err(
          anyhow::anyhow!(format!(
            "Apple failed to notarize your app. {}",
            std::str::from_utf8(&output.stdout)?
          ))
          .into(),
        )
      } else if status != "success" {
        Err(
          anyhow::anyhow!(format!(
            "Unknown notarize status {}. {}",
            status,
            std::str::from_utf8(&output.stdout)?
          ))
          .into(),
        )
      } else {
        Ok(())
      }
    } else {
      get_notarization_status(uuid, auth_args)
    }
  }
}

pub fn notarize_auth_args() -> crate::Result<Vec<String>> {
  match (
    std::env::var_os("APPLE_ID"),
    std::env::var_os("APPLE_PASSWORD"),
  ) {
    (Some(apple_id), Some(apple_password)) => {
      let apple_id = apple_id
        .to_str()
        .expect("failed to convert APPLE_ID to string")
        .to_string();
      let apple_password = apple_password
        .to_str()
        .expect("failed to convert APPLE_PASSWORD to string")
        .to_string();
      Ok(vec![
        "-u".to_string(),
        apple_id,
        "-p".to_string(),
        apple_password,
      ])
    }
    _ => {
      match (std::env::var_os("APPLE_API_KEY"), std::env::var_os("APPLE_API_ISSUER")) {
        (Some(api_key), Some(api_issuer)) => {
          let api_key = api_key.to_str().expect("failed to convert APPLE_API_KEY to string").to_string();
          let api_issuer = api_issuer.to_str().expect("failed to convert APPLE_API_ISSUER to string").to_string();
          Ok(vec!["--apiKey".to_string(), api_key, "--apiIssuer".to_string(), api_issuer])
        },
        _ => Err(anyhow::anyhow!("no APPLE_ID & APPLE_PASSWORD or APPLE_API_KEY & APPLE_API_ISSUER environment variables found").into())
      }
    }
  }
}
