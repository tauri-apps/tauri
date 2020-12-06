#!/usr/bin/env sh
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
mutex() {
  command=$1
  shift 1

  for feature in "no-server" "embedded-server"; do
    echo "[$command][$feature] tauri"
    cargo "$command" --manifest-path tauri/Cargo.toml --all-targets --features "$feature,cli,all-api" "$@"
  done

  echo "[$command] other crates"
  cargo "$command" --workspace --exclude tauri --all-targets --all-features "$@"
}

for command in "$@"; do
  case "$command" in
  check | test)
    mutex "$command"
    ;;
  clippy)
    mutex clippy -- -D warnings
    ;;
  fmt)
    echo "[$command] checking formatting"
    cargo fmt -- --check
    ;;
  *)
    echo "[cargo-check.sh] Unknown cargo sub-command: $command"
    exit 1
    ;;
  esac
done
