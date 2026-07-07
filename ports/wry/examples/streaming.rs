// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

fn main() -> wry::Result<()> {
  imp::main()
}

#[cfg(not(feature = "protocol"))]
mod imp {
  pub fn main() -> wry::Result<()> {
    unimplemented!()
  }
}

#[cfg(feature = "protocol")]
mod imp {

  use std::{
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
  };

  use http::{header, StatusCode};
  use http_range::HttpRange;
  use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
  };
  use wry::{
    http::{header::*, Request, Response},
    WebViewBuilder,
  };

  pub fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let builder = WebViewBuilder::new()
      .with_custom_protocol(
        "wry".into(),
        move |_webview_id, request| match wry_protocol(request) {
          Ok(r) => r.map(Into::into),
          Err(e) => http::Response::builder()
            .header(CONTENT_TYPE, "text/plain")
            .status(500)
            .body(e.to_string().as_bytes().to_vec())
            .unwrap()
            .map(Into::into),
        },
      )
      .with_custom_protocol(
        "stream".into(),
        move |_webview_id, request| match stream_protocol(request) {
          Ok(r) => r.map(Into::into),
          Err(e) => http::Response::builder()
            .header(CONTENT_TYPE, "text/plain")
            .status(500)
            .body(e.to_string().as_bytes().to_vec())
            .unwrap()
            .map(Into::into),
        },
      )
      // tell the webview to load the custom protocol
      .with_url("wry://localhost");

    #[cfg(any(
      target_os = "windows",
      target_os = "macos",
      target_os = "ios",
      target_os = "android"
    ))]
    let _webview = builder.build(&window)?;
    #[cfg(not(any(
      target_os = "windows",
      target_os = "macos",
      target_os = "ios",
      target_os = "android"
    )))]
    let _webview = {
      use tao::platform::unix::WindowExtUnix;
      use wry::WebViewBuilderExtUnix;
      let vbox = window.default_vbox().unwrap();
      builder.build_gtk(vbox)?
    };

    event_loop.run(move |event, _, control_flow| {
      *control_flow = ControlFlow::Wait;

      if let Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } = event
      {
        *control_flow = ControlFlow::Exit
      }
    });
  }

  fn wry_protocol(
    request: Request<Vec<u8>>,
  ) -> Result<http::Response<Vec<u8>>, Box<dyn std::error::Error>> {
    let path = request.uri().path();
    // Read the file content from file path
    let root = PathBuf::from("examples/streaming");
    let path = if path == "/" {
      "index.html"
    } else {
      //  removing leading slash
      &path[1..]
    };
    let content = std::fs::read(std::fs::canonicalize(root.join(path))?)?;

    // Return asset contents and mime types based on file extentions
    // If you don't want to do this manually, there are some crates for you.
    // Such as `infer` and `mime_guess`.
    let mimetype = if path.ends_with(".html") || path == "/" {
      "text/html"
    } else if path.ends_with(".js") {
      "text/javascript"
    } else {
      unimplemented!();
    };

    Response::builder()
      .header(CONTENT_TYPE, mimetype)
      .body(content)
      .map_err(Into::into)
  }

  fn stream_protocol(
    request: http::Request<Vec<u8>>,
  ) -> Result<http::Response<Vec<u8>>, Box<dyn std::error::Error>> {
    // skip leading `/`
    let path = percent_encoding::percent_decode(&request.uri().path().as_bytes()[1..])
      .decode_utf8_lossy()
      .to_string();

    let mut file = std::fs::File::open(path)?;

    // get file length
    let len = {
      let old_pos = file.stream_position()?;
      let len = file.seek(SeekFrom::End(0))?;
      file.seek(SeekFrom::Start(old_pos))?;
      len
    };

    let mut resp = Response::builder().header(CONTENT_TYPE, "video/mp4");

    // if the webview sent a range header, we need to send a 206 in return
    // Actually only macOS and Windows are supported. Linux will ALWAYS return empty headers.
    let http_response = if let Some(range_header) = request.headers().get("range") {
      let not_satisfiable = || {
        Response::builder()
          .status(StatusCode::RANGE_NOT_SATISFIABLE)
          .header(header::CONTENT_RANGE, format!("bytes */{len}"))
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
    getrandom::fill(&mut x).expect("failed to get random bytes");
    (x[..])
      .iter()
      .map(|&x| format!("{x:x}"))
      .fold(String::new(), |mut a, x| {
        a.push_str(x.as_str());
        a
      })
  }
}
