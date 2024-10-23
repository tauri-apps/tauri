// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// taken from https://github.com/oxc-project/oxc/blob/main/crates/oxc_linter/src/partial_loader/mod.rs

mod svelte;
mod vue;

use oxc_span::SourceType;

pub use self::{svelte::SveltePartialLoader, vue::VuePartialLoader};

const SCRIPT_START: &str = "<script";
const SCRIPT_END: &str = "</script>";

#[derive(Debug, Clone, Copy)]
pub struct JavaScriptSource<'a> {
  pub source_text: &'a str,
  pub source_type: SourceType,
  /// The javascript source could be embedded in some file,
  /// use `start` to record start offset of js block in the original file.
  pub start: usize,
}

impl<'a> JavaScriptSource<'a> {
  pub fn new(source_text: &'a str, source_type: SourceType, start: usize) -> Self {
    Self {
      source_text,
      source_type,
      start,
    }
  }
}

pub struct PartialLoader;

impl PartialLoader {
  /// Extract js section of specifial files.
  /// Returns `None` if the specifial file does not have a js section.
  pub fn parse<'a>(ext: &str, source_text: &'a str) -> Option<Vec<JavaScriptSource<'a>>> {
    match ext {
      "vue" => Some(VuePartialLoader::new(source_text).parse()),
      "svelte" => Some(SveltePartialLoader::new(source_text).parse()),
      _ => None,
    }
  }
}

/// Find closing angle for situations where there is another `>` in between.
/// e.g. `<script generic="T extends Record<string, string>">`
fn find_script_closing_angle(source_text: &str, pointer: usize) -> Option<usize> {
  let mut numbers_of_open_angle = 0;
  for (offset, c) in source_text[pointer..].char_indices() {
    match c {
      '>' => {
        if numbers_of_open_angle == 0 {
          return Some(offset);
        }
        numbers_of_open_angle -= 1;
      }
      '<' => {
        numbers_of_open_angle += 1;
      }
      _ => {}
    }
  }
  None
}
