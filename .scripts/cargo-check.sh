#!/usr/bin/env sh
# Copyright 2019-2021 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

# note: you can pass in the cargo sub-commands used to check manually.
# allowed commands: check, clippy, fmt, test
# default: clippy, fmt, test

# exit the script early if any of the commands return an error
set -e

# set the script arguments if none are found
if [ -z "$*" ]; then
  set -- "clippy" "fmt" "test"
fi

# run n+1 times, where n is the amount of mutually exclusive features.
# the extra run is for all the crates without mutually exclusive features.
# as many features as possible are enabled at for each command
run() {
  command=$1
  shift 1
  cargo "$command" --workspace --all-targets --all-features "$@"
}

for command in "$@"; do
  case "$command" in
  check | test)
    run "$command"
    ;;
  clippy)
    run clippy -- -D warnings
    ;;
  fmt)
    echo "[$command] checking formatting"
    cargo +nightly fmt -- --check
    ;;
  *)
    echo "[cargo-check.sh] Unknown cargo sub-command: $command"
    exit 1
    ;;
  esac
done
