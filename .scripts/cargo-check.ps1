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

function run {
  $command, $_args = $args

  Write-Output "[$command]"
  cargo $command --workspace --all-targets --all-features $_args
  check_error
}

foreach ($command in $args) {
  Switch ($command) {
    "check" {
      run check
      break
    }
    "test" {
      run test
      break
    }
    "clippy" {
      run clippy "--" -D warnings
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
