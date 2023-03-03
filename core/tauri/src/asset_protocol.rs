// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(protocol_asset)]

use crate::api::file::SafePathBuf;
use crate::scope::FsScope;
use rand::RngCore;
use std::io::SeekFrom;
use std::pin::Pin;
use tauri_runtime::http::HttpRange;
use tauri_runtime::http::{
  header::*, status::StatusCode, MimeType, Request, Response, ResponseBuilder,
};
use tauri_utils::debug_eprintln;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use url::Position;
use url::Url;

pub fn asset_protocol_handler(
  request: &Request,
  scope: FsScope,
  window_origin: String,
) -> Result<Response, Box<dyn std::error::Error>> {
  let parsed_path = Url::parse(request.uri())?;
  let filtered_path = &parsed_path[..Position::AfterPath];
  let path = filtered_path
    .strip_prefix("asset://localhost/")
    // the `strip_prefix` only returns None when a request is made to `https://tauri.$P` on Windows
    // where `$P` is not `localhost/*`
    .unwrap_or("");
  let path = percent_encoding::percent_decode(path.as_bytes())
    .decode_utf8_lossy()
    .to_string();

  if let Err(e) = SafePathBuf::new(path.clone().into()) {
    debug_eprintln!("asset protocol path \"{}\" is not valid: {}", path, e);
    return ResponseBuilder::new().status(403).body(Vec::new());
  }

  if !scope.is_allowed(&path) {
    debug_eprintln!("asset protocol not configured to allow the path: {}", path);
    return ResponseBuilder::new().status(403).body(Vec::new());
  }

  let mut resp = ResponseBuilder::new().header("Access-Control-Allow-Origin", &window_origin);

  crate::async_runtime::block_on(async move {
    let mut file = File::open(&path).await?;
    // get file length
    let len = {
      let old_pos = file.stream_position().await?;
      let len = file.seek(SeekFrom::End(0)).await?;
      file.seek(SeekFrom::Start(old_pos)).await?;
      len
    };
    // get file mime type
    let mime_type = {
      let mut magic_bytes = [0; 8192];
      let old_pos = file.stream_position().await?;
      file.read_exact(&mut magic_bytes).await?;
      file.seek(SeekFrom::Start(old_pos)).await?;
      MimeType::parse(&magic_bytes, &path)
    };

    resp = resp.header(CONTENT_TYPE, &mime_type);

    // handle 206 (partial range) http requests
    let response = if let Some(range_header) = request
      .headers()
      .get("range")
      .and_then(|r| r.to_str().map(|r| r.to_string()).ok())
    {
      resp = resp.header(ACCEPT_RANGES, "bytes");

      let not_satisfiable = || {
        ResponseBuilder::new()
          .status(StatusCode::RANGE_NOT_SATISFIABLE)
          .header(CONTENT_RANGE, format!("bytes */{len}"))
          .body(vec![])
      };

      // parse range header
      let ranges = if let Ok(ranges) = HttpRange::parse(&range_header, len) {
        ranges
          .iter()
          // map the output back to spec range <start-end>, example: 0-499
          .map(|r| (r.start, r.start + r.length - 1))
          .collect::<Vec<_>>()
      } else {
        return not_satisfiable();
      };

      /// only send 1MB or less at a time
      const MAX_LEN: u64 = 1000 * 1024;

      if ranges.len() == 1 {
        let &(start, mut end) = ranges.first().unwrap();

        // check if a range is not satisfiable
        //
        // this should be already taken care of by the range parsing library
        // but checking here again for extra assurance
        if start >= len || end >= len || end < start {
          return not_satisfiable();
        }

        // adjust for MAX_LEN
        end = start + (end - start).min(len - start).min(MAX_LEN - 1);

        file.seek(SeekFrom::Start(start)).await?;

        let mut stream: Pin<Box<dyn AsyncRead>> = Box::pin(file);
        if end + 1 < len {
          stream = Box::pin(stream.take(end + 1 - start));
        }

        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await?;

        resp = resp.header(CONTENT_RANGE, format!("bytes {start}-{end}/{len}"));
        resp = resp.header(CONTENT_LENGTH, end + 1 - start);
        resp = resp.status(StatusCode::PARTIAL_CONTENT);
        resp.body(buf)
      } else {
        let mut buf = Vec::new();
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

        drop(file);

        for (end, start) in ranges {
          buf.write_all(boundary_sep.as_bytes()).await?;
          buf
            .write_all(format!("{CONTENT_TYPE}: {mime_type}\r\n").as_bytes())
            .await?;
          buf
            .write_all(format!("{CONTENT_RANGE}: bytes {start}-{end}/{len}\r\n").as_bytes())
            .await?;
          buf.write_all("\r\n".as_bytes()).await?;

          let mut file = File::open(&path).await?;
          file.seek(SeekFrom::Start(start)).await?;
          file
            .take(if end + 1 < len { end + 1 - start } else { len })
            .read_to_end(&mut buf)
            .await?;
        }
        buf.write_all(boundary_closer.as_bytes()).await?;

        resp.body(buf)
      }
    } else {
      resp = resp.header(CONTENT_LENGTH, len);
      let mut buf = vec![0_u8; len as usize];
      file.read_to_end(&mut buf).await?;
      resp.body(buf)
    };

    response
  })
}

fn random_boundary() -> String {
  let mut x = [0_u8; 30];
  rand::thread_rng().fill_bytes(&mut x);
  (x[..])
    .iter()
    .map(|&x| format!("{x:x}"))
    .fold(String::new(), |mut a, x| {
      a.push_str(x.as_str());
      a
    })
}
