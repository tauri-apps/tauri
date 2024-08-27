// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// taken from https://github.com/oxc-project/oxc/blob/main/crates/oxc_linter/src/partial_loader/svelte.rs

use memchr::memmem::Finder;
use oxc_span::SourceType;

use super::{find_script_closing_angle, JavaScriptSource, SCRIPT_END, SCRIPT_START};

pub struct SveltePartialLoader<'a> {
  source_text: &'a str,
}

impl<'a> SveltePartialLoader<'a> {
  pub fn new(source_text: &'a str) -> Self {
    Self { source_text }
  }

  pub fn parse(self) -> Vec<JavaScriptSource<'a>> {
    self
      .parse_script()
      .map_or_else(Vec::new, |source| vec![source])
  }

  fn parse_script(&self) -> Option<JavaScriptSource<'a>> {
    let script_start_finder = Finder::new(SCRIPT_START);
    let script_end_finder = Finder::new(SCRIPT_END);

    let mut pointer = 0;

    // find opening "<script"
    let offset = script_start_finder.find(self.source_text[pointer..].as_bytes())?;
    pointer += offset + SCRIPT_START.len();

    // find closing ">"
    let offset = find_script_closing_angle(self.source_text, pointer)?;

    // get lang="ts" attribute
    let content = &self.source_text[pointer..pointer + offset];
    let is_ts = content.contains("ts");

    pointer += offset + 1;
    let js_start = pointer;

    // find "</script>"
    let offset = script_end_finder.find(self.source_text[pointer..].as_bytes())?;
    let js_end = pointer + offset;

    let source_text = &self.source_text[js_start..js_end];
    let source_type = SourceType::default()
      .with_module(true)
      .with_typescript(is_ts);
    Some(JavaScriptSource::new(source_text, source_type, js_start))
  }
}

#[cfg(test)]
mod test {
  use super::{JavaScriptSource, SveltePartialLoader};

  fn parse_svelte(source_text: &str) -> JavaScriptSource<'_> {
    let sources = SveltePartialLoader::new(source_text).parse();
    *sources.first().unwrap()
  }

  #[test]
  fn test_parse_svelte() {
    let source_text = r#"
        <script>
          console.log("hi");
        </script>
        <h1>Hello World</h1>
        "#;

    let result = parse_svelte(source_text);
    assert_eq!(result.source_text.trim(), r#"console.log("hi");"#);
  }

  #[test]
  fn test_parse_svelte_ts_with_generic() {
    let source_text = r#"
        <script lang="ts" generics="T extends Record<string, unknown>">
          console.log("hi");
        </script>
        <h1>Hello World</h1>
        "#;

    let result = parse_svelte(source_text);
    assert_eq!(result.source_text.trim(), r#"console.log("hi");"#);
  }
}
