// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{app_paths::walk_builder, cargo, npm::PackageManager},
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
};
const JS_EXTENSIONS: &[&str] = &["js", "mjs", "jsx", "ts", "mts", "tsx"];

/// Returns a list of paths that could not be migrated
pub fn migrate(app_dir: &Path, tauri_dir: &Path) -> Result<()> {
  let mut new_npm_packages = Vec::new();
  let mut new_cargo_packages = Vec::new();

  let pm = PackageManager::from_project(app_dir)
    .into_iter()
    .next()
    .unwrap_or(PackageManager::Npm);

  for entry in walk_builder(app_dir).build().flatten() {
    if entry.file_type().map(|t| t.is_file()).unwrap_or_default() {
      let path = entry.path();
      let ext = path.extension().unwrap_or_default();
      if JS_EXTENSIONS.iter().any(|e| e == &ext) {
        let js_contents = std::fs::read_to_string(path)?;
        let new_contents = migrate_imports(
          path,
          &js_contents,
          &mut new_cargo_packages,
          &mut new_npm_packages,
        )?;
        if new_contents != js_contents {
          fs::write(path, new_contents)
            .with_context(|| format!("Error writing {}", path.display()))?;
        }
      }
    }
  }

  new_npm_packages.sort();
  new_npm_packages.dedup();
  if !new_npm_packages.is_empty() {
    pm.install(&new_npm_packages)
      .context("Error installing new npm packages")?;
  }

  new_cargo_packages.sort();
  new_cargo_packages.dedup();
  if !new_cargo_packages.is_empty() {
    cargo::install(&new_cargo_packages, Some(tauri_dir))
      .context("Error installing new Cargo packages")?;
  }

  Ok(())
}

fn migrate_imports<'a>(
  path: &'a Path,
  js_source: &'a str,
  new_cargo_packages: &mut Vec<String>,
  new_npm_packages: &mut Vec<String>,
) -> crate::Result<String> {
  let mut magic_js_source = MagicString::new(js_source);

  let source_type = SourceType::from_path(path).unwrap();
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

      // skip parsing non @tauri-apps/api imports
      if !module.starts_with("@tauri-apps/api") {
        continue;
      }

      // convert module to its pluginfied module or renamed one
      // import { ... } from "@tauri-apps/api/window" -> import { ... } from "@tauri-apps/api/webviewWindow"
      // import { ... } from "@tauri-apps/api/cli" -> import { ... } from "@tauri-apps/plugin-cli"
      if let Some(&module) = MODULES_MAP.get(module) {
        // +1 and -1, to skip modifying the import quotes
        magic_js_source
          .overwrite(
            stmt.source.span.start as i64 + 1,
            stmt.source.span.end as i64 - 1,
            module,
            Default::default(),
          )
          .map_err(|e| anyhow::anyhow!("{e}"))?;

        // if module was pluginified, add to packages
        let module = module.split_once("plugin-");
        if let Some((_, module)) = module {
          let js_plugin = format!("@tauri-apps/plugin-{module}");
          let cargo_crate = format!("tauri-plugin-{module}");
          new_npm_packages.push(js_plugin);
          new_cargo_packages.push(cargo_crate);
        }
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
            // import { getCurrent } from "@tauri-apps/api/webviewWindow"
            // const appWindow = getCurrent()
            // ```
            "appWindow" if module == "@tauri-apps/api/window" => {
              stmts_to_add.push("\nconst appWindow = getCurrent()");
              Some("getCurrent")
            }

            // migrate pluginigied moduls from:
            // ```
            // import { dialog, cli as superCli } from "@tauri-apps/api"
            // ```
            // to:
            // ```
            // import dialog from "@tauri-apps/plugin-dialog"
            // import cli as superCli from "@tauri-apps/plugin-cli"
            // ```
            import if PLUGINIFIED_MODULES.contains(&import) && module == "@tauri-apps/api" => {
              let js_plugin: &str = MODULES_MAP[&format!("@tauri-apps/api/{import}")];
              let (_, plugin_name) = js_plugin.split_once("plugin-").unwrap();
              let cargo_crate = format!("tauri-plugin-{plugin_name}");
              new_npm_packages.push(js_plugin.to_string());
              new_cargo_packages.push(cargo_crate);

              if specifier.local.name.as_str() != import {
                let local = &specifier.local.name;
                imports_to_add.push(format!("\nimport {import} as {local} from {js_plugin}"));
              } else {
                imports_to_add.push(format!("\nimport {import} from {js_plugin}"));
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
          // import { appWindow } from "@tauri-apps/api" -> import { getCurrent } from "@tauri-apps/api"
          if let Some(new_identifier) = new_identifier {
            magic_js_source
              .overwrite(
                specifier.span.start as _,
                specifier.span.end as _,
                new_identifier,
                Default::default(),
              )
              .map_err(|e| anyhow::anyhow!("{e}"))?;
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
              .remove(start as _, end as _)
              .map_err(|e| anyhow::anyhow!("{e}"))?;
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
        .append_right(start as _, &import)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    }
  }

  if !stmts_to_add.is_empty() {
    for stmt in stmts_to_add {
      magic_js_source
        .append_right(start as _, stmt)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    }
  }

  Ok(magic_js_source.to_string())
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn migrates() {
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
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
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
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
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
import { getCurrent } from "@tauri-apps/api/webviewWindow";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { register } from "@tauri-apps/plugin-global-shortcut";
import clipboard from "@tauri-apps/plugin-clipboard-manager";
import * as fs from "@tauri-apps/plugin-fs";
import "./App.css";
import dialog from @tauri-apps/plugin-dialog
import cli as superCli from @tauri-apps/plugin-cli
const appWindow = getCurrent()

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
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
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
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

    let mut new_cargo_packages = Vec::new();
    let mut new_npm_packages = Vec::new();

    let migrated = migrate_imports(
      Path::new("file.js"),
      input,
      &mut new_cargo_packages,
      &mut new_npm_packages,
    )
    .unwrap();

    assert_eq!(migrated, expected);

    assert_eq!(
      new_cargo_packages,
      vec![
        "tauri-plugin-dialog",
        "tauri-plugin-cli",
        "tauri-plugin-dialog",
        "tauri-plugin-global-shortcut",
        "tauri-plugin-clipboard-manager",
        "tauri-plugin-fs"
      ]
    );

    assert_eq!(
      new_npm_packages,
      vec![
        "@tauri-apps/plugin-dialog",
        "@tauri-apps/plugin-cli",
        "@tauri-apps/plugin-dialog",
        "@tauri-apps/plugin-global-shortcut",
        "@tauri-apps/plugin-clipboard-manager",
        "@tauri-apps/plugin-fs"
      ]
    );
  }
}
