// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Generates the frontend of the app: around 4 MB of scripts, styles and binary files,
//! which is what the bundle of a real application looks like to the asset protocol.
//! It is generated instead of committed to keep megabytes of filler out of the repository,
//! and it is deterministic, so every build embeds the same bytes.

use std::{fmt::Write, fs, path::Path};

const CHUNKS: usize = 20;
const IMAGES: usize = 24;

struct Lcg(u64);

impl Lcg {
  fn next(&mut self) -> u32 {
    self.0 = self
      .0
      .wrapping_mul(6364136223846793005)
      .wrapping_add(1442695040888963407);
    (self.0 >> 33) as u32
  }
}

/// Minified looking JavaScript: it compresses about as well as real code does.
fn script(namespace: &str, size: usize, random: &mut Lcg) -> String {
  let mut script = format!("window.__bench=window.__bench||{{}};window.__bench.{namespace}={{}};");
  let mut index = 0;
  while script.len() < size {
    write!(
      script,
      "window.__bench.{namespace}.f{index}=function(a,b){{var c=a*{}+b%{};if(c>{}){{return c-{}}}return \"{:x}\"+c}};",
      random.next() % 1000,
      random.next() % 97 + 1,
      random.next(),
      random.next() % 4096,
      random.next()
    )
    .unwrap();
    index += 1;
  }
  script
}

fn stylesheet(size: usize, random: &mut Lcg) -> String {
  let mut css = String::new();
  let mut index = 0;
  while css.len() < size {
    write!(
      css,
      ".c{index}{{margin:{}px;padding:{}px {}px;color:#{:06x};border-radius:{}px}}",
      random.next() % 64,
      random.next() % 32,
      random.next() % 32,
      random.next() % 0xffffff,
      random.next() % 16
    )
    .unwrap();
    index += 1;
  }
  css
}

/// Random bytes: like images and fonts, they do not compress.
fn binary(size: usize, random: &mut Lcg) -> Vec<u8> {
  (0..size).map(|_| random.next() as u8).collect()
}

/// Only touches the file when its content changed, so the build is not invalidated every time.
fn write(path: &Path, content: impl AsRef<[u8]>) {
  let content = content.as_ref();
  if fs::read(path).is_ok_and(|current| current == content) {
    return;
  }
  fs::create_dir_all(path.parent().unwrap()).unwrap();
  fs::write(path, content).unwrap();
}

fn main() {
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=../frontend");

  let dist = Path::new("../dist");
  let mut random = Lcg(2024);

  let mut html = String::from(
    "<!doctype html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"UTF-8\" />\n    <title>Tauri assets benchmark</title>\n    <link rel=\"stylesheet\" href=\"assets/style.css\" />\n  </head>\n  <body>\n    <h1>Tauri assets benchmark</h1>\n    <script src=\"assets/vendor.js\"></script>\n",
  );

  write(
    &dist.join("assets/vendor.js"),
    script("vendor", 1536 * 1024, &mut random),
  );
  write(
    &dist.join("assets/style.css"),
    stylesheet(160 * 1024, &mut random),
  );
  for chunk in 0..CHUNKS {
    write(
      &dist.join(format!("assets/chunk-{chunk}.js")),
      script(&format!("chunk{chunk}"), 40 * 1024, &mut random),
    );
    writeln!(
      html,
      "    <script src=\"assets/chunk-{chunk}.js\"></script>"
    )
    .unwrap();
  }
  for image in 0..IMAGES {
    write(
      &dist.join(format!("assets/image-{image}.bin")),
      binary(30 * 1024, &mut random),
    );
  }
  write(
    &dist.join("assets/data.bin"),
    binary(1024 * 1024, &mut random),
  );

  write(
    &dist.join("main.js"),
    fs::read_to_string("../frontend/main.js")
      .unwrap()
      .replace("__IMAGES__", &IMAGES.to_string())
      .replace("__CHUNKS__", &CHUNKS.to_string()),
  );
  html.push_str("    <script src=\"main.js\"></script>\n  </body>\n</html>\n");
  write(&dist.join("index.html"), html);

  tauri_build::build()
}
