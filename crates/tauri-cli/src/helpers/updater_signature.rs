// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use base64::Engine;
use minisign::{
  sign, KeyPair as KP, PublicKey, PublicKeyBox, SecretKey, SecretKeyBox, SignatureBox,
};
use std::{
  fs::{self, File, OpenOptions},
  io::{BufReader, BufWriter, Write},
  path::{Path, PathBuf},
  str,
  time::{SystemTime, UNIX_EPOCH},
};

use crate::error::{Context, ErrorExt};

/// A key pair (`PublicKey` and `SecretKey`).
#[derive(Clone, Debug)]
pub struct KeyPair {
  pub pk: String,
  pub sk: String,
}

fn create_file(path: &Path) -> crate::Result<BufWriter<File>> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).fs_context("failed to create directory", parent.to_path_buf())?;
  }
  let file = File::create(path).fs_context("failed to create file", path.to_path_buf())?;
  Ok(BufWriter::new(file))
}

/// Generate base64 encoded keypair
pub fn generate_key(password: Option<String>) -> crate::Result<KeyPair> {
  // Normalize empty password to None. minisign's generate_encrypted_keypair
  // sets KDF parameters even for empty passwords but skips actual encryption,
  // creating an inconsistent state where into_secret_key later fails with
  // "Wrong password for that key".
  let password = password.filter(|p| !p.is_empty());
  let KP { pk, sk } = match password {
    Some(_) => KP::generate_encrypted_keypair(password).unwrap(),
    None => KP::generate_unencrypted_keypair().unwrap(),
  };

  let pk_box_str = pk.to_box().unwrap().to_string();
  let sk_box_str = sk.to_box(None).unwrap().to_string();

  let encoded_pk = base64::engine::general_purpose::STANDARD.encode(pk_box_str);
  let encoded_sk = base64::engine::general_purpose::STANDARD.encode(sk_box_str);

  Ok(KeyPair {
    pk: encoded_pk,
    sk: encoded_sk,
  })
}

/// Transform a base64 String to readable string for the main signer
pub fn decode_key<S: AsRef<[u8]>>(base64_key: S) -> crate::Result<String> {
  let decoded_str = &base64::engine::general_purpose::STANDARD
    .decode(base64_key)
    .context("failed to decode base64 key")?[..];
  Ok(String::from(
    str::from_utf8(decoded_str).context("failed to convert base64 to utf8")?,
  ))
}

/// Save KeyPair to disk
pub fn save_keypair<P>(
  force: bool,
  sk_path: P,
  key: &str,
  pubkey: &str,
) -> crate::Result<(PathBuf, PathBuf)>
where
  P: AsRef<Path>,
{
  let sk_path = sk_path.as_ref();

  let pubkey_path = format!("{}.pub", sk_path.display());
  let pk_path = Path::new(&pubkey_path);

  if sk_path.exists() {
    if !force {
      crate::error::bail!(
        "Key generation aborted:\n{} already exists\nIf you really want to overwrite the existing key pair, add the --force switch to force this operation.",
        sk_path.display()
      );
    } else {
      std::fs::remove_file(sk_path)
        .fs_context("failed to remove secret key file", sk_path.to_path_buf())?;
    }
  }

  if pk_path.exists() {
    std::fs::remove_file(pk_path)
      .fs_context("failed to remove public key file", pk_path.to_path_buf())?;
  }

  let write_file = |mut writer: BufWriter<File>, contents: &str| -> std::io::Result<()> {
    write!(writer, "{contents:}")?;
    writer.flush()?;
    Ok(())
  };

  write_file(create_file(sk_path)?, key)
    .fs_context("failed to write secret key", sk_path.to_path_buf())?;

  write_file(create_file(pk_path)?, pubkey)
    .fs_context("failed to write public key", pk_path.to_path_buf())?;

  Ok((
    fs::canonicalize(sk_path).fs_context(
      "failed to canonicalize secret key path",
      sk_path.to_path_buf(),
    )?,
    fs::canonicalize(pk_path).fs_context(
      "failed to canonicalize public key path",
      pk_path.to_path_buf(),
    )?,
  ))
}

/// Sign files
pub fn sign_file<P>(secret_key: &SecretKey, bin_path: P) -> crate::Result<(PathBuf, SignatureBox)>
where
  P: AsRef<Path>,
{
  let bin_path = bin_path.as_ref();
  // We need to append .sig at the end it's where the signature will be stored
  // TODO: use with_added_extension when we bump MSRV to > 1.91'
  let signature_path = if let Some(ext) = bin_path.extension() {
    let mut extension = ext.to_os_string();
    extension.push(".sig");
    bin_path.with_extension(extension)
  } else {
    bin_path.with_extension("sig")
  };

  let trusted_comment = format!(
    "timestamp:{}\tfile:{}",
    unix_timestamp(),
    bin_path.file_name().unwrap().to_string_lossy()
  );

  let data_reader = open_data_file(bin_path)?;

  let signature_box = sign(
    None,
    secret_key,
    data_reader,
    Some(trusted_comment.as_str()),
    Some("signature from tauri secret key"),
  )
  .context("failed to sign file")?;

  let encoded_signature =
    base64::engine::general_purpose::STANDARD.encode(signature_box.to_string());
  std::fs::write(&signature_path, encoded_signature.as_bytes())
    .fs_context("failed to write signature file", signature_path.clone())?;
  Ok((
    fs::canonicalize(&signature_path)
      .fs_context("failed to canonicalize signature file", &signature_path)?,
    signature_box,
  ))
}

