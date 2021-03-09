extern crate minisign;

use base64::{decode, encode};
use minisign::{sign, KeyPair as KP, SecretKeyBox};
use std::fs::{self};
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

/// A key pair (`PublicKey` and `SecretKey`).
#[derive(Clone, Debug)]
pub struct KeyPair {
  pub pk: String,
  pub sk: String,
}

/// Generate base64 encoded keypair
pub fn generate_key(password: Option<String>) -> crate::Result<KeyPair> {
  let KP { pk, sk } = KP::generate_encrypted_keypair(password).unwrap();

  let pk_box_str = pk.to_box().unwrap().to_string();
  let sk_box_str = sk.to_box(None).unwrap().to_string();

  let encoded_pk = encode(&pk_box_str);
  let encoded_sk = encode(&sk_box_str);

  Ok(KeyPair {
    pk: encoded_pk,
    sk: encoded_sk,
  })
}

/// Transform a base64 String to readable string for the main signer
pub fn decode_key(base64_key: String) -> crate::Result<String> {
  let decoded_str = &decode(&base64_key)?[..];
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
      return Err(crate::Error::GenericError(format!(
        "Key generation aborted:\n{} already exists\nIf you really want to overwrite the existing key pair, add the --force switch to force this operation.",
        sk_path.display()
      )));
    } else {
      std::fs::remove_file(&sk_path)?;
    }
  }

  if pk_path.exists() {
    std::fs::remove_file(&pk_path)?;
  }

  let mut sk_writer = crate::bundle::common::create_file(&sk_path)?;
  write!(sk_writer, "{:}", key)?;
  sk_writer.flush()?;

  let mut pk_writer = crate::bundle::common::create_file(&pk_path)?;
  write!(pk_writer, "{:}", pubkey)?;
  pk_writer.flush()?;

  Ok((fs::canonicalize(&sk_path)?, fs::canonicalize(&pk_path)?))
}

/// Read key from file
pub fn read_key_from_file<P>(sk_path: P) -> crate::Result<String>
where
  P: AsRef<Path>,
{
  Ok(fs::read_to_string(sk_path)?)
}

/// Sign files
pub fn sign_file<P>(
  private_key: String,
  password: String,
  bin_path: P,
  prehashed: bool,
) -> crate::Result<(PathBuf, String)>
where
  P: AsRef<Path>,
{
  let decoded_secret = decode_key(private_key)?;
  let sk_box = SecretKeyBox::from_string(&decoded_secret).unwrap();
  let sk = sk_box.into_secret_key(Some(password)).unwrap();

  // We need to append .sig at the end it's where the signature will be stored
  let signature_path_string = format!("{}.sig", bin_path.as_ref().display());
  let signature_path = Path::new(&signature_path_string);

  let mut signature_box_writer = super::bundle::common::create_file(&signature_path)?;

  let trusted_comment = format!(
    "timestamp:{}\tfile:{}",
    unix_timestamp(),
    bin_path.as_ref().display()
  );

  let (data_reader, should_be_prehashed) = open_data_file(bin_path)?;

  let signature_box = sign(
    None,
    &sk,
    data_reader,
    prehashed | should_be_prehashed,
    Some(trusted_comment.as_str()),
    Some("signature from tauri secret key"),
  )?;

  let encoded_signature = encode(&signature_box.to_string());
  signature_box_writer.write_all(&encoded_signature.as_bytes())?;
  signature_box_writer.flush()?;
  Ok((fs::canonicalize(&signature_path)?, encoded_signature))
}

fn unix_timestamp() -> u64 {
  let start = SystemTime::now();
  let since_the_epoch = start
    .duration_since(UNIX_EPOCH)
    .expect("system clock is incorrect");
  since_the_epoch.as_secs()
}

fn open_data_file<P>(data_path: P) -> crate::Result<(BufReader<File>, bool)>
where
  P: AsRef<Path>,
{
  let data_path = data_path.as_ref();
  let file = OpenOptions::new()
    .read(true)
    .open(data_path)
    .map_err(|e| minisign::PError::new(minisign::ErrorKind::Io, e))?;
  let should_be_hashed = match file.metadata() {
    Ok(metadata) => metadata.len() > (1u64 << 30),
    Err(_) => true,
  };
  Ok((BufReader::new(file), should_be_hashed))
}
