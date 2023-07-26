// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

fn main() {
  tauri_build::build();

  let mut codegen = tauri_build::CodegenContext::new();
  if !cfg!(feature = "custom-protocol") {
    codegen = codegen.dev();
  }
  codegen.build();
}
