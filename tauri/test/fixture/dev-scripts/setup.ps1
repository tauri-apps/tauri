Write-Output "Setting up enviromental Variables"

$dist_path = "../dist"
$src_path = "../src-tauri"

$env:TAURI_DIST_DIR = Resolve-Path $dist_path
$env:TAURI_DIR = Resolve-Path $src_path 

Write-Output "Ready to work"