#!/usr/bin/env bash

# Copyright 2019-2024 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

echo "Building API definitions..."
cd tooling/api
yarn && yarn build
cd ../..

echo "Building the Tauri Rust CLI..."
cd tooling/cli
cargo install --path .
cd ../..
echo "Tauri Rust CLI installed. Run it with '$ cargo tauri [COMMAND]'."

echo "Do you want to install the Node.js CLI?"
select yn in "Yes" "No"; do
    case $yn in
        Yes )
            cd tooling/cli/node
            yarn && yarn build && yarn link
            cd ../../..
            echo "Tauri Node.js CLI installed. use `yarn link @tauri-apps/cli` and run it with '$ yarn tauri [COMMAND]'."
            break;;
        No ) break;;
    esac
done
