Write-Output "Setting up enviromental Variables"
# setup relative paths
$dist_path = "../dist"
$src_path = "../src-tauri"

# convert relative paths to absolute paths. 
# put these absolute paths in enviromental variables
$env:TAURI_DIST_DIR = Resolve-Path $dist_path
$env:TAURI_DIR = Resolve-Path $src_path 

Write-Output "Ready to work"