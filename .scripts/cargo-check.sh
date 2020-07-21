#!/usr/bin/env sh
set -e

echo "Checking tauri crates"

for command in "clippy" "test"
do
  for feature in "no-server" "embedded-server"
  do
    echo "[$command][$feature] checking tauri"
    cargo "$command" --manifest-path tauri/Cargo.toml --all-targets --features "$feature,cli,all-api"
  done

  echo "[$command] checking other crates"
  cargo "$command" --workspace --exclude tauri --all-targets --all-features
done
