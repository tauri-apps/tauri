#!/bin/bash

# Copyright 2019-2024 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

set -euxo pipefail

for o in outputs/*; do
  pushd "$o"

  chmod +x cargo-tauri*
  cp ../../crates/tauri-cli/LICENSE* ../../crates/tauri-cli/README.md .

  target=$(basename "$o" | cut -d. -f1)
  if grep -qE '(apple|windows)' <<< "$target"; then
    zip "../${target}.zip" *
  else
    tar cv * | gzip -9 > "../${target}.tgz"
  fi

  popd
done
