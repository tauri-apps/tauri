// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{app_paths::walk_builder, npm::PackageManager},
  Result,
};
use anyhow::Context;
use itertools::Itertools;
use magic_string::MagicString;
use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_parser::Parser;
use oxc_span::SourceType;

use std::{fs, path::Path};

mod partial_loader;

const RENAMED_MODULES: phf::Map<&str, &str> = phf::phf_map! {
  "tauri" => "core",
  "window" => "webviewWindow"
};
const PLUGINIFIED_MODULES: [&str; 11] = [
  "cli",
  "clipboard",
  "dialog",
  "fs",
  "globalShortcut",
  "http",
  "notification",
  "os",
  "process",
  "shell",
  "updater",
];
// (from, to)
const MODULES_MAP: phf::Map<&str, &str> = phf::phf_map! {
  // renamed
  "@tauri-apps/api/tauri" => "@tauri-apps/api/core",
  "@tauri-apps/api/window" => "@tauri-apps/api/webviewWindow",
  // pluginified
  "@tauri-apps/api/cli" => "@tauri-apps/plugin-cli",
  "@tauri-apps/api/clipboard" => "@tauri-apps/plugin-clipboard-manager",
  "@tauri-apps/api/dialog" => "@tauri-apps/plugin-dialog",
  "@tauri-apps/api/fs" => "@tauri-apps/plugin-fs",
  "@tauri-apps/api/globalShortcut" => "@tauri-apps/plugin-global-shortcut",
  "@tauri-apps/api/http" => "@tauri-apps/plugin-http",
  "@tauri-apps/api/notification" => "@tauri-apps/plugin-notification",
  "@tauri-apps/api/os" => "@tauri-apps/plugin-os",
  "@tauri-apps/api/process" => "@tauri-apps/plugin-process",
  "@tauri-apps/api/shell" => "@tauri-apps/plugin-shell",
  "@tauri-apps/api/updater" => "@tauri-apps/plugin-updater",
  // v1 plugins to v2
  "tauri-plugin-sql-api" => "@tauri-apps/plugin-sql",
  "tauri-plugin-store-api" => "@tauri-apps/plugin-store",
  "tauri-plugin-upload-api" => "@tauri-apps/plugin-upload",
  "tauri-plugin-fs-extra-api" => "@tauri-apps/plugin-fs",
  "tauri-plugin-fs-watch-api" => "@tauri-apps/plugin-fs",
  "tauri-plugin-autostart-api" => "@tauri-apps/plugin-autostart",
  "tauri-plugin-websocket-api" => "@tauri-apps/plugin-websocket",
  "tauri-plugin-positioner-api" => "@tauri-apps/plugin-positioner",
  "tauri-plugin-stronghold-api" => "@tauri-apps/plugin-stronghold",
  "tauri-plugin-window-state-api" => "@tauri-apps/plugin-window-state",
  "tauri-plugin-authenticator-api" => "@tauri-apps/plugin-authenticator",
};
const JS_EXTENSIONS: &[&str] = &["js", "mjs", "jsx", "ts", "mts", "tsx", "svelte", "vue"];

/// Returns a list of migrated plugins
pub fn migrate(frontend_dir: &Path) -> Result<Vec<String>> {
  let mut new_npm_packages = Vec::new();
  let mut new_plugins = Vec::new();
  let mut npm_packages_to_remove = Vec::new();

  let pre = env!("CARGO_PKG_VERSION_PRE");
  let npm_version = if pre.is_empty() {
    format!("{}.0.0", env!("CARGO_PKG_VERSION_MAJOR"))
  } else {
    format!(
      "{}.0.0-{}.0",
      env!("CARGO_PKG_VERSION_MAJOR"),
      pre.split('.').next().unwrap()
    )
  };

  let pm = PackageManager::from_project(frontend_dir);

  for pkg in ["@tauri-apps/cli", "@tauri-apps/api"] {
    let version = pm
      .current_package_version(pkg, frontend_dir)
      .unwrap_or_default()
      .unwrap_or_default();
    if version.starts_with('1') {
      new_npm_packages.push(format!("{pkg}@^{npm_version}"));
    }
  }

  for entry in walk_builder(frontend_dir).build().flatten() {
    if entry.file_type().map(|t| t.is_file()).unwrap_or_default() {
      let path = entry.path();
      let ext = path.extension().unwrap_or_default();
      if JS_EXTENSIONS.iter().any(|e| e == &ext) {
        let js_contents = std::fs::read_to_string(path)?;
        let new_contents = migrate_imports(
          path,
          &js_contents,
          &mut new_plugins,
          &mut npm_packages_to_remove,
        )?;
        if new_contents != js_contents {
          fs::write(path, new_contents)
            .with_context(|| format!("Error writing {}", path.display()))?;
        }
      }
    }
  }

  if !npm_packages_to_remove.is_empty() {
    npm_packages_to_remove.sort();
    npm_packages_to_remove.dedup();
    pm.remove(&npm_packages_to_remove, frontend_dir)
      .context("Error removing npm packages")?;
  }

  if !new_npm_packages.is_empty() {
    new_npm_packages.sort();
    new_npm_packages.dedup();
    pm.install(&new_npm_packages, frontend_dir)
      .context("Error installing new npm packages")?;
  }

  Ok(new_plugins)
}

