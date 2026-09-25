// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use base64::Engine;
use minisign::{
  KeyPair as KP, PublicKey, PublicKeyBox, SecretKey, SecretKeyBox, SignatureBox, sign,
};
use std::{
  fs::{self, File, OpenOptions},
  io::{BufReader, IsTerminal, Write},
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

/// Writes the secret key to `path`, making sure it is only readable by the current user on Unix.
fn write_secret_key(path: &Path, contents: &str) -> std::io::Result<()> {
  let mut options = OpenOptions::new();
  options.write(true).create(true).truncate(true);
  #[cfg(unix)]
  {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o600);
  }
  let mut file = options.open(path)?;
  // the mode above only applies to newly created files, so also restrict an existing one
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
  }
  file.write_all(contents.as_bytes())?;
  file.flush()
}

/// Generate base64 encoded keypair
pub fn generate_key(password: Option<String>) -> crate::Result<KeyPair> {
  let KP { pk, sk } =
    KP::generate_encrypted_keypair(password).context("failed to generate key pair")?;

  let pk_box_str = pk
    .to_box()
    .context("failed to encode public key")?
    .to_string();
  let sk_box_str = sk
    .to_box(None)
    .context("failed to encode secret key")?
    .to_string();

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

  if !force {
    for path in [sk_path, pk_path] {
      if path.exists() {
        crate::error::bail!(
          "Key generation aborted:\n{} already exists\nIf you really want to overwrite the existing key pair, add the --force switch to force this operation.",
          path.display()
        );
      }
    }
  }

  if let Some(parent) = sk_path.parent() {
    fs::create_dir_all(parent).fs_context("failed to create directory", parent.to_path_buf())?;
  }

  write_secret_key(sk_path, key).fs_context("failed to write secret key", sk_path.to_path_buf())?;
  fs::write(pk_path, pubkey).fs_context("failed to write public key", pk_path.to_path_buf())?;

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
///
/// When `version` is given it is embedded in the signature's trusted comment. minisign covers
/// the trusted comment with its global signature, so this binds the signed artifact to the
/// version it was released as. The update manifest is not itself signed, so without this the
/// manifest's `version` field can be paired with an older release's URL and signature to force
/// a downgrade. See the `requireSignedVersion` updater config option.
pub fn sign_file<P>(
  secret_key: &SecretKey,
  bin_path: P,
  version: Option<&str>,
) -> crate::Result<(PathBuf, SignatureBox)>
where
  P: AsRef<Path>,
{
  let bin_path = bin_path.as_ref();
  // We need to append .sig at the end it's where the signature will be stored
  // TODO: use `with_added_extension` when we bump MSRV to >= 1.91
  let signature_path = if let Some(ext) = bin_path.extension() {
    let mut extension = ext.to_os_string();
    extension.push(".sig");
    bin_path.with_extension(extension)
  } else {
    bin_path.with_extension("sig")
  };

  let file_name = bin_path
    .file_name()
    .with_context(|| format!("{} is not a file path", bin_path.display()))?
    .to_string_lossy();
  // the trusted comment is a single line of tab separated fields, so a value carrying
  // either separator would produce a signature we cannot parse back
  if file_name.contains(['\t', '\r', '\n']) {
    crate::error::bail!(
      "the file {file_name:?} cannot be signed because its name contains a tab or newline"
    );
  }
  let mut trusted_comment = format!("timestamp:{}\tfile:{file_name}", unix_timestamp());
  if let Some(version) = version {
    if version.contains(['\t', '\r', '\n']) {
      crate::error::bail!(
        "the app version {version:?} cannot be signed because it contains a tab or newline"
      );
    }
    // appended last so anything parsing the historical `timestamp:...\tfile:...` prefix keeps working
    trusted_comment.push_str("\tversion:");
    trusted_comment.push_str(version);
  }

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
///
/// If `password` is `None`, a password is going to be prompted interactively,
/// or an empty password is assumed when stdin is not a terminal.
pub fn secret_key<S: AsRef<[u8]>>(
  private_key: S,
  mut password: Option<String>,
) -> crate::Result<SecretKey> {
  let decoded_secret = decode_key(private_key).context("failed to decode base64 secret key")?;
  let sk_box =
    SecretKeyBox::from_string(&decoded_secret).context("failed to load updater private key")?;
  if password.is_none() {
    if std::io::stdin().is_terminal() {
      log::info!("Decrypting updater private key, expect a prompt for password");
    } else {
      // the password prompt needs a terminal, so assume the key has no password
      log::info!("No updater private key password provided, assuming an empty password");
      password.replace(String::new());
    }
  }
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
  use super::*;

  // This was encrypted with an empty string
  const PRIVATE_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5dkpDN09RZm5GeVAzc2RuYlNzWVVJelJRQnNIV2JUcGVXZUplWXZXYXpqUUFBQkFBQUFBQUFBQUFBQUlBQUFBQTZrN2RnWGh5dURxSzZiL1ZQSDdNcktiaHRxczQwMXdQelRHbjRNcGVlY1BLMTBxR2dpa3I3dDE1UTVDRDE4MXR4WlQwa1BQaXdxKy9UU2J2QmVSNXhOQWFDeG1GSVllbUNpTGJQRkhhTnROR3I5RmdUZi90OGtvaGhJS1ZTcjdZU0NyYzhQWlQ5cGM9Cg==";

  // minisign >=0.7.4,<0.8.0 couldn't handle empty passwords if the private key is encrypted with an empty string.
  #[test]
  fn empty_password_is_valid() {
    let path = std::env::temp_dir().join("minisign-password-text.txt");
    std::fs::write(&path, b"TAURI").expect("failed to write test file");

    let secret_key =
      secret_key(PRIVATE_KEY, Some("".into())).expect("failed to resolve secret key");
    sign_file(&secret_key, &path, None).expect("failed to sign file");
  }

  #[test]
  fn embeds_version_in_trusted_comment() {
    let path = std::env::temp_dir().join("minisign-versioned-text.txt");
    std::fs::write(&path, b"TAURI").expect("failed to write test file");

    let secret_key =
      secret_key(PRIVATE_KEY, Some("".into())).expect("failed to resolve secret key");

    let (_, signature) = sign_file(&secret_key, &path, Some("1.2.3")).expect("failed to sign file");
    let trusted_comment = signature
      .trusted_comment()
      .expect("failed to read trusted comment");
    assert!(
      trusted_comment.ends_with("\tversion:1.2.3"),
      "unexpected trusted comment: {trusted_comment}"
    );
    // the historical prefix must stay intact so older consumers keep parsing it
    assert!(trusted_comment.starts_with("timestamp:"));
    assert!(trusted_comment.contains("\tfile:minisign-versioned-text.txt\t"));

    let (_, signature) = sign_file(&secret_key, &path, None).expect("failed to sign file");
    assert!(
      !signature
        .trusted_comment()
        .expect("failed to read trusted comment")
        .contains("version:")
    );
  }

  #[test]
  fn rejects_version_that_breaks_the_trusted_comment() {
    let path = std::env::temp_dir().join("minisign-invalid-version-text.txt");
    std::fs::write(&path, b"TAURI").expect("failed to write test file");

    let secret_key =
      secret_key(PRIVATE_KEY, Some("".into())).expect("failed to resolve secret key");
    assert!(sign_file(&secret_key, &path, Some("1.0.0\ttampered")).is_err());
    assert!(sign_file(&secret_key, &path, Some("1.0.0\ntampered")).is_err());
  }

  #[test]
  fn rejects_file_name_that_breaks_the_trusted_comment() {
    let dir = tempfile::tempdir().unwrap();
    // the name is validated before the file is opened, so it does not need to exist
    // (Windows does not allow tabs in file names at all)
    let path = dir.path().join("app\tfile:evil.txt");

    let secret_key =
      secret_key(PRIVATE_KEY, Some("".into())).expect("failed to resolve secret key");
    let Err(error) = sign_file(&secret_key, &path, None) else {
      panic!("expected signing to fail");
    };
    assert!(error.to_string().contains("tab or newline"), "{error}");
  }

  #[test]
  fn save_keypair_refuses_to_overwrite_without_force() {
    let dir = tempfile::tempdir().unwrap();
    let sk_path = dir.path().join("key");
    let pk_path = dir.path().join("key.pub");

    // only the public key exists: it must be kept
    std::fs::write(&pk_path, "old public").unwrap();
    assert!(save_keypair(false, &sk_path, "secret", "public").is_err());
    assert_eq!(std::fs::read_to_string(&pk_path).unwrap(), "old public");
    assert!(!sk_path.exists());

    save_keypair(true, &sk_path, "secret", "public").unwrap();
    assert_eq!(std::fs::read_to_string(&sk_path).unwrap(), "secret");
    assert_eq!(std::fs::read_to_string(&pk_path).unwrap(), "public");

    assert!(save_keypair(false, &sk_path, "secret2", "public2").is_err());
    assert_eq!(std::fs::read_to_string(&sk_path).unwrap(), "secret");

    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
      assert_eq!(mode(&sk_path), 0o600);
      assert_eq!(mode(&pk_path), 0o644);
    }
  }

  // This tests the newly generated keys with empty string password works
  // minisign >=0.7.4,<=0.8.0 generate keys unencrypted if the password is empty but is marked encrypted hence unusable
  #[test]
  fn generate_empty_password_keys_and_use() {
    let KeyPair { pk, sk } = generate_key(Some("".to_owned())).unwrap();
    let pk = pub_key(pk).unwrap();
    let sk = secret_key(sk, Some("".into())).unwrap();
    let data = b"TAURI".as_slice();
    sign(Some(&pk), &sk, data, None, None).expect("failed to sign file");
  }
}
