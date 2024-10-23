// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use http::{header::*, response::Builder as ResponseBuilder, status::StatusCode};
use http_range::HttpRange;
use std::{
  io::{Read, Seek, SeekFrom, Write},
  path::PathBuf,
  process::{Command, Stdio},
};

fn get_stream_response(
  request: http::Request<Vec<u8>>,
) -> Result<http::Response<Vec<u8>>, Box<dyn std::error::Error>> {
  // skip leading `/`
  let path = percent_encoding::percent_decode(request.uri().path()[1..].as_bytes())
    .decode_utf8_lossy()
    .to_string();

  // return error 404 if it's not our video
  if path != "streaming_example_test_video.mp4" {
    return Ok(ResponseBuilder::new().status(404).body(Vec::new())?);
  }

  let mut file = std::fs::File::open(&path)?;

  // get file length
  let len = {
    let old_pos = file.stream_position()?;
    let len = file.seek(SeekFrom::End(0))?;
    file.seek(SeekFrom::Start(old_pos))?;
    len
  };

  let mut resp = ResponseBuilder::new().header(CONTENT_TYPE, "video/mp4");

  // if the webview sent a range header, we need to send a 206 in return
  let http_response = if let Some(range_header) = request.headers().get("range") {
    let not_satisfiable = || {
      ResponseBuilder::new()
        .status(StatusCode::RANGE_NOT_SATISFIABLE)
        .header(CONTENT_RANGE, format!("bytes */{len}"))
        .body(vec![])
    };

    // parse range header
    let ranges = if let Ok(ranges) = HttpRange::parse(range_header.to_str()?, len) {
      ranges
        .iter()
        // map the output back to spec range <start-end>, example: 0-499
        .map(|r| (r.start, r.start + r.length - 1))
        .collect::<Vec<_>>()
    } else {
      return Ok(not_satisfiable()?);
    };

    /// The Maximum bytes we send in one range
    const MAX_LEN: u64 = 1000 * 1024;

    if ranges.len() == 1 {
      let &(start, mut end) = ranges.first().unwrap();

      // check if a range is not satisfiable
      //
      // this should be already taken care of by HttpRange::parse
      // but checking here again for extra assurance
      if start >= len || end >= len || end < start {
        return Ok(not_satisfiable()?);
      }

      // adjust end byte for MAX_LEN
      end = start + (end - start).min(len - start).min(MAX_LEN - 1);

      // calculate number of bytes needed to be read
      let bytes_to_read = end + 1 - start;

      // allocate a buf with a suitable capacity
      let mut buf = Vec::with_capacity(bytes_to_read as usize);
      // seek the file to the starting byte
      file.seek(SeekFrom::Start(start))?;
      // read the needed bytes
      file.take(bytes_to_read).read_to_end(&mut buf)?;

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
          // this should be already taken care of by HttpRange::parse
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

      for (end, start) in ranges {
        // a new range is being written, write the range boundary
        buf.write_all(boundary_sep.as_bytes())?;

        // write the needed headers `Content-Type` and `Content-Range`
        buf.write_all(format!("{CONTENT_TYPE}: video/mp4\r\n").as_bytes())?;
        buf.write_all(format!("{CONTENT_RANGE}: bytes {start}-{end}/{len}\r\n").as_bytes())?;

        // write the separator to indicate the start of the range body
        buf.write_all("\r\n".as_bytes())?;

        // calculate number of bytes needed to be read
        let bytes_to_read = end + 1 - start;

        let mut local_buf = vec![0_u8; bytes_to_read as usize];
        file.seek(SeekFrom::Start(start))?;
        file.read_exact(&mut local_buf)?;
        buf.extend_from_slice(&local_buf);
      }
      // all ranges have been written, write the closing boundary
      buf.write_all(boundary_closer.as_bytes())?;

      resp.body(buf)
    }
  } else {
    resp = resp.header(CONTENT_LENGTH, len);
    let mut buf = Vec::with_capacity(len as usize);
    file.read_to_end(&mut buf)?;
    resp.body(buf)
  };

  http_response.map_err(Into::into)
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

fn download_video() {
  let video_file = PathBuf::from("streaming_example_test_video.mp4");
  if !video_file.exists() {
    let video_url =
      "http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4";

    // Downloading with curl this saves us from adding
    // a Rust HTTP client dependency.
    println!("Downloading {video_url}");
    let status = Command::new("curl")
      .arg("-L")
      .arg("-o")
      .arg(&video_file)
      .arg(video_url)
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .output()
      .unwrap();

    assert!(status.status.success());
    assert!(video_file.exists());
  }
}

fn main() {
  download_video();

  tauri::Builder::default()
    .register_asynchronous_uri_scheme_protocol("stream", move |_ctx, request, responder| {
      match get_stream_response(request) {
        Ok(http_response) => responder.respond(http_response),
        Err(e) => responder.respond(
          ResponseBuilder::new()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(CONTENT_TYPE, "text/plain")
            .body(e.to_string().as_bytes().to_vec())
            .unwrap(),
        ),
      }
    })
    .run(tauri::generate_context!(
      "../../examples/streaming/tauri.conf.json"
    ))
    .expect("error while running tauri application");
}