fn migrate_imports<'a>(
  path: &'a Path,
  js_source: &'a str,
  new_plugins: &mut Vec<String>,
  npm_packages_to_remove: &mut Vec<String>,
) -> crate::Result<String> {
  let mut magic_js_source = MagicString::new(js_source);

  let has_partial_js = path
    .extension()
    .is_some_and(|ext| ext == "vue" || ext == "svelte");

  let sources = if !has_partial_js {
    vec![(SourceType::from_path(path).unwrap(), js_source, 0i64)]
  } else {
    partial_loader::PartialLoader::parse(
      path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default(),
      js_source,
    )
    .unwrap()
    .into_iter()
    .map(|s| (s.source_type, s.source_text, s.start as i64))
    .collect()
  };

  for (source_type, js_source, script_start) in sources {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, js_source, source_type).parse();
    if !ret.errors.is_empty() {
      anyhow::bail!(
        "failed to parse {} as valid Javascript/Typescript file",
        path.display()
      )
    }

    let mut program = ret.program;

    let mut stmts_to_add = Vec::new();
    let mut imports_to_add = Vec::new();

    for import in program.body.iter_mut() {
      if let Statement::ImportDeclaration(stmt) = import {
        let module = stmt.source.value.as_str();

        // convert module to its pluginfied module or renamed one
        // import { ... } from "@tauri-apps/api/window" -> import { ... } from "@tauri-apps/api/webviewWindow"
        // import { ... } from "@tauri-apps/api/cli" -> import { ... } from "@tauri-apps/plugin-cli"
        if let Some(&new_module) = MODULES_MAP.get(module) {
          // +1 and -1, to skip modifying the import quotes
          magic_js_source
            .overwrite(
              script_start + stmt.source.span.start as i64 + 1,
              script_start + stmt.source.span.end as i64 - 1,
              new_module,
              Default::default(),
            )
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("failed to replace import source")?;

          // if module was pluginified, add to packages
          if let Some(plugin_name) = new_module.strip_prefix("@tauri-apps/plugin-") {
            new_plugins.push(plugin_name.to_string());
          }

          // if the module is a v1 plugin, we should remove it
          if module.starts_with("tauri-plugin-") {
            npm_packages_to_remove.push(module.to_string());
          }
        }

        // skip parsing non @tauri-apps/api imports
        if !module.starts_with("@tauri-apps/api") {
          continue;
        }

        let Some(specifiers) = &mut stmt.specifiers else {
          continue;
        };

        for specifier in specifiers.iter() {
          if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier {
            let new_identifier = match specifier.imported.name().as_str() {
              // migrate appWindow from:
              // ```
              // import { appWindow } from "@tauri-apps/api/window"
              // ```
              // to:
              // ```
              // import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow"
              // const appWindow = getCurrentWebviewWindow()
              // ```
              "appWindow" if module == "@tauri-apps/api/window" => {
                stmts_to_add.push("\nconst appWindow = getCurrentWebviewWindow()");
                Some("getCurrentWebviewWindow")
              }

              // migrate pluginified modules from:
              // ```
              // import { dialog, cli as superCli } from "@tauri-apps/api"
              // ```
              // to:
              // ```
              // import * as dialog from "@tauri-apps/plugin-dialog"
              // import * as cli as superCli from "@tauri-apps/plugin-cli"
              // ```
              import if PLUGINIFIED_MODULES.contains(&import) && module == "@tauri-apps/api" => {
                let js_plugin: &str = MODULES_MAP[&format!("@tauri-apps/api/{import}")];
                let (_, plugin_name) = js_plugin.split_once("plugin-").unwrap();

                new_plugins.push(plugin_name.to_string());

                if specifier.local.name.as_str() != import {
                  let local = &specifier.local.name;
                  imports_to_add.push(format!(
                    "\nimport * as {import} as {local} from \"{js_plugin}\""
                  ));
                } else {
                  imports_to_add.push(format!("\nimport * as {import} from \"{js_plugin}\""));
                };
                None
              }

              import if module == "@tauri-apps/api" => match RENAMED_MODULES.get(import) {
                Some(m) => Some(*m),
                None => continue,
              },

              // nothing to do, go to next specifier
              _ => continue,
            };

            // if identifier was renamed, it will be Some()
            // and so we convert the import
            // import { appWindow } from "@tauri-apps/api/window" -> import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow"
            if let Some(new_identifier) = new_identifier {
              magic_js_source
                .overwrite(
                  script_start + specifier.span.start as i64,
                  script_start + specifier.span.end as i64,
                  new_identifier,
                  Default::default(),
                )
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("failed to rename identifier")?;
            } else {
              // if None, we need to remove this specifier,
              // it will also be replaced with an import from its new plugin below

              // find the next comma or the bracket ending the import
              let start = specifier.span.start as usize;
              let sliced = &js_source[start..];
              let comma_or_bracket = sliced.chars().find_position(|&c| c == ',' || c == '}');
              let end = match comma_or_bracket {
                Some((n, ',')) => n + start + 1,
                Some((_, '}')) => specifier.span.end as _,
                _ => continue,
              };

              magic_js_source
                .remove(script_start + start as i64, script_start + end as i64)
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("failed to remove identifier")?;
            }
          }
        }
      }
    }

    // find the end of import list
    // fallback to the program start
    let start = program
      .body
      .iter()
      .rev()
      .find(|s| matches!(s, Statement::ImportDeclaration(_)))
      .map(|s| match s {
        Statement::ImportDeclaration(s) => s.span.end,
        _ => unreachable!(),
      })
      .unwrap_or(program.span.start);

    if !imports_to_add.is_empty() {
      for import in imports_to_add {
        magic_js_source
          .append_right(script_start as u32 + start, &import)
          .map_err(|e| anyhow::anyhow!("{e}"))
          .context("failed to add import")?;
      }
    }

    if !stmts_to_add.is_empty() {
      for stmt in stmts_to_add {
        magic_js_source
          .append_right(script_start as u32 + start, stmt)
          .map_err(|e| anyhow::anyhow!("{e}"))
          .context("failed to add statement")?;
      }
    }
  }

  Ok(magic_js_source.to_string())
}

