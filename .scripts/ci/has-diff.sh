#!/bin/bash

# Copyright 2019-2024 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

git_output=$(git diff --ignore-submodules --name-only HEAD)
if [ -z "$git_output" ];
then
  echo "✔ working directory is clean"
else
  echo "✘ found diff:"
  echo "$git_output"
  exit 1
fi
