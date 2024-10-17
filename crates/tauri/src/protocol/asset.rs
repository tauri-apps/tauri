// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{path::SafePathBuf, scope, webview::UriSchemeProtocolHandler};
use http::{header::*, status::StatusCode, Request, Response};
use http_range::HttpRange;
use std::{borrow::Cow, io::SeekFrom};
use tauri_utils::mime_type::MimeType;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

pub fn get(scope: scope::fs::Scope, window_origin: String) -> UriSchemeProtocolHandler {
  Box::new(
    move |_, request, responder| match get_response(request, &scope, &window_origin) {
      Ok(response) => responder.respond(response),
      Err(e) => responder.respond(
        http::Response::builder()
          .status(http::StatusCode::INTERNAL_SERVER_ERROR)
          .header(CONTENT_TYPE, mime::TEXT_PLAIN.essence_str())
          .header("Access-Control-Allow-Origin", &window_origin)
          .body(e.to_string().as_bytes().to_vec())
          .unwrap(),
      ),
    },
  )
}

fn get_response(
  request: Request<Vec<u8>>,
  scope: &scope::fs::Scope,
  window_origin: &str,
) -> Result<Response<Cow<'static, [u8]>>, Box<dyn std::error::Error>> {
  // skip leading `/`
  let path = percent_encoding::percent_decode(request.uri().path()[1..].as_bytes())
    .decode_utf8_lossy()
    .to_string();

  let mut resp = Response::builder().header("Access-Control-Allow-Origin", window_origin);

  if let Err(e) = SafePathBuf::new(path.clone().into()) {
    log::error!("asset protocol path \"{}\" is not valid: {}", path, e);
    return resp.status(403).body(Vec::new().into()).map_err(Into::into);
  }

  if !scope.is_allowed(&path) {
    log::error!("asset protocol not configured to allow the path: {}", path);
    return resp.status(403).body(Vec::new().into()).map_err(Into::into);
  }

  let (mut file, len, mime_type, read_bytes) = crate::async_runtime::safe_block_on(async move {
    let mut file = File::open(&path).await?;

    // get file length
    let len = {
      let old_pos = file.stream_position().await?;
      let len = file.seek(SeekFrom::End(0)).await?;
      file.seek(SeekFrom::Start(old_pos)).await?;
      len
    };

    // get file mime type
    let (mime_type, read_bytes) = {
      let nbytes = len.min(8192);
      let mut magic_buf = Vec::with_capacity(nbytes as usize);
      let old_pos = file.stream_position().await?;
      (&mut file).take(nbytes).read_to_end(&mut magic_buf).await?;
      file.seek(SeekFrom::Start(old_pos)).await?;
      (
        MimeType::parse(&magic_buf, &path),
        // return the `magic_bytes` if we read the whole file
        // to avoid reading it again later if this is not a range request
        if len < 8192 { Some(magic_buf) } else { None },
      )
    };

    Ok::<(File, u64, String, Option<Vec<u8>>), anyhow::Error>((file, len, mime_type, read_bytes))
  })?;

  resp = resp.header(CONTENT_TYPE, &mime_type);

  // handle 206 (partial range) http requests
  let response = if let Some(range_header) = request
    .headers()
    .get("range")
    .and_then(|r| r.to_str().map(|r| r.to_string()).ok())
  {
    resp = resp.header(ACCEPT_RANGES, "bytes");
    resp = resp.header(ACCESS_CONTROL_EXPOSE_HEADERS, "content-range");

    let not_satisfiable = || {
      Response::builder()
        .status(StatusCode::RANGE_NOT_SATISFIABLE)
        .header(CONTENT_RANGE, format!("bytes */{len}"))
        .body(vec![].into())
        .map_err(Into::into)
    };

    // parse range header
    let ranges = if let Ok(ranges) = HttpRange::parse(&range_header, len) {
      ranges
        .iter()
        // map the output to spec range <start-end>, example: 0-499
        .map(|r| (r.start, r.start + r.length - 1))
        .collect::<Vec<_>>()
    } else {
      return not_satisfiable();
    };

    /// The Maximum bytes we send in one range
    const MAX_LEN: u64 = 1000 * 1024;

    // single-part range header
    if ranges.len() == 1 {
      let &(start, mut end) = ranges.first().unwrap();

      // check if a range is not satisfiable
      //
      // this should be already taken care of by the range parsing library
      // but checking here again for extra assurance
      if start >= len || end >= len || end < start {
        return not_satisfiable();
      }

      // adjust end byte for MAX_LEN
      end = start + (end - start).min(len - start).min(MAX_LEN - 1);

      // calculate number of bytes needed to be read
      let nbytes = end + 1 - start;

      let buf = crate::async_runtime::safe_block_on(async move {
        let mut buf = Vec::with_capacity(nbytes as usize);
        file.seek(SeekFrom::Start(start)).await?;
        file.take(nbytes).read_to_end(&mut buf).await?;
        Ok::<Vec<u8>, anyhow::Error>(buf)
      })?;

      resp = resp.header(CONTENT_RANGE, format!("bytes {start}-{end}/{len}"));
      resp = resp.header(CONTENT_LENGTH, end + 1 - start);
      resp = resp.status(StatusCode::PARTIAL_CONTENT);
      resp.body(buf.into())
    } else {
      let ranges = ranges
        .iter()
        .filter_map(|&(start, mut end)| {
          // filter out unsatisfiable ranges
          //
          // this should be already taken care of by the range parsing library
          // but checking here again for extra assurance
          if start >= len || end >= len || end < start {
            None
          } else {
            // adjust end byte for MAX_LEN
            end = start + (end - start).min(len - start).min(MAX_LEN - 1);
            Some((start, end))
          }
        })
        .collect::<Vec<_>>();

      let boundary = random_boundary();
      let boundary_sep = format!("\r\n--{boundary}\r\n");
      let boundary_closer = format!("\r\n--{boundary}\r\n");

      resp = resp.header(
        CONTENT_TYPE,
        format!("multipart/byteranges; boundary={boundary}"),
      );

      let buf = crate::async_runtime::safe_block_on(async move {
        // multi-part range header
        let mut buf = Vec::new();

        for (end, start) in ranges {
          // a new range is being written, write the range boundary
          buf.write_all(boundary_sep.as_bytes()).await?;

          // write the needed headers `Content-Type` and `Content-Range`
          buf
            .write_all(format!("{CONTENT_TYPE}: {mime_type}\r\n").as_bytes())
            .await?;
          buf
            .write_all(format!("{CONTENT_RANGE}: bytes {start}-{end}/{len}\r\n").as_bytes())
            .await?;

          // write the separator to indicate the start of the range body
          buf.write_all("\r\n".as_bytes()).await?;

          // calculate number of bytes needed to be read
          let nbytes = end + 1 - start;

          let mut local_buf = Vec::with_capacity(nbytes as usize);
          file.seek(SeekFrom::Start(start)).await?;
          (&mut file).take(nbytes).read_to_end(&mut local_buf).await?;
          buf.extend_from_slice(&local_buf);
        }
        // all ranges have been written, write the closing boundary
        buf.write_all(boundary_closer.as_bytes()).await?;

        Ok::<Vec<u8>, anyhow::Error>(buf)
      })?;
      resp.body(buf.into())
    }
  } else if request.method() == http::Method::HEAD {
    // if the HEAD method is used, we should not return a body
    resp = resp.header(CONTENT_LENGTH, len);
    resp.body(Vec::new().into())
  } else {
    // avoid reading the file if we already read it
    // as part of mime type detection
    let buf = if let Some(b) = read_bytes {
      b
    } else {
      crate::async_runtime::safe_block_on(async move {
        let mut local_buf = Vec::with_capacity(len as usize);
        file.read_to_end(&mut local_buf).await?;
        Ok::<Vec<u8>, anyhow::Error>(local_buf)
      })?
    };
    resp = resp.header(CONTENT_LENGTH, len);
    resp.body(buf.into())
  };

  response.map_err(Into::into)
}

fn random_boundary() -> String {
  let mut x = [0_u8; 30];
  getrandom::getrandom(&mut x).expect("failed to get random bytes");
  (x[..])
    .iter()
    .map(|&x| format!("{x:x}"))
    .fold(String::new(), |mut a, x| {
      a.push_str(x.as_str());
      a
    })
}
