#!/bin/bash

# Copyright 2019-2023 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

if git diff --quiet --ignore-submodules HEAD
then
  echo "working directory is clean"
else
  echo "found diff"
  exit 1
fi
