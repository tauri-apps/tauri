Write-Output "Checking tauri crates"


$commands=@("clippy","test")
$features=@("no-server","embedded-server")
foreach ($command in $commands) {
  foreach ($feature in $features) {
    Write-Output "[$command][$feature] checking tauri"
    cargo $command --manifest-path tauri/Cargo.toml --all-targets --features "$feature,cli,all-api"
  }

  Write-Output "[$command] checking other crates"
  cargo $command --workspace --exclude tauri --all-targets --all-features
}
