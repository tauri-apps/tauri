#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0 OR MIT

echo "Building API definitions..."
cd api
yarn && yarn build
cd ..

echo "Building the Tauri Rust CLI..."
cd cli/core
cargo install --path .
cd ../..
echo "Tauri Rust CLI installed. Run it with '$ cargo tauri [COMMAND]'."

echo "Do you want to install the Node.js CLI?"
select yn in "Yes" "No"; do
    case $yn in
        Yes )
            cd cli/tauri.js
            yarn && yarn build && yarn link
            cd ../..
            echo "Tauri Node.js CLI installed. Run it with '$ tauri [COMMAND]'."
            break;;
        No ) break;;
    esac
done
