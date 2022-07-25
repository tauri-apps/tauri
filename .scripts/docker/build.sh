#!/bin/sh

docker build -t ghcr.io/tauri-apps/tauri/aarch64-unknown-linux-gnu:latest --file .docker/cross/aarch64.Dockerfile .docker/cross
