#!/bin/sh

docker build -t tauri:arm64 --file .docker/cross/aarch64.Dockerfile .docker/cross
