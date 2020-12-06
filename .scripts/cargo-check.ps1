#!/usr/bin/env pwsh
# note: you can pass in the cargo sub-commands used to check manually.
# allowed commands: check, clippy, fmt, test
# default: clippy, fmt, test

# set the script arguments if none are found
if(-Not $args) {
  $args=@("clippy","fmt","test")
}

# exit the script early if the last command returned an error
function check_error {
  if($LASTEXITCODE -ne 0 ) {
    Exit $LASTEXITCODE
  }
}

# run n+1 times, where n is the amount of mutually exclusive features.
# the extra run is for all the crates without mutually exclusive features.
# as many features as possible are enabled at for each command
function mutex {
  $command, $_args = $args

  foreach ($feature in @("no-server","embedded-server")) {
    Write-Output "[$command][$feature] tauri"
    cargo $command --manifest-path tauri/Cargo.toml --all-targets --features "$feature,cli,all-api" $_args
    check_error
  }

  Write-Output "[$command] other crates"
  cargo $command --workspace --exclude tauri --all-targets --all-features $_args
  check_error
}

foreach ($command in $args) {
  Switch ($command) {
    "check" {
      mutex check
      break
    }
    "test" {
      mutex test
      break
    }
    "clippy" {
      mutex clippy "--" -D warnings
      break
    }
    "fmt" {
      Write-Output "[$command] checking formatting"
      cargo fmt "--" --check
      check_error
    }
    default {
      Write-Output "[cargo-check.ps1] Unknown cargo sub-command: $command"
      Exit 1
    }
  }
}