#[cfg(test)]
mod tests {

  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn migrates_vue() {
    let input = r#"
<template>
    <div>Tauri!</div>
</template>

<script setup>
  import { useState } from "react";
  import reactLogo from "./assets/react.svg";
  import { invoke, dialog, cli as superCli } from "@tauri-apps/api";
  import { appWindow } from "@tauri-apps/api/window";
  import { convertFileSrc } from "@tauri-apps/api/tauri";
  import { open } from "@tauri-apps/api/dialog";
  import { register } from "@tauri-apps/api/globalShortcut";
  import clipboard from "@tauri-apps/api/clipboard";
  import * as fs from "@tauri-apps/api/fs";
  import "./App.css";
</script>

<style>
.greeting {
  color: red;
  font-weight: bold;
}
</style>
"#;

    let expected = r#"
<template>
    <div>Tauri!</div>
</template>

<script setup>
  import { useState } from "react";
  import reactLogo from "./assets/react.svg";
  import { invoke,   } from "@tauri-apps/api";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { register } from "@tauri-apps/plugin-global-shortcut";
  import clipboard from "@tauri-apps/plugin-clipboard-manager";
  import * as fs from "@tauri-apps/plugin-fs";
  import "./App.css";
import * as dialog from "@tauri-apps/plugin-dialog"
import * as cli as superCli from "@tauri-apps/plugin-cli"
const appWindow = getCurrentWebviewWindow()
</script>

<style>
.greeting {
  color: red;
  font-weight: bold;
}
</style>
"#;

    let mut new_plugins = Vec::new();
    let mut npm_packages_to_remove = Vec::new();

    let migrated = migrate_imports(
      Path::new("file.vue"),
      input,
      &mut new_plugins,
      &mut npm_packages_to_remove,
    )
    .unwrap();

    assert_eq!(migrated, expected);

    assert_eq!(
      new_plugins,
      vec![
        "dialog",
        "cli",
        "dialog",
        "global-shortcut",
        "clipboard-manager",
        "fs"
      ]
    );
    assert_eq!(npm_packages_to_remove, Vec::<String>::new());
  }

