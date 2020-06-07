use attohttpc;
use serde::Serialize;

use std::io::{BufWriter, Write};

pub(crate) mod link_value;

pub fn get(url: String) -> crate::Result<attohttpc::Response> {
  let response = attohttpc::get(url).send()?;

  Ok(response)
}

pub fn post_as_json<T: Serialize>(url: String, payload: &T) -> crate::Result<attohttpc::Response> {
  let response = attohttpc::post(url).json(payload)?.send()?;

  Ok(response)
}

pub fn download<T: Write>(url: String, dest: T, _display_progress: bool) -> crate::Result<()> {
  set_ssl_vars!();

  let resp = get(url)?;

  if !resp.status().is_success() {
    return Err(crate::Error::Download(resp.status()).into());
  }

  let file = BufWriter::new(dest);
  resp.write_to(file)?;
  Ok(())
}
