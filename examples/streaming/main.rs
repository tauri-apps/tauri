// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  use std::{
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    process::{Command, Stdio},
  };
  use tauri::http::{header::*, status::StatusCode, HttpRange, ResponseBuilder};

  let video_file = PathBuf::from("test_video.mp4");
  let video_url =
    "http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4";

  if !video_file.exists() {
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

  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![video_uri])
    .register_uri_scheme_protocol("stream", move |_app, request| {
      // get the file path
      let path = request.uri().strip_prefix("stream://localhost/").unwrap();
      let path = percent_encoding::percent_decode(path.as_bytes())
        .decode_utf8_lossy()
        .to_string();

      if path != "example/test_video.mp4" {
        // return error 404 if it's not our video
        return ResponseBuilder::new().status(404).body(Vec::new());
      }

      let mut file = std::fs::File::open(&path)?;

      let len = {
        let old_pos = file.stream_position()?;
        let len = file.seek(SeekFrom::End(0))?;
        file.seek(SeekFrom::Start(old_pos))?;
        len
      };

      let mut resp = ResponseBuilder::new().header(CONTENT_TYPE, "video/mp4");

      // if the webview sent a range header, we need to send a 206 in return
      // Actually only macOS and Windows are supported. Linux will ALWAYS return empty headers.
      let response = if let Some(range_header) = request.headers().get("range") {
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
          return not_satisfiable();
        };

        /// only send 1MB or less at a time
        const MAX_LEN: u64 = 1000 * 1024;

        if ranges.len() == 1 {
          let &(start, mut end) = ranges.first().unwrap();

          // check if a range is not satisfiable
          //
          // this should be already taken care of by HttpRange::parse
          // but checking here again for extra assurance
          if start >= len || end >= len || end < start {
            return not_satisfiable();
          }

          // adjust for MAX_LEN
          end = start + (end - start).min(len - start).min(MAX_LEN - 1);

          file.seek(SeekFrom::Start(start))?;

          let mut stream: Box<dyn Read> = Box::new(file);
          if end + 1 < len {
            stream = Box::new(stream.take(end + 1 - start));
          }

          let mut buf = Vec::new();
          stream.read_to_end(&mut buf)?;

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
                end = start + (end - start).min(len - start).min(MAX_LEN - 1);
                Some((start, end))
              }
            })
            .collect::<Vec<_>>();

          let boundary = "sadasdq2e";
          let boundary_sep = format!("\r\n--{boundary}\r\n");
          let boundary_closer = format!("\r\n--{boundary}\r\n");

          resp = resp.header(
            CONTENT_TYPE,
            format!("multipart/byteranges; boundary={boundary}"),
          );

          drop(file);

          for (end, start) in ranges {
            buf.write_all(boundary_sep.as_bytes())?;
            buf.write_all(format!("{CONTENT_TYPE}: video/mp4\r\n").as_bytes())?;
            buf.write_all(format!("{CONTENT_RANGE}: bytes {start}-{end}/{len}\r\n").as_bytes())?;
            buf.write_all("\r\n".as_bytes())?;

            let mut file = std::fs::File::open(&path)?;
            file.seek(SeekFrom::Start(start))?;
            file
              .take(if end + 1 < len { end + 1 - start } else { len })
              .read_to_end(&mut buf)?;
          }
          buf.write_all(boundary_closer.as_bytes())?;

          resp.body(buf)
        }
      } else {
        resp = resp.header(CONTENT_LENGTH, len);
        let mut buf = vec![0; len as usize];
        file.read_to_end(&mut buf)?;
        resp.body(buf)
      };

      response
    })
    .run(tauri::generate_context!(
      "../../examples/streaming/tauri.conf.json"
    ))
    .expect("error while running tauri application");
}

// returns the scheme and the path of the video file
// we're using this just to allow using the custom `stream` protocol or tauri built-in `asset` protocol
#[tauri::command]
fn video_uri() -> (&'static str, std::path::PathBuf) {
  #[cfg(feature = "protocol-asset")]
  {
    let mut path = std::env::current_dir().unwrap();
    path.push("test_video.mp4");
    ("asset", path)
  }

  #[cfg(not(feature = "protocol-asset"))]
  ("stream", "example/test_video.mp4".into())
}

// fn random_boundary() -> String {
//   use rand::RngCore;

//   let mut x = [0 as u8; 30];
//   rand::thread_rng().fill_bytes(&mut x);
//   (&x[..])
//     .iter()
//     .map(|&x| format!("{:x}", x))
//     .fold(String::new(), |mut a, x| {
//       a.push_str(x.as_str());
//       a
//     })
// }
