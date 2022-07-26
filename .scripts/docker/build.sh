#!/bin/sh

docker build -t aarch64-unknown-linux-gnu:latest --file .docker/cross/aarch64.Dockerfile .docker/cross
