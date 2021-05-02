---
title: Make your own CLI
---

import Alert from '@theme/Alert'

Tauri enables your app to have a CLI through <a href="https://github.com/clap-rs/clap" target="_blank">clap</a>, a robust command line argument parser. With a simple CLI definition in your `tauri.conf.json` file, you can define your interface and read its argument matches map on JavaScript and/or Rust.

## Base Configuration

Under `tauri.conf.json`, you have the following structure to configure the interface:

```js title=src-tauri/tauri.conf.json
{
  "tauri": {
    "cli": {
      "description": "", // command description that's shown on help
      "longDescription": "", // command long description that's shown on help
      "beforeHelp": "", // content to show before the help text
      "afterHelp": "", // content to show after the help text
      "args": [], // list of arguments of the command, we'll explain it later
      "subcommands": {
        "subcommand-name": {
          // configures a subcommand that is accessible
          // with `$ ./app subcommand-name --arg1 --arg2 --etc`
          // configuration as above, with "description", "args", etc.
        }
      }
    }
  }
}
```

<Alert title="Note">
  All JSON configurations here are just samples, many other fields have been omitted for the sake of clarity.
</Alert>

## Adding Arguments

The `args` array represents the list of arguments accepted by its command or subcommand. You can find more details about the way to configure them <a href="/docs/api/config#tauri">here</a>.

### Positional Arguments

A positional argument is identified by its position in the list of arguments. With the following configuration:

```json title=src-tauri/tauri.conf.json:tauri.cli
{
  "args": [
    {
      "name": "source",
      "index": 1
    },
    {
      "name": "destination",
      "index": 2
    }
  ]
}
```

Users can run your app as `$ ./app tauri.txt dest.txt` and the arg matches map will define `source` as `"tauri.txt"` and `destination` as `"dest.txt"`.

### Named Arguments

A named argument is a (key, value) pair where the key identifies the value. With the following configuration:

```json title=src-tauri/tauri.conf.json:tauri.cli
{
  "args": [
    {
      "name": "type",
      "short": "t",
      "takesValue": true,
      "multiple": true,
      "possibleValues": ["foo", "bar"]
    }
  ]
}
```

Users can run your app as `$ ./app --type foo bar`, `$ ./app -t foo -t bar` or `$ ./app --type=foo,bar` and the arg matches map will define `type` as `["foo", "bar"]`.

### Flag Arguments

A flag argument is a standalone key whose presence or absence provides information to your application. With the following configuration:

```js title=src-tauri/tauri.conf.json:tauri.cli
{
  "args": [
    "name": "verbose",
    "short": "v",
    "multipleOccurrences": true
  ]
}
```

Users can run your app as `$ ./app -v -v -v`, `$ ./app --verbose --verbose --verbose` or `$ ./app -vvv` and the arg matches map will define `verbose` as `true`, with `occurrences = 3`.

## Subcommands

Some CLI applications has additional interfaces as subcommands. For instance, the `git` CLI has `git branch`, `git commit` and `git push`. You can define additional nested interfaces with the `subcommands` array:

```js title=src-tauri/tauri.conf.json:tauri
{
  "cli": {
    ...
    "subcommands": {
      "branch": {
        "args": []
      },
      "push": {
        "args": []
      }
    }
  }
}
```

Its configuration is the same as the root application configuration, with the `description`, `longDescription`, `args`, etc.

## Reading the matches

### Rust

```rust
use tauri::cli::get_matches;

fn main() {
  match get_matches() {
    Some(matches) => {
      // `matches` here is a Struct with { args, subcommand }
      // where args is the HashMap mapping each arg's name to it's { value, occurrences }
      // and subcommand is an Option of { name, matches }
    }
  }
}
```

### JavaScript

```js
import { getMatches } from '@tauri-apps/api/cli'

getMatches().then((matches) => {
  // do something with the { args, subcommand } matches
})
```

## Complete documentation

You can find more about the CLI configuration <a href="/docs/api/config#tauri">here</a>.