  #[test]
  fn migrates_svelte() {
    let input = r#"
<form>
</form>

<script>
  import { useState } from "react";
  import reactLogo from "./assets/react.svg";
  import { invoke, dialog, cli as superCli } from "@tauri-apps/api";
  import { appWindow } from "@tauri-apps/api/window";
  import { convertFileSrc } from "@tauri-apps/api/tauri";
  import { open } from "@tauri-apps/api/dialog";
  import { register } from "@tauri-apps/api/globalShortcut";
  import clipboard from "@tauri-apps/api/clipboard";
  import * as fs from "@tauri-apps/api/fs";
  import "./App.css";
</script>
"#;

    let expected = r#"
<form>
</form>

<script>
  import { useState } from "react";
  import reactLogo from "./assets/react.svg";
  import { invoke,   } from "@tauri-apps/api";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { register } from "@tauri-apps/plugin-global-shortcut";
  import clipboard from "@tauri-apps/plugin-clipboard-manager";
  import * as fs from "@tauri-apps/plugin-fs";
  import "./App.css";
import * as dialog from "@tauri-apps/plugin-dialog"
import * as cli as superCli from "@tauri-apps/plugin-cli"
const appWindow = getCurrentWebviewWindow()
</script>
"#;

    let mut new_plugins = Vec::new();
    let mut npm_packages_to_remove = Vec::new();

    let migrated = migrate_imports(
      Path::new("file.svelte"),
      input,
      &mut new_plugins,
      &mut npm_packages_to_remove,
    )
    .unwrap();

    assert_eq!(migrated, expected);

    assert_eq!(
      new_plugins,
      vec![
        "dialog",
        "cli",
        "dialog",
        "global-shortcut",
        "clipboard-manager",
        "fs"
      ]
    );
    assert_eq!(npm_packages_to_remove, Vec::<String>::new());
  }

  #[test]
  fn migrates_js() {
    let input = r#"
import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke, dialog, cli as superCli } from "@tauri-apps/api";
import { appWindow } from "@tauri-apps/api/window";
import { convertFileSrc } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog";
import { register } from "@tauri-apps/api/globalShortcut";
import clipboard from "@tauri-apps/api/clipboard";
import * as fs from "@tauri-apps/api/fs";
import { Store } from "tauri-plugin-store-api";
import Database from "tauri-plugin-sql-api";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://v2.tauri.app/develop/calling-rust/#commands
    setGreetMsg(await invoke("greet", { name }));
    await open();
    await dialog.save();
    await convertFileSrc("");
    const a = appWindow.label;
    superCli.getMatches();
    clipboard.readText();
    fs.exists("");
  }

  return (
    <div className="container">
      <h1>Welcome to Tauri!</h1>

      <div className="row">
        <a href="https://vite.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>

      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>

      <p>{greetMsg}</p>
    </div>
  );
}

export default App;
"#;

    let expected = r#"
import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke,   } from "@tauri-apps/api";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { register } from "@tauri-apps/plugin-global-shortcut";
import clipboard from "@tauri-apps/plugin-clipboard-manager";
import * as fs from "@tauri-apps/plugin-fs";
import { Store } from "@tauri-apps/plugin-store";
import Database from "@tauri-apps/plugin-sql";
import "./App.css";
import * as dialog from "@tauri-apps/plugin-dialog"
import * as cli as superCli from "@tauri-apps/plugin-cli"
const appWindow = getCurrentWebviewWindow()

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://v2.tauri.app/develop/calling-rust/#commands
    setGreetMsg(await invoke("greet", { name }));
    await open();
    await dialog.save();
    await convertFileSrc("");
    const a = appWindow.label;
    superCli.getMatches();
    clipboard.readText();
    fs.exists("");
  }

  return (
    <div className="container">
      <h1>Welcome to Tauri!</h1>

      <div className="row">
        <a href="https://vite.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>

      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>

      <p>{greetMsg}</p>
    </div>
  );
}

export default App;
"#;

    let mut new_plugins = Vec::new();
    let mut npm_packages_to_remove = Vec::new();

    let migrated = migrate_imports(
      Path::new("file.js"),
      input,
      &mut new_plugins,
      &mut npm_packages_to_remove,
    )
    .unwrap();

    assert_eq!(migrated, expected);

    assert_eq!(
      new_plugins,
      vec![
        "dialog",
        "cli",
        "dialog",
        "global-shortcut",
        "clipboard-manager",
        "fs",
        "store",
        "sql"
      ]
    );
    assert_eq!(
      npm_packages_to_remove,
      vec!["tauri-plugin-store-api", "tauri-plugin-sql-api"]
    );
  }
}
