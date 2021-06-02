---
id: cli
title: CLI
---

import Command from '@theme/Command'
import Alert from '@theme/Alert'


The tauri.js cli is composed in TypeScript and published as JavaScript. 

## `info`

<Command name="info" />

```
  Description
    Returns the known state of tauri dependencies and configuration
```

It shows a concise list of information about the environment, Rust, Node.js and their versions as well as some relevant configurations.

<Alert title="Note" icon="info-alt">
This command is pretty helpful when you need to have a quick overview of your application. When requesting some help, it can be useful that you share this report with us.
</Alert>

## `init`

<Command name="init" />

```
  Description
    Inits the Tauri template. If Tauri cannot find the src-tauri/tauri.conf.json
    it will create one.
  Usage
    $ tauri init
  Options
    --help, -h        Displays this message
    --force, -f       Force init to overwrite [conf|template|all]
    --log, -l         Logging [boolean]
    --directory, -d   Set target directory for init
    --tauriPath, -t   Path of the Tauri project to use (relative to the cwd)
```

## `dev`

<Command name="dev" />

```
  Description
    Tauri dev.
  Usage
    $ tauri dev
  Options
    --help, -h     Displays this message
```

This command will open the WebView in development mode. It makes use of the `build.devPath` property from your `src-tauri/tauri.conf.json` file.

If you have entered a command to the `build.beforeDevCommand` property, this one will be executed before the `dev` command.

<a href="/docs/api/config#build">See more about the configuration.</a><br/><br/>

<Alert title="Troubleshooting" type="warning" icon="alert">

If you're not using `build.beforeDevCommand`, make sure your `build.devPath` is correct and, if using a development server, that it's started before using this command.
</Alert>

## `deps`

<Command name="deps update" />

```sh
  Description
    Tauri dependency management script
  Usage
    $ tauri deps [install|update]
```


## `build`

<Command name="build" />

```
  Description
    Tauri build.
  Usage
    $ tauri build
  Options
    --help, -h     Displays this message
    --debug, -d    Build a tauri app with debugging
```

This command will bundle your application, either in production mode or debug mode if you used the `--debug` flag. It makes use of the `build.distDir` property from your `src-tauri/tauri.conf.json` file.

If you have entered a command to the `build.beforeBuildCommand` property, this one will be executed before the `build` command.

<a href="/docs/api/config#build">See more about the configuration.</a>

## `icon`

<Command name="icon" />

```
  Description
    Create all the icons you need for your Tauri app.

  Usage
    $ tauri icon

  Options
    --help, -h          Displays this message
    --log, -l            Logging [boolean]
    --icon, -i           Source icon (png, 1240x1240 with transparency)
    --target, -t         Target folder (default: 'src-tauri/icons')
    --compression, -c    Compression type [pngquant|optipng|zopfli]
```

This command will generate a set of icons, based on the source icon you've entered.

## `version`

<Command name="--version" />

```
  Description
    Returns the current version of tauri
```

This command will show the current version of Tauri.

## CLI usage

See more about the usage through this [complete guide](/docs/usage/development/integration).

