<img src=".github/splash.png" alt="Tauri" />

[![status](https://img.shields.io/badge/status-stable-blue.svg)](https://github.com/tauri-apps/tauri/tree/dev)
[![License](https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg)](https://opencollective.com/tauri)
[![test library](https://img.shields.io/github/workflow/status/tauri-apps/tauri/test%20library?label=test%20library)](https://github.com/tauri-apps/tauri/actions?query=workflow%3A%22test+library%22)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri?ref=badge_shield)
[![Chat Server](https://img.shields.io/badge/chat-discord-7289da.svg)](https://discord.gg/SpmNs4S)
[![website](https://img.shields.io/badge/website-tauri.app-purple.svg)](https://tauri.app)
[![https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg](https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg)](https://good-labs.github.io/greater-good-affirmation)
[![support](https://img.shields.io/badge/sponsor-Open%20Collective-blue.svg)](https://opencollective.com/tauri)

## 当前版本

### 核心

| 元件                                                                                    | 描述                               | 版本                                                                                                  | Lin | Win | Mac |
| -------------------------------------------------------------------------------------------- | ----------------------------------------- | -------------------------------------------------------------------------------------------------------- | --- | --- | --- |
| [**tauri**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri)                         | 运行时核心                                 | [![](https://img.shields.io/crates/v/tauri.svg)](https://crates.io/crates/tauri)                         | ✅  | ✅  | ✅  |
| [**tauri-build**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-build)             | 在构建时应用宏(marco)                      | [![](https://img.shields.io/crates/v/tauri-build.svg)](https://crates.io/crates/tauri-build)             | ✅  | ✅  | ✅  |
| [**tauri-codegen**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-codegen)         | 处理资产，解析tauri.conf.json              | [![](https://img.shields.io/crates/v/tauri-codegen.svg)](https://crates.io/crates/tauri-codegen)         | ✅  | ✅  | ✅  |
| [**tauri-macros**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-macros)           | 使用 tauri-codegen 创建宏(marco)           | [![](https://img.shields.io/crates/v/tauri-macros.svg)](https://crates.io/crates/tauri-macros)           | ✅  | ✅  | ✅  |
| [**tauri-runtime**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-runtime)         | Tauri 和 webview 库之间的层                | [![](https://img.shields.io/crates/v/tauri-runtime.svg)](https://crates.io/crates/tauri-runtime)         | ✅  | ✅  | ✅  |
| [**tauri-runtime-wry**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-runtime-wry) | 通过 WRY 实现系统级交互                     | [![](https://img.shields.io/crates/v/tauri-runtime-wry.svg)](https://crates.io/crates/tauri-runtime-wry) | ✅  | ✅  | ✅  |
| [**tauri-utils**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-utils)             | tauri 工具箱使用的通用代码                  | [![](https://img.shields.io/crates/v/tauri-utils.svg)](https://crates.io/crates/tauri-utils)             | ✅  | ✅  | ✅  |

### 工具

| 元件                                                                   | 描述                              | 版本                                                                                                | Lin | Win | Mac |
| --------------------------------------------------------------------------- | ---------------------------------------- | ------------------------------------------------------------------------------------------------------ | --- | --- | --- |
| [**bundler**](https://github.com/tauri-apps/tauri/tree/dev/tooling/bundler) | 生成最终二进制文件                        | [![](https://img.shields.io/crates/v/tauri-bundler.svg)](https://crates.io/crates/tauri-bundler)       | ✅  | ✅  | ✅  |
| [**api.js**](https://github.com/tauri-apps/tauri/tree/dev/tooling/api)      | 用于与 Rust 后端交互的 JS API             | [![](https://img.shields.io/npm/v/@tauri-apps/api.svg)](https://www.npmjs.com/package/@tauri-apps/api) | ✅  | ✅  | ✅  |
| [**cli.rs**](https://github.com/tauri-apps/tauri/tree/dev/tooling/cli)      | 创建、开发和构建应用程序                   | [![](https://img.shields.io/crates/v/tauri-cli.svg)](https://crates.io/crates/tauri-cli)               | ✅  | ✅  | ✅  |
| [**cli.js**](https://github.com/tauri-apps/tauri/tree/dev/tooling/cli/node) | 用于 cli.rs 的 node.js CLI 包装器           | [![](https://img.shields.io/npm/v/@tauri-apps/cli.svg)](https://www.npmjs.com/package/@tauri-apps/cli) | ✅  | ✅  | ✅  |

### 实用程序和插件

| 元件                                                                       | 描述                           | 版本                                                                                                          | Lin | Win | Mac |
| ------------------------------------------------------------------------------- | ------------------------------------- | ---------------------------------------------------------------------------------------------------------------- | --- | --- | --- |
| [**create-tauri-app**](https://github.com/tauri-apps/create-tauri-app)          | 开始使用您的第一个 Tauri 应用程序     | [![](https://img.shields.io/npm/v/create-tauri-app.svg)](https://www.npmjs.com/package/create-tauri-app)         | ✅  | ✅  | ✅  |
| [**vue-cli-plugin-tauri**](https://github.com/tauri-apps/vue-cli-plugin-tauri/) | Tauri 的 Vue CLI 插件                | [![](https://img.shields.io/npm/v/vue-cli-plugin-tauri.svg)](https://www.npmjs.com/package/vue-cli-plugin-tauri) | ✅  | ✅  | ✅  |

## 介绍

Tauri 是一个框架，用于为所有主要桌面平台构建微小、快速的二进制文件。开发人员可以集成任何编译为 HTML、JS 和 CSS 的前端框架，以构建他们的用户界面。应用程序的后端是一个 rust 源二进制文件，其中包含前端可以与之交互的 API。

Tauri 应用程序中的用户界面目前利用 [`tao`](https://docs.rs/tao) 作为 macOS 和 Windows 上的窗口处理库， 并通过 **Tauri 团队**孵化和维护的 [WRY](https://github.com/tauri-apps/wry) 在Linux上使用 [`gtk`](https://gtk-rs.org/docs/gtk/), 它为系统Webview（以及其他好东西，如 Menu 和 Taskbar）创建了一个统一的界面，利用 MacOS 上的 WebKit，Windows 上的 WebView2 和 Linux 上的 WebKitGTK。

要了解有关所有这些部分如何组合在一起的详细信息，请参阅此 [ARCHITECTURE.md](https://github.com/tauri-apps/tauri/blob/dev/ARCHITECTURE.md) 文档.

## 快速开始

如果您有兴趣制作 Tauri 应用程序，请访问 [文档网站](https://tauri.app)。本自述文件面向那些有兴趣为核心库做出贡献的人。但是，如果您只想快速了解 `tauri` 开发阶段，这里有一个快速开始：

### 平台

Tauri目前支持在以下平台上进行开发和分发：

| 平台                      | 版本            |
| :----------------------- | :-------------- |
| Windows                  | 7 及以上         |
| macOS                    | 10.15 及以上     |
| Linux                    | 见下文           |
| iOS/iPadOS （即将推出）   |                 |
| Android （即将推出）      |                 |

**Linux 平台支持**

有关**开发** Tauri 应用程序的信息，请参阅 [tauri.app 入门指南](https://tauri.app/v1/guides/getting-started/prerequisites#setting-up-linux).

对于 **运行** Tauri 应用程序，我们支持以下配置（这些配置会自动添加为 .deb 依赖项，并捆绑到 AppImage 中，因此您的用户无需手动安装它们）：

- Debian (Ubuntu 18.04 及以上版本或同等版本) ，需要安装以下软件包：
  - `libwebkit2gtk-4.0-37`
  - `libgtk-3-0`
  - `libayatana-appindicator3-1`<sup>1</sup>
- Arch 需要安装以下软件包：
  - `webkit2gtk`
  - `gtk3`
  - `libayatana-appindicator`<sup>1</sup>
- Fedora (最新 2 个版本) 需要安装以下软件包：
  - `webkit2gtk3`
  - `gtk3`
  - `libappindicator-gtk3`<sup>1</sup>

说明：<sup>1</sup> 仅在使用系统托盘时才需要的软件包

### 特征功能

- [x] 桌面安装包 (.app, .dmg, .deb, AppImage, .msi)
- [x] 自我更新程序
- [x] 应用签名
- [x] 系统通知 (弹窗)
- [x] 应用托盘
- [x] 核心插件系统
- [x] 文件系统
- [x] Sidecar 模式

### 安全功能

- [x] 无本地主机 (:fire:)
- [x] 安全模式的自定义协议
- [x] 具有 tree-shaking 功能的动态提前编译 (dAoT)
- [x] 功能地址空间布局随机化
- [x] 在运行时对函数名称和消息进行一次性密码（OTP）混淆
- [x] CSP 注入

### 实用程序

- [x] 基于 Rust 的 CLI
- [x] Github Action 为所有平台创建二进制文件的操作
- [x] VS Code 插件

## 开发

Tauri 是一个由许多移动部件组成的系统：

### 基础设施

- 用于代码管理的 Git
- 用于项目管理的 GitHub
- 针对 CI 和 CD 的 GitHub 操作
- Discord 上交流和探讨
- Netlify 托管的文档网站
- DigitalOcean Meilisearch 应用

### 操作系统

Tauri 核心可以在 Mac、Linux 和 Windows 上开发，但建议您使用最新的操作系统并为您的操作系统构建工具。

### 贡献

在开始处理某些事情之前，最好先检查是否存在现有问题。在 Discord 服务器停下来并与团队确认它是否有意义或其他人是否已经在处理它也是一个好主意。

在提出拉取请求之前，请务必阅读 [贡献指南](./.github/CONTRIBUTING.md)。
Thank you to everyone contributing to Tauri!

### 文档

D多语言系统中的文档是一个棘手的命题。为此，我们更喜欢使用 Rust 代码的内联文档，并在 JSDoc 中使用 Typescript/JavaScript 代码。我们自动收集这些并使用 Docusaurus v2 和 netlify 发布它们。以下是文档站点的托管存储库： https://github.com/tauri-apps/tauri-docs

### 测试 & 交付

测试所有的东西！我们有许多测试套件，但一直在寻求提高我们的覆盖范围：

- Rust () => 通过内联声明获取 `cargo test` `#[cfg(test)]`
- Typescript  => 通过 `jest` 规范文件
- 冒烟测试 (在合并到最新版本时运行)
- eslint, clippy

### CI/CD

建议阅读本文，以更好地了解我们如何运行管道： https://www.jacobbolda.com/setting-up-ci-and-cd-for-tauri/

## 组织

Tauri 的目标是成为一个基于指导 [Tauri 的目标是成为一个基于指导](https://sfosc.org)的原则的可持续集体。为此，它已成为 [为此，它已成为](https://commonsconservancy.org/)内的一项计划，您可以通过 [开放集体](https://opencollective.com/tauri)进行经济贡献。

## 语义版本控制规范（SemVer）

**tauri** 遵循 [Semantic Versioning 2.0](https://semver.org/).

## Licenses

Code: (c) 2015 - 2021 - The Tauri Programme within The Commons Conservancy.

MIT or MIT/Apache 2.0 where applicable.

Logo: CC-BY-NC-ND

- Original Tauri Logo Designs by [Alve Larsson](https://alve.io/), [Daniel Thompson-Yvetot](https://github.com/nothingismagick) and [Guillaume Chau](https://github.com/akryum) 设计的原始Tauri标志

[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri?ref=badge_large)
