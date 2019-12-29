#!/bin/sh
# Note: Script must be run like this `. .init_env.sh` to setup variables for your current shell
# define relative paths

DistPath="tauri/test/fixture/dist"
SrcPath="tauri/test/fixture/src-tauri"

echo "Setting up enviromental Variables"

# check if relative paths exist
if [ -d "$DistPath" ]||[ -d "$SrcPath" ]
    then
        # Convert to absolute paths
        DistPath="$(cd "$DistPath" && pwd -P)"
        SrcPath="$(cd "$SrcPath" && pwd -P)"

        # export enviromental variables 
        export TAURI_DIST_DIR=$DistPath
        export TAURI_DIR=$SrcPath
        echo "Variables set, ready to work!"
    
else
    # if directories don't exist then exit script and tell user run script in root dir. 
    echo "Error: Variables are not setup properly. Please run from Tauri Root directory '. .scripts/init_env.sh'"
fi
