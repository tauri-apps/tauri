#!/bin/sh
# Note: Script must be run like this `. .setup.sh` to setup variables for your current shell

# define relative paths
DistPath="../dist"
SrcPath="../src-tauri"

# Convert to absolute paths
DistPath="$(cd "$DistPath" && pwd -P)"
SrcPath="$(cd "$SrcPath" && pwd -P)"

# export enviromental variables 
export TAURI_DIST_DIR=$DistPath
export TAURI_DIR=$SrcPath
