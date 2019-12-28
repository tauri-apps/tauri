#!/bin/sh
# Note: Script must be run like this `. .init_env.sh` to setup variables for your current shell

# define relative paths
DistPath="tauri/test/fixture/dist"
SrcPath="tauri/test/fixture/src-tauri"

# Convert to absolute paths
DistPath="$(cd "$DistPath" && pwd -P)"
SrcPath="$(cd "$SrcPath" && pwd -P)"

# export enviromental variables 
export TAURI_DIST_DIR=$DistPath
export TAURI_DIR=$SrcPath
