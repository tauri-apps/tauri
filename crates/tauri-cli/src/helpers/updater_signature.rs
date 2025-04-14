// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use anyhow::Context;
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

/// A key pair (`PublicKey` and `SecretKey`).
#[derive(Clone, Debug)]
pub struct KeyPair {
  pub pk: String,
  pub sk: String,
}

fn create_file(path: &Path) -> crate::Result<BufWriter<File>> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  let file = File::create(path)?;
  Ok(BufWriter::new(file))
}

/// Generate base64 encoded keypair
pub fn generate_key(password: Option<String>) -> crate::Result<KeyPair> {
  let KP { pk, sk } = KP::generate_encrypted_keypair(password).unwrap();

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
  let decoded_str = &base64::engine::general_purpose::STANDARD.decode(base64_key)?[..];
  Ok(String::from(str::from_utf8(decoded_str)?))
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
      return Err(anyhow::anyhow!(
        "Key generation aborted:\n{} already exists\nIf you really want to overwrite the existing key pair, add the --force switch to force this operation.",
        sk_path.display()
      ));
    } else {
      std::fs::remove_file(sk_path)?;
    }
  }

  if pk_path.exists() {
    std::fs::remove_file(pk_path)?;
  }

  let mut sk_writer = create_file(sk_path)?;
  write!(sk_writer, "{key:}")?;
  sk_writer.flush()?;

  let mut pk_writer = create_file(pk_path)?;
  write!(pk_writer, "{pubkey:}")?;
  pk_writer.flush()?;

  Ok((fs::canonicalize(sk_path)?, fs::canonicalize(pk_path)?))
}

/// Sign files
pub fn sign_file<P>(secret_key: &SecretKey, bin_path: P) -> crate::Result<(PathBuf, SignatureBox)>
where
  P: AsRef<Path>,
{
  let bin_path = bin_path.as_ref();
  // We need to append .sig at the end it's where the signature will be stored
  let mut extension = bin_path.extension().unwrap().to_os_string();
  extension.push(".sig");
  let signature_path = bin_path.with_extension(extension);

  let mut signature_box_writer = create_file(&signature_path)?;

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
  )?;

  let encoded_signature =
    base64::engine::general_purpose::STANDARD.encode(signature_box.to_string());
  signature_box_writer.write_all(encoded_signature.as_bytes())?;
  signature_box_writer.flush()?;
  Ok((fs::canonicalize(&signature_path)?, signature_box))
}

/// Gets the updater secret key from the given private key and password.
pub fn secret_key<S: AsRef<[u8]>>(
  private_key: S,
  password: Option<String>,
) -> crate::Result<SecretKey> {
  let decoded_secret = decode_key(private_key).context("failed to decode base64 secret key")?;
  let sk_box =
    SecretKeyBox::from_string(&decoded_secret).context("failed to load updater private key")?;
  let sk = sk_box
    .into_secret_key(password)
    .context("incorrect updater private key password")?;
  Ok(sk)
}

/// Gets the updater secret key from the given private key and password.
pub fn pub_key<S: AsRef<[u8]>>(public_key: S) -> crate::Result<PublicKey> {
  let decoded_publick = decode_key(public_key).context("failed to decode base64 pubkey")?;
  let pk_box =
    PublicKeyBox::from_string(&decoded_publick).context("failed to load updater pubkey")?;
  let pk = pk_box.into_public_key()?;
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
    .map_err(|e| minisign::PError::new(minisign::ErrorKind::Io, e))?;
  Ok(BufReader::new(file))
}

#[cfg(test)]
mod tests {
  const PRIVATE_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5dkpDN09RZm5GeVAzc2RuYlNzWVVJelJRQnNIV2JUcGVXZUplWXZXYXpqUUFBQkFBQUFBQUFBQUFBQUlBQUFBQTZrN2RnWGh5dURxSzZiL1ZQSDdNcktiaHRxczQwMXdQelRHbjRNcGVlY1BLMTBxR2dpa3I3dDE1UTVDRDE4MXR4WlQwa1BQaXdxKy9UU2J2QmVSNXhOQWFDeG1GSVllbUNpTGJQRkhhTnROR3I5RmdUZi90OGtvaGhJS1ZTcjdZU0NyYzhQWlQ5cGM9Cg==";

  // we use minisign=0.7.3 to prevent a breaking change
  #[test]
  fn empty_password_is_valid() {
    let path = std::env::temp_dir().join("minisign-password-text.txt");
    std::fs::write(&path, b"TAURI").expect("failed to write test file");

    let secret_key =
      super::secret_key(PRIVATE_KEY, Some("".into())).expect("failed to resolve secret key");
    super::sign_file(&secret_key, &path).expect("failed to sign file");
  }
}
