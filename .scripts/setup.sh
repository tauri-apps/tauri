#!/usr/bin/env sh
echo "Building API definitions..."
cd api
yarn && yarn build
cd ..

echo "Building the Tauri CLI..."
cd cli/core
cargo build --release
cd ../..

alias rtauri="$(pwd)/cli/core/target/release/cargo-tauri.exe tauri"
echo "Added alias 'rtauri' for '$(pwd)/cli/core/target/release/cargo-tauri.exe tauri'"
echo "Tauri CLI installed. Run it with '$ rtauri [COMMAND]'."

echo "Do you want to use the Node.js CLI?"
select yn in "Yes" "No"; do
    case $yn in
        Yes )
            cd cli/tauri.js
            yarn && yarn build
            cd ../..
            alias stauri="$(pwd)/cli/tauri.js/bin/tauri.js"
            echo "Added alias 'stauri' for '$(pwd)/cli/tauri.js/bin/tauri.js'"
            echo "Tauri Node.js CLI installed. Run it with '$ stauri [COMMAND]'"
            break;;
        No ) break;;
    esac
done
