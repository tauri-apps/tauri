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
Initializes a Tauri project

USAGE:
    cargo tauri init [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
        --ci         Skip prompting for values
    -f, --force      Force init to overwrite the src-tauri folder
    -h, --help       Print help information
    -l, --log        Enables logging
    -V, --version    Print version information

OPTIONS:
    -A, --app-name <app-name>            Name of your Tauri application
    -d, --directory <directory>          Set target directory for init
    -D, --dist-dir <dist-dir>            Web assets location, relative to <project-dir>/src-tauri
    -P, --dev-path <dev-path>            Url of your dev server
    -t, --tauri-path <tauri-path>        Path of the Tauri project to use (relative to the cwd)
    -W, --window-title <window-title>    Window title of your Tauri application

SUBCOMMANDS:
    help      Print this message or the help of the given subcommand(s)
    plugin    Initialize a Tauri plugin.
```

### `init plugin`

<Command name="init plugin" />

```
Initializes a Tauri plugin project.

USAGE:
    cargo tauri init plugin [FLAGS] [OPTIONS] --name <name>

FLAGS:
    -a, --api        Initializes a Tauri plugin with TypeScript API.
    -h, --help       Print help information
    -V, --version    Print version information

OPTIONS:
    -d, --directory <directory>      Set target directory for init
    -n, --name <name>                Name of your Tauri plugin
    -t, --tauri-path <tauri-path>    Path of the Tauri project to use (relative to the cwd)
```

## `dev`

<Command name="dev" />

```
Tauri dev.

USAGE:
    cargo tauri dev [FLAGS] [OPTIONS] [--] [args]...

ARGS:
    <args>...    Args passed to the binary

FLAGS:
    -e, --exit-on-panic    Exit on panic
    -h, --help             Print help information
        --release          Run the code in release mode
    -V, --version          Print version information

OPTIONS:
    -c, --config <config>           config JSON to merge with tauri.conf.json
    -f, --features <features>...    list of cargo features to activate
    -r, --runner <runner>           binary to use to run the application
    -t, --target <target>...        target triple to build against
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
Tauri build.

USAGE:
    cargo tauri build [FLAGS] [OPTIONS]

FLAGS:
    -d, --debug      Builds with the debug flag
    -h, --help       Print help information
    -v, --verbose    Enables verbose logging
    -V, --version    Print version information

OPTIONS:
    -b, --bundle <bundle>...        list of bundles to package
    -c, --config <config>           config JSON to merge with tauri.conf.json
    -f, --features <features>...    list of cargo features to activate
    -r, --runner <runner>           binary to use to build the application
    -t, --target <target>...        target triple to build against
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
    $ tauri icon /path/to/icon.png

  Options
    --help, -h           Displays this message
    --log, -l            Logging [boolean]
    --target, -t         Target folder (default: 'src-tauri/icons')
    --compression, -c    Compression type [optipng|zopfli]
    --ci                 Runs the script in CI mode     
```

This command will generate a set of icons, based on the source icon you've entered. Note that the source icon must be 1240x1240 with transparency.

## `version`

<Command name="--version" />

```
  Description
    Returns the current version of tauri
```

This command will show the current version of Tauri.

## CLI usage

See more about the usage through this [complete guide](/docs/development/integration).
