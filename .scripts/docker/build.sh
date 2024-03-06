#!/bin/sh

# Copyright 2019-2024 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

docker build -t aarch64-unknown-linux-gnu:latest --file .docker/cross/aarch64.Dockerfile .docker/cross
