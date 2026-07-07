// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

fn main() {
    #[cfg(target_os = "windows")]
    embed_resource::compile("manifest.rc", embed_resource::NONE);
}
