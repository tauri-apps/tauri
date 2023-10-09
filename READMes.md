<img src=".github/splash.png" alt="Tauri" />

[![status](https://img.shields.io/badge/status-stable-blue.svg)](https://github.com/tauri-apps/tauri/tree/dev)
[![License](https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg)](https://opencollective.com/tauri)
[![test core](https://img.shields.io/github/actions/workflow/status/tauri-apps/tauri/test-core.yml?label=test%20core&logo=github)](https://github.com/tauri-apps/tauri/actions/workflows/test-core.yml)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri?ref=badge_shield)
[![Chat Server](https://img.shields.io/badge/chat-discord-7289da.svg)](https://discord.gg/SpmNs4S)
[![website](https://img.shields.io/badge/website-tauri.app-purple.svg)](https://tauri.app)
[![https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg](https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg)](https://good-labs.github.io/greater-good-affirmation)
[![support](https://img.shields.io/badge/sponsor-Open%20Collective-blue.svg)](https://opencollective.com/tauri)

## Versiones Actuales

### Núcleo

| Componente                                                                                   | Descripción                                 | Versión                                                                                                  | Lin | Win | Mac |
| ------------------------------------------------------------------------------------------- | ------------------------------------------- | -------------------------------------------------------------------------------------------------------- | --- | --- | --- |
| [**tauri**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri)                        | núcleo de tiempo de ejecución               | [![](https://img.shields.io/crates/v/tauri.svg)](https://crates.io/crates/tauri)                         | ✅  | ✅  | ✅  |
| [**tauri-build**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-build)            | aplica macros en tiempo de compilación     | [![](https://img.shields.io/crates/v/tauri-build.svg)](https://crates.io/crates/tauri-build)             | ✅  | ✅  | ✅  |
| [**tauri-codegen**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-codegen)        | maneja activos, analiza tauri.conf.json     | [![](https://img.shields.io/crates/v/tauri-codegen.svg)](https://crates.io/crates/tauri-codegen)         | ✅  | ✅  | ✅  |
| [**tauri-macros**](https://ithub.com/tauri-apps/tauri/tree/dev/core/tauri-macros)           | crea macros utilizando tauri-codegen        | [![](https://img.shields.io/crates/v/tauri-macros.svg)](https://crates.io/crates/tauri-macros)           | ✅  | ✅  | ✅  |
| [**tauri-runtime**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-runtime)        | capa entre Tauri y las bibliotecas webview  | [![](https://img.shields.io/crates/v/tauri-runtime.svg)](https://crates.io/crates/tauri-runtime)         | ✅  | ✅  | ✅  |
| [**tauri-runtime-wry**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-runtime-wry) | permite la interacción a nivel de sistema via WRY | [![](https://img.shields.io/crates/v/tauri-runtime-wry.svg)](https://crates.io/crates/tauri-runtime-wry) | ✅  | ✅  | ✅  |
| [**tauri-utils**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-utils)             | código común utilizado en todos los paquetes de tauri | [![](https://img.shields.io/crates/v/tauri-utils.svg)](https://crates.io/crates/tauri-utils)             | ✅  | ✅  | ✅  |

### Herramientas

| Componente                                                                             | Descripción                               | Versión                                                                                                | Lin | Win | Mac |
| ------------------------------------------------------------------------------------- | ----------------------------------------- | ------------------------------------------------------------------------------------------------------ | --- | --- | --- |
| [**bundler**](https://github.com/tauri-apps/tauri/tree/dev/tooling/bundler)           | fabrica los binarios finales               | [![](https://img.shields.io/crates/v/tauri-bundler.svg)](https://crates.io/crates/tauri-bundler)       | ✅  | ✅  | ✅  |
| [**tauri-cli**](https://github.com/tauri-apps/tauri/tree/dev/tooling/cli)             | crea, desarrolla y compila aplicaciones    | [![](https://img.shields.io/crates/v/tauri-cli.svg)](https://crates.io/crates/tauri-cli)               | ✅  | ✅  | ✅  |
| [**@tauri-apps/cli**](https://github.com/tauri-apps/tauri/tree/dev/tooling/cli/node)  | Envoltorio de CLI de Node.js para `tauri-cli` | [![](https://img.shields.io/npm/v/@tauri-apps/cli.svg)](https://www.npmjs.com/package/@tauri-apps/cli) | ✅  | ✅  | ✅  |
| [**@tauri-apps/api**](https://github.com/tauri-apps/tauri/tree/dev/tooling/api)      | API de JavaScript para interactuar con el backend de Rust | [![](https://img.shields.io/npm/v/@tauri-apps/api.svg)](https://www.npmjs.com/package/@tauri-apps/api) | ✅  | ✅  | ✅  |

### Utilidades y Complementos

| Componente                                                                            | Descripción                             | Versión                                                                                                          | Lin | Win | Mac |
| ------------------------------------------------------------------------------------ | --------------------------------------- | ---------------------------------------------------------------------------------------------------------------- | --- | --- | --- |
| [**create-tauri-app**](https://github.com/tauri-apps/create-tauri-app)          | Comienza con tu primera aplicación Tauri | [![](https://img.shields.io/npm/v/create-tauri-app.svg)](https://www.npmjs.com/package/create-tauri-app)         | ✅  | ✅  | ✅  |
| [**vue-cli-plugin-tauri**](https://github.com/tauri-apps/vue-cli-plugin-tauri/) | Complemento de Vue CLI para Tauri       | [![](https://img.shields.io/npm/v/vue-cli-plugin-tauri.svg)](https://www.npmjs.com/package/vue-cli-plugin-tauri) | ✅  | ✅  | ✅  |

## Introducción

Tauri es un marco para construir binarios pequeños y extremadamente rápidos para todas las principales plataformas de escritorio. Los desarrolladores pueden integrar cualquier marco de frontend que se compile a HTML, JS y CSS para construir su interfaz de usuario. El backend de la aplicación es un binario de origen Rust con una API con la que el frontend puede interactuar.

La interfaz de usuario en las aplicaciones de Tauri actualmente aprovecha [`tao`](https://docs.rs/tao) como una biblioteca de manejo de ventanas en macOS y Windows, y [`gtk`](https://gtk-rs.org/docs/gtk/) en Linux a través de [WRY](https://github.com/tauri-apps/wry), incubado y mantenido por el equipo de Tauri, que crea una interfaz unificada para el webview del sistema (y otras características como el menú y la barra de tareas), aprovechando WebKit en macOS, WebView2 en Windows y WebKitGTK en Linux.

Para obtener más información sobre cómo encajan todas estas piezas, consulta el documento [ARCHITECTURE.md](https://github.com/tauri-apps/tauri/blob/dev/ARCHITECTURE.md).

## Empezar

Si estás interesado en crear una aplicación Tauri, visita el [sitio web de documentación](https://tauri.app). Este README está dirigido a aquellos que están interesados en contribuir a la biblioteca principal. Pero si solo quieres obtener una visión general rápida de dónde se encuentra `tauri` en su desarrollo, aquí tienes una descripción rápida:

### Plataformas

Tauri actualmente admite el desarrollo y la distribución en las siguientes plataformas:

| Plataforma                | Versiones        |
| :-----------------------  | :-------------- |
| Windows                   | 7 en adelante    |
| macOS                     | 10.15 en adelante |
| Linux                     | Ver más abajo    |
| iOS/iPadOS (próximamente) |                 |
| Android (próximamente)    |                 |

**Soporte para Linux**

Para **desarrollar** aplicaciones Tauri, consulta la [Guía de inicio en tauri.app](https://tauri.app/v1/guides/getting-started/prerequisites#setting-up-linux).

Para **ejecutar** aplicaciones Tauri, admitimos las siguientes configuraciones (estas se agregan automáticamente como dependencias para .deb y se incluyen en AppImage para que tus usuarios no tengan que instalarlos manualmente):

- Debian (Ubuntu 18.04 y posteriores o equivalentes) con los siguientes paquetes instalados:
  - `libwebkit2gtk-4.1-0`, `libgtk-3-0`, `libayatana-appindicator3-1`<sup>1</sup>
- Arch con los siguientes paquetes instalados:
  - `webkit2gtk`, `gtk3`, `libayatana-appindicator`<sup>1</sup>
- Fedora (las 2 versiones más recientes) con los siguientes paquetes instalados:
  - `webkit2gtk3`, `gtk3`, `libappindicator-gtk3`<sup>1</sup>
- Void con los siguientes paquetes instalados:
  - `webkit2gtk`, `gtk+3`, `libappindicator`<sup>1</sup>

<sup>1</sup> `appindicator` solo es necesario si se utilizan bandejas del sistema.

### Características

- [x] Empaquetador de escritorio (.app, .dmg, .deb, AppImage, .msi)
- [x] Actualizador automático
- [x] Firma de la aplicación
- [x] Notificaciones nativas (toast)
- [x] Bandeja de la aplicación
- [x] Sistema de complementos principal
- [x] Sistema de sistema de archivos con alcance
- [x] Sidecar

### Características de Seguridad

- [x] Sin localhost (:fire:)
- [x] Protocolo personalizado para el modo seguro
- [x] Compilación dinámica anticipada (dAoT) con eliminación funcional de árboles
- [x] Disposición funcional aleatoria del espacio de direcciones
- [x] Salado OTP de nombres de funciones y mensajes en tiempo de ejecución
- [x] Inyección de CSP

### Utilidades

- [x] CLI basada en Rust
- [x] Acción de GH para crear binarios para todas las plataformas
- [x] Extensión de VS Code

## Desarrollo

Tauri es un sistema compuesto por varias piezas en movimiento:

### Infraestructura

- Git para la gestión de código
- GitHub para la gestión de proyectos
- Acciones de GitHub para CI y CD
- Discord para discusiones
- Sitio web de documentación alojado en Netlify
- Instancia de Meilisearch de DigitalOcean

### Sistemas operativos

Tauri core se puede desarrollar en Mac, Linux y Windows, pero se recomienda utilizar los sistemas operativos y herramientas de construcción más recientes posibles para tu sistema operativo.

### Contribuciones

Antes de comenzar a trabajar en algo, es mejor verificar si ya existe un problema existente. También es una buena idea pasar por el servidor de Discord y confirmar con el equipo si tiene sentido o si alguien más ya está trabajando en ello.

Asegúrate de leer la [Guía de Contribución](./.github/CONTRIBUTING.md) antes de enviar una solicitud de extracción.

¡Gracias a todos los que contribuyen a Tauri!

### Documentación

La documentación en un sistema políglota es una propuesta complicada. Con este fin, preferimos utilizar documentación en línea del código Rust y JSDoc en el código TypeScript / JavaScript. Recopilamos automáticamente esto y lo publicamos usando Docusaurus v2 y Netlify. Aquí está el repositorio de alojamiento del sitio de documentación: https://github.com/tauri-apps/tauri-docs

### Pruebas y Linting

¡Prueba todas las cosas! Tenemos varias suites de pruebas, pero siempre estamos buscando mejorar nuestra cobertura:

- Rust (`cargo test`) => obtenido a través de declaraciones `#[cfg(test)]` en línea
- TypeScript (`jest`) => mediante archivos de especificación
- Pruebas de Humo (se ejecutan en fusiones a la última versión)
- eslint, clippy

### CI/CD

Recomendamos que leas este artículo para entender mejor cómo ejecutamos nuestras canalizaciones: https://www.jacobbolda.com/setting-up-ci-and-cd-for-tauri/

## Organización

Tauri tiene como objetivo ser un colectivo sostenible basado en principios que guían a [comunidades de software libre y abierto sostenibles](https://sfosc.org). Con este fin, se ha convertido en un Programa dentro de [Commons Conservancy](https://commonsconservancy.org/), y puedes contribuir financieramente a través de [Open Collective](https://opencollective.com/tauri).

## Semver

**tauri** sigue la [Versión Semántica 2.0](https://semver.org/).

## Licencias

Código: (c) 2015 - 2021 - El Programa Tauri dentro de Commons Conservancy.

MIT o MIT/Apache 2.0 cuando corresponda.

Logo: CC-BY-NC-ND

- Diseños originales del logotipo de Tauri por [Alve Larsson](https://alve.io/), [Daniel Thompson-Yvetot](https://github.com/nothingismagick) y [Guillaume Chau](https://github.com/akryum)

[![Estado FOSSA](https://app.fossa.com/api/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri?ref=badge_large)
