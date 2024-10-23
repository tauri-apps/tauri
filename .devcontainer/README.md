# VS Code Devcontainer for Tauri

## Overview

Please note that most of these instructions are derived from Microsoft's VS Code documentation: [Developing inside a Container](https://code.visualstudio.com/docs/remote/containers). Check the official documentation if you encounter problems and submit a PR with any corrections you find for the instructions below.

The development container included in this repository is derived from [Microsoft's default Ubuntu development container](https://github.com/microsoft/vscode-dev-containers/tree/master/containers/ubuntu). Contents of the Ubuntu Docker image can be in the [VS Code devcontainer Ubuntu base Dockerfile](https://github.com/microsoft/vscode-dev-containers/blob/main/containers/ubuntu/.devcontainer/base.Dockerfile). The contents of the container used for development can be found in the [Dockerfile](./Dockerfile) located in the same directory as this README.

## Usage

1. Ensure you have all [Devcontainer Prerequisites](#devcontainer-prerequisites)
2. Open the directory containing your [`tauri-apps/tauri`](https://github.com/tauri-apps/tauri) code.
3. Install the [Remote Development](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.vscode-remote-extensionpack) extension pack for VS Code. This will be included if you install recommended workspace extensions upon opening this repository.
4. Ensure Docker is running
5. [Open your workspace in the provided devcontainer](https://code.visualstudio.com/docs/remote/containers#_open-an-existing-workspace-in-a-container): Open this repository in VS Code and run **Remote-Containers: Reopen in Container...** from the Command Palette (<kbd>F1</kbd>).

### Devcontainer Prerequisites

Prerequisites are mainly derived from VS Code's instructions for usage of development containers, documented here: [Developing inside a Container: Getting Started](https://code.visualstudio.com/docs/remote/containers#_getting-started).

1. Docker (Docker Desktop recommended)
2. VS Code
3. X window host - required if you want to be able to interact with a GUI from your Docker host

### A note on filesystem performance

Due to limitations in how Docker shares files between the Docker host and a container, it's also recommended that developers [clone Tauri source code into a container volume](https://code.visualstudio.com/remote/advancedcontainers/improve-performance#_use-clone-repository-in-container-volume). This is optional, but highly advised as many filesystem/IO heavy operations (`cargo build`, `pnpm install`, etc) will be very slow if they operate on directories shared with a Docker container from the Docker host.

To do this, open your project with VS Code and run **Remote-Containers: Clone Repository in Container Volume...** from the Command Palette (<kbd>F1</kbd>).

### Accessing a Tauri application running you the devcontainer

Docker Desktop provides facilities for [allowing the development container to connect to a service on the Docker host](https://docs.docker.com/desktop/windows/networking/#i-want-to-connect-from-a-container-to-a-service-on-the-host). So long as you have an X window server running on your Docker host, the devcontainer can connect to it and expose your Tauri GUI via an X window.

**Export the `DISPLAY` variable within the devcontainer terminal you launch your Tauri application from to expose your GUI outside of the devcontainer**.

```bash
export DISPLAY="host.docker.internal:0"
```