/// Gets the updater secret key from the given private key and password.
pub fn secret_key<S: AsRef<[u8]>>(
  private_key: S,
  password: Option<String>,
) -> crate::Result<SecretKey> {
  // Normalize empty password to None. minisign's generate_encrypted_keypair
  // with Some("") sets KDF parameters but skips encryption, creating an
  // inconsistent key state. into_secret_key(Some("")) then tries scrypt
  // decryption on unencrypted data, corrupting it and failing with
  // "Wrong password for that key". Using into_unencrypted_secret_key
  // avoids this by skipping decryption entirely.
  let password = password.filter(|p| !p.is_empty());
  let decoded_secret = decode_key(private_key).context("failed to decode base64 secret key")?;
  let sk_box =
    SecretKeyBox::from_string(&decoded_secret).context("failed to load updater private key")?;
  let sk = match password {
    Some(_) => sk_box
      .into_secret_key(password)
      .context("incorrect updater private key password")?,
    None => sk_box
      .into_unencrypted_secret_key()
      .context("incorrect updater private key password")?,
  };
  Ok(sk)
}

/// Gets the updater secret key from the given private key and password.
pub fn pub_key<S: AsRef<[u8]>>(public_key: S) -> crate::Result<PublicKey> {
  let decoded_publick = decode_key(public_key).context("failed to decode base64 pubkey")?;
  let pk_box =
    PublicKeyBox::from_string(&decoded_publick).context("failed to load updater pubkey")?;
  let pk = pk_box
    .into_public_key()
    .context("failed to convert updater pubkey")?;
  Ok(pk)
}

fn unix_timestamp() -> u64 {
  let start = SystemTime::now();
  let since_the_epoch = start
    .duration_since(UNIX_EPOCH)
    .expect("system clock is incorrect");
  since_the_epoch.as_secs()
}

fn open_data_file<P>(data_path: P) -> crate::Result<BufReader<File>>
where
  P: AsRef<Path>,
{
  let data_path = data_path.as_ref();
  let file = OpenOptions::new()
    .read(true)
    .open(data_path)
    .fs_context("failed to open data file", data_path.to_path_buf())?;
  Ok(BufReader::new(file))
}

#[cfg(test)]
mod tests {
  #[test]
  fn generate_and_sign_with_empty_password() {
    let path = std::env::temp_dir().join("minisign-empty-pw-roundtrip.txt");
    std::fs::write(&path, b"TAURI ROUNDTRIP").expect("failed to write test file");

    // Generate a key with an empty password (same as `tauri signer generate -p ""`)
    let keypair = super::generate_key(Some("".into())).expect("failed to generate key");

    // Sign with an empty password (same as `tauri signer sign -p ""`)
    let secret_key =
      super::secret_key(&keypair.sk, Some("".into())).expect("failed to resolve secret key");
    super::sign_file(&secret_key, &path).expect("failed to sign file");
  }

  #[test]
  fn generate_and_sign_with_no_password() {
    let path = std::env::temp_dir().join("minisign-none-pw-roundtrip.txt");
    std::fs::write(&path, b"TAURI ROUNDTRIP NONE").expect("failed to write test file");

    // Generate a key with no password
    let keypair = super::generate_key(None).expect("failed to generate key");

    // Sign with no password
    let secret_key =
      super::secret_key(&keypair.sk, None).expect("failed to resolve secret key");
    super::sign_file(&secret_key, &path).expect("failed to sign file");
  }

  #[test]
  fn generate_with_password_and_sign() {
    let path = std::env::temp_dir().join("minisign-with-pw-roundtrip.txt");
    std::fs::write(&path, b"TAURI ROUNDTRIP PW").expect("failed to write test file");

    // Generate with a real password
    let keypair =
      super::generate_key(Some("test-password".into())).expect("failed to generate key");

    // Sign with the same password
    let secret_key = super::secret_key(&keypair.sk, Some("test-password".into()))
      .expect("failed to resolve secret key");
    super::sign_file(&secret_key, &path).expect("failed to sign file");
  }

  #[test]
  fn empty_password_sign_then_none_sign_are_interchangeable() {
    let path = std::env::temp_dir().join("minisign-empty-none-interop.txt");
    std::fs::write(&path, b"TAURI INTEROP").expect("failed to write test file");

    // Generate with empty password
    let keypair = super::generate_key(Some("".into())).expect("failed to generate key");

    // Sign with None (should work the same as empty password)
    let secret_key =
      super::secret_key(&keypair.sk, None).expect("failed to resolve secret key");
    super::sign_file(&secret_key, &path).expect("failed to sign file");
  }
}
