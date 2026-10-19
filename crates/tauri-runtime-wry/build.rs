// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// creates a cfg alias if `has_feature` is true.
// `alias` must be a snake case string.
fn alias(alias: &str, has_feature: bool) {
  println!("cargo:rustc-check-cfg=cfg({alias})");
  if has_feature {
    println!("cargo:rustc-cfg={alias}");
  }
}

fn main() {
  let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
  let mobile = target_os == "ios" || target_os == "android";
  alias("desktop", !mobile);
  alias("mobile", mobile);
}
