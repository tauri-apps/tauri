// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tao_macros::generate_package_name;

pub const PACKAGE: &str = generate_package_name!(com_example, tao_app);

fn main() {}

#[test]
fn it_works() {
  assert_eq!(PACKAGE, "com/example/tao_app")
}
