---
title: Setup for Linux
---

import Alert from '@theme/Alert'
import Icon from '@theme/Icon'
import { Intro } from '@theme/SetupDocs'
import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Intro />

## 1. System Dependencies&nbsp;<Icon title="alert" color="danger"/>

<Tabs
defaultValue="debian"
values={[
{label: 'Debian', value: 'debian'},
{label: 'Arch', value: 'arch'},
{label: 'Fedora', value: 'fedora'},
]}>
<TabItem value="debian">

```sh
$ sudo apt update && sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    squashfs-tools
```

</TabItem>
<TabItem value="arch">

```sh
$ sudo pacman -Syy && sudo pacman -S  webkit2gtk \
    base-devel \
    curl \
    wget \
    openssl \
    appmenu-gtk-module \
    gtk3 \
    squashfs-tools \
    libvips
```

</TabItem>
<TabItem value="fedora">

```sh
$ sudo dnf check-update && sudo dnf install webkit2gtk3-devel.x86_64 \
    openssl-devel \
    curl \
    wget \
    squashfs-tools \
    && sudo dnf group install "C Development Tools and Libraries"
```

</TabItem>
</Tabs>

## 2. Node.js Runtime and Package Manager&nbsp;<Icon title="control-skip-forward" color="warning"/>

### Node.js (npm included)

We recommend using nvm to manage your Node.js runtime. It allows you to easily switch versions and update Node.js.

```sh
$ curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.2/install.sh | bash
```

<Alert title="Note">
We have audited this bash script, and it does what it says it is supposed to do. Nevertheless, before blindly curl-bashing a script, it is always wise to look at it first. Here is the file as a mere <a href="https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.2/install.sh" target="_blank">download link</a>.
</Alert>

Once nvm is installed, close and reopen your terminal, then install the latest version of Node.js and npm:

```sh
$ nvm install node --latest-npm
$ nvm use node
```

If you have any problems with nvm, please consult their <a href="https://github.com/nvm-sh/nvm">project readme</a>.

### Optional Node.js Package Manager

You may want to use an alternative to npm:

- <a href="https://yarnpkg.com/getting-started" target="_blank">Yarn</a>, is preferred by Tauri's team
- <a href="https://pnpm.js.org/en/installation" target="_blank">pnpm</a>

## 3. Rustc and Cargo Package Manager&nbsp;<Icon title="control-skip-forward" color="warning"/>

The following command will install <a href="https://rustup.rs/" target="_blank">rustup</a>, the official installer for <a href="https://www.rust-lang.org/" target="_blank">Rust</a>.

```bash
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

<Alert title="Note">
We have audited this bash script, and it does what it says it is supposed to do. Nevertheless, before blindly curl-bashing a script, it is always wise to look at it first. Here is the file as a mere <a href="https://sh.rustup.rs" target="_blank">download link</a>.
</Alert>

To make sure that Rust has been installed successfully, run the following command:

```sh
$ rustc --version
latest update on 2019-12-19, rust version 1.40.0
```

You may need to restart your terminal if the command does not work.

## 4. For Windows Subsystem for Linux (WSL) Users&nbsp;<Icon title="info-alt" color="info"/>

In order to run a graphical application with WSL, you need to download **one** of these X servers: Xming, Cygwin X, and vcXsrv.
Since vcXsrv has been used internally, it's the one we recommend to install.

### WSL Version 1

Open the X server and then run `export DISPLAY=:0` in the terminal. You should now be able to run any graphical application via the terminal.

### WSL Version 2

You'll need to run a command that is slightly more complex than WSL 1: `export DISPLAY=$(cat /etc/resolv.conf | grep nameserver | awk '{print $2}'):0` and you need to add `-ac` to the X server as an argument. Note: if for some reason this command doesn't work you can use an alternative command such as: `export DISPLAY=$(cat /etc/resolv.conf | grep nameserver | sed 's/.* //g'):0` or you can manually find the Address using `cat /etc/resolve.conf | grep nameserver`.

<Alert type="info" title="Note">

Don't forget that you'll have to use the "export" command anytime you want to use a graphical application, for each newly opened terminal.

You can download some examples to try with `sudo apt-get install x11-apps`. xeyes is always a good one. It can be handy when troubleshooting WSL issues.
</Alert>

## Continue

Now that you have set up the Linux-specific dependencies for Tauri, learn how to [add Tauri to your project](/docs/usage/development/integration).
