# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/master/containers/ubuntu/.devcontainer/base.Dockerfile
# [Choice] Ubuntu version (use jammy or bionic on local arm64/Apple Silicon): jammy, focal, bionic
ARG VARIANT="jammy"
FROM mcr.microsoft.com/vscode/devcontainers/base:0-${VARIANT}

# Derived from Tauri contribution and setup guides:
# See: https://github.com/tauri-apps/tauri/blob/dev/.github/CONTRIBUTING.md#development-guide
# See: https://v2.tauri.app/start/prerequisites/
ARG TAURI_BUILD_DEPS="build-essential curl libappindicator3-dev libgtk-3-dev librsvg2-dev libssl-dev libwebkit2gtk-4.1-dev wget"

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get install -y --no-install-recommends $TAURI_BUILD_DEPS
