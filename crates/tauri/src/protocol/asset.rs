// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{path::SafePathBuf, scope, webview::UriSchemeProtocolHandler};
use http::{header::*, status::StatusCode, Request, Response};
use http_range::HttpRange;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::{borrow::Cow, io::SeekFrom};
use tauri_utils::mime_type::MimeType;

pub fn get(scope: scope::fs::Scope, window_origin: String) -> UriSchemeProtocolHandler {
  Box::new(move |_, request, responder| {
    let scope = scope.clone();
    let window_origin = window_origin.clone();
    // `get_response` performs blocking filesystem I/O, so it must not run on the
    // thread the webview calls us on, otherwise the whole event loop stalls.
    crate::async_runtime::spawn_blocking(move || {
      match get_response(request, &scope, &window_origin) {
        Ok(response) => responder.respond(response),
        Err(e) => responder.respond(
          http::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .header(CONTENT_TYPE, mime::TEXT_PLAIN.essence_str())
            .header("Access-Control-Allow-Origin", &window_origin)
            .body(e.to_string().into_bytes())
            .unwrap(),
        ),
      }
    });
  })
}

fn get_response(
  request: Request<Vec<u8>>,
  scope: &scope::fs::Scope,
  window_origin: &str,
) -> Result<Response<Cow<'static, [u8]>>, Box<dyn std::error::Error>> {
  // skip leading `/`
  let path = percent_encoding::percent_decode(&request.uri().path().as_bytes()[1..])
    .decode_utf8_lossy()
    .to_string();

  let mut resp = Response::builder().header("Access-Control-Allow-Origin", window_origin);

  if let Err(e) = SafePathBuf::new(path.clone().into()) {
    log::error!("asset protocol path \"{path}\" is not valid: {e}");
    return resp.status(403).body(Vec::new().into()).map_err(Into::into);
  }

  if !scope.is_allowed(&path) {
    log::error!("asset protocol not configured to allow the path: {path}");
    return resp.status(403).body(Vec::new().into()).map_err(Into::into);
  }

  // Separate block for easier error handling
  let mut file = match File::open(path.clone()) {
    Ok(file) => file,
    Err(e) => {
      #[cfg(target_os = "android")]
      {
        if path.starts_with("/storage/emulated/0/Android/data/") {
          log::error!("Failed to open Android external storage file '{path}': {e}. This may be due to missing storage permissions.");
        }
      }
      return if e.kind() == std::io::ErrorKind::NotFound {
        log::error!("File does not exist at path: {path}");
        return resp.status(404).body(Vec::new().into()).map_err(Into::into);
      } else if e.kind() == std::io::ErrorKind::PermissionDenied {
        log::error!("Missing OS permission to access path \"{path}\": {e}");
        return resp.status(403).body(Vec::new().into()).map_err(Into::into);
      } else {
        Err(e.into())
      };
    }
  };

  let len = file.metadata()?.len();
  let (mime_type, read_bytes) = {
    // get file mime type
    let nbytes = len.min(8192);
    let mut magic_buf = Vec::with_capacity(nbytes as usize);
    (&mut file).take(nbytes).read_to_end(&mut magic_buf)?;
    file.rewind()?;
    (
      MimeType::parse(&magic_buf, &path),
      // return the `magic_bytes` if we read the whole file
      // to avoid reading it again later if this is not a range request
      if len < 8192 { Some(magic_buf) } else { None },
    )
  };

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

      let buf = {
        let mut buf = Vec::with_capacity(nbytes as usize);
        file.seek(SeekFrom::Start(start))?;
        file.take(nbytes).read_to_end(&mut buf)?;
        buf
      };

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

      let buf = {
        // multi-part range header
        let mut buf = Vec::new();

        for (start, end) in ranges {
          // a new range is being written, write the range boundary
          buf.write_all(boundary_sep.as_bytes())?;

          // write the needed headers `Content-Type` and `Content-Range`
          buf.write_all(format!("{CONTENT_TYPE}: {mime_type}\r\n").as_bytes())?;
          buf.write_all(format!("{CONTENT_RANGE}: bytes {start}-{end}/{len}\r\n").as_bytes())?;

          // write the separator to indicate the start of the range body
          buf.write_all("\r\n".as_bytes())?;

          // calculate number of bytes needed to be read
          let nbytes = end + 1 - start;

          let mut local_buf = Vec::with_capacity(nbytes as usize);
          file.seek(SeekFrom::Start(start))?;
          (&mut file).take(nbytes).read_to_end(&mut local_buf)?;
          buf.extend_from_slice(&local_buf);
        }
        // all ranges have been written, write the closing boundary
        buf.write_all(boundary_closer.as_bytes())?;

        buf
      };
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
      let mut local_buf = Vec::with_capacity(len as usize);
      file.read_to_end(&mut local_buf)?;
      local_buf
    };
    resp = resp.header(CONTENT_LENGTH, len);
    resp.body(buf.into())
  };

  response.map_err(Into::into)
}

fn random_boundary() -> String {
  let mut x = [0_u8; 30];
  getrandom::fill(&mut x).expect("failed to get random bytes");
  (x[..])
    .iter()
    .map(|&x| format!("{x:x}"))
    .fold(String::new(), |mut a, x| {
      a.push_str(x.as_str());
      a
    })
}

#[cfg(all(test, unix))]
mod tests {
  use crate::app::UriSchemeResponder;
  use crate::Manager;
  use std::sync::mpsc::{channel, TryRecvError};
  use std::time::Duration;

  /// The handler must hand the request off and return, rather than reading the
  /// file on the thread the webview called it on.
  ///
  /// A FIFO stands in for a slow or unreachable path: opening one for reading
  /// blocks until a writer appears, which is the same shape as the unreachable
  /// network share in #7434 without needing one. If the read happened inline,
  /// `handler(..)` below would not return until the writer thread runs.
  #[test]
  fn does_not_block_the_calling_thread() {
    let dir = std::env::temp_dir().join(format!("tauri-asset-protocol-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("blocking-asset");
    let _ = std::fs::remove_file(&path);
    assert!(std::process::Command::new("mkfifo")
      .arg(&path)
      .status()
      .unwrap()
      .success());

    let app = crate::test::mock_app();
    let scope = app.asset_protocol_scope();
    scope.allow_file(&path).unwrap();

    let handler = super::get(scope, "tauri://localhost".into());

    let (tx, rx) = channel();
    let responder = UriSchemeResponder(Box::new(move |response| {
      let _ = tx.send(response);
    }));

    let encoded = percent_encoding::percent_encode(
      path.to_str().unwrap().as_bytes(),
      percent_encoding::NON_ALPHANUMERIC,
    );
    let request = http::Request::builder()
      .uri(format!("asset://localhost/{encoded}"))
      .body(Vec::new())
      .unwrap();

    // call the handler off the test thread so a regression fails the test instead
    // of hanging it
    let (returned_tx, returned_rx) = channel();
    std::thread::spawn(move || {
      handler("main", request, responder);
      let _ = returned_tx.send(());
    });
    returned_rx
      .recv_timeout(Duration::from_secs(30))
      .expect("asset protocol handler did not return before reading the file");

    // it cannot have read the file yet, because nothing has opened the write end
    // of the FIFO
    assert_eq!(rx.try_recv().unwrap_err(), TryRecvError::Empty);

    // unblock the read so the request resolves instead of leaking a blocked thread
    std::thread::spawn(move || {
      let _ = std::fs::write(&path, b"tauri");
    });

    rx.recv_timeout(Duration::from_secs(30))
      .expect("asset protocol never responded");

    let _ = std::fs::remove_dir_all(&dir);
  }
}
