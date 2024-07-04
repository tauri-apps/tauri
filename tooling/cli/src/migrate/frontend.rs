// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{app_paths::walk_builder, cargo, npm::PackageManager},
  Result,
};
use anyhow::Context;
use oxc_allocator::{Allocator, Box};
use oxc_ast::ast::*;
use oxc_codegen::*;
use oxc_parser::Parser;
use oxc_span::Span;
use oxc_span::{Atom, SourceType};

use std::{
  collections::{HashMap, HashSet},
  fs,
  path::Path,
};

const RENAMED_MODULES: [(&str, &str); 4] = [
  ("tauri", "core"),
  ("window", "webviewWindow"),
  ("clipboard", "clipboard-manager"),
  ("globalShortcut", "global-shortcut"),
];
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
const MODULES_MAP: [(&str, &str); 13] = [
  // renamed
  ("@tauri-apps/api/tauri", "@tauri-apps/api/core"),
  ("@tauri-apps/api/window", "@tauri-apps/api/webviewWindow"),
  // pluginified
  ("@tauri-apps/api/cli", "@tauri-apps/plugin-cli"),
  (
    "@tauri-apps/api/clipboard",
    "@tauri-apps/plugin-clipboard-manager",
  ),
  ("@tauri-apps/api/dialog", "@tauri-apps/plugin-dialog"),
  ("@tauri-apps/api/fs", "@tauri-apps/plugin-fs"),
  (
    "@tauri-apps/api/globalShortcut",
    "@tauri-apps/plugin-global-shortcut",
  ),
  ("@tauri-apps/api/http", "@tauri-apps/plugin-http"),
  (
    "@tauri-apps/api/notification",
    "@tauri-apps/plugin-notification",
  ),
  ("@tauri-apps/api/os", "@tauri-apps/plugin-os"),
  ("@tauri-apps/api/process", "@tauri-apps/plugin-process"),
  ("@tauri-apps/api/shell", "@tauri-apps/plugin-shell"),
  ("@tauri-apps/api/updater", "@tauri-apps/plugin-updater"),
];
const JS_EXTENSIONS: &[&str] = &["js", "mjs", "jsx", "ts", "mts", "tsx"];

/// Returns a list of paths that could not be migrated
pub fn migrate(app_dir: &Path, tauri_dir: &Path) -> Result<()> {
  let mut new_npm_packages = HashSet::new();
  let mut new_cargo_packages = HashSet::new();

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

  let new_npm_packages = new_npm_packages.into_iter().collect::<Vec<_>>();
  if !new_npm_packages.is_empty() {
    pm.install(&new_npm_packages)
      .context("Error installing new npm packages")?;
  }

  let new_cargo_packages = new_cargo_packages.into_iter().collect::<Vec<_>>();
  if !new_cargo_packages.is_empty() {
    cargo::install(&new_cargo_packages, Some(tauri_dir))
      .context("Error installing new Cargo packages")?;
  }

  Ok(())
}

fn migrate_imports<'a>(
  path: &'a Path,
  js_source: &'a str,
  new_cargo_packages: &mut HashSet<String>,
  new_npm_packages: &mut HashSet<String>,
) -> crate::Result<String> {
  let modules_map: HashMap<&str, &str> = MODULES_MAP.into_iter().collect();
  let renamed_modules: HashMap<&str, &str> = RENAMED_MODULES.into_iter().collect();

  let source_type = SourceType::from_path(path).unwrap();
  let allocator = Allocator::default();
  let ret = Parser::new(&allocator, js_source, source_type).parse();
  if !ret.errors.is_empty() {
    anyhow::bail!(
      "failed to parse {} as valid Javascript/Typescript file",
      path.display()
    )
  }

  let mut imports_to_add = Vec::new();
  let mut program = ret.program;

  for import in program.body.iter_mut() {
    if let Statement::ImportDeclaration(stmt) = import {
      let module = stmt.source.value.as_str();

      // skip parsing non @tauri-apps/api imports
      if !module.starts_with("@tauri-apps/api") {
        continue;
      }

      // convert module to its pluginfied module or renamed one
      // import {} from "@tauri-apps/api/window" -> import {} from "@tauri-apps/api/webviewWindow"
      // import {} from "@tauri-apps/api/cli" -> import {} from "@tauri-apps/plugin-cli"
      if let Some(&module) = modules_map.get(module) {
        stmt.source = StringLiteral::new(Span::empty(0), Atom::from(module));

        // if module was pluginified, add to packages
        let module = module.split_once("plugin-");
        if let Some((_, module)) = module {
          let js_plugin = format!("@tauri-apps/plugin-{module}");
          let cargo_crate = format!("tauri-plugin-{module}");
          new_npm_packages.insert(js_plugin);
          new_cargo_packages.insert(cargo_crate);
        }
      }

      let Some(specifiers) = &mut stmt.specifiers else {
        continue;
      };

      // Some modules have been pluginified
      // and so we will need to remove their imports
      // if they were imported from `@tauri-apps/api` root export.
      //
      // we store indexes here and remove later to staisfy borrow rules.
      let mut indexes_to_remove = Vec::new();

      for (index, specifier) in specifiers.iter_mut().enumerate() {
        if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier {
          let new_identifier = match specifier.imported.name().as_str() {
            "appWindow" if module == "@tauri-apps/api/window" => Some("getCurrent"),

            import if PLUGINIFIED_MODULES.contains(&import) => {
              let plugin_name = renamed_modules.get(import).unwrap_or(&import);
              let js_plugin = format!("@tauri-apps/plugin-{plugin_name}");
              let cargo_crate = format!("tauri-plugin-{plugin_name}");
              new_npm_packages.insert(js_plugin.clone());
              new_cargo_packages.insert(cargo_crate);

              let new_import = if specifier.local.name.as_str() != import {
                specifier.local.name.as_str()
              } else {
                import
              };
              imports_to_add.push((js_plugin, new_import, specifier.import_kind));
              None
            }

            import if module == "@tauri-apps/api" => match renamed_modules.get(import) {
              Some(m) => Some(*m),
              None => continue,
            },

            // nothing to do, go to next specifier
            _ => continue,
          };

          // if identifier was renamed, it will be Some()
          // and so we convert the import
          // import { appWindow } from "@tauri-apps/api"  -> import { getCurrent as appWindow } from "@tauri-apps/api"
          if let Some(new_identifier) = new_identifier {
            let new_identifier_atom = Atom::from(new_identifier);
            let new_identifier = IdentifierName::new(Span::empty(0), new_identifier_atom.clone());
            specifier.imported = ModuleExportName::IdentifierName(new_identifier);
            specifier.local = specifier.local.clone();
          } else {
            // if None, we need to remove this specified and replace it
            // with an import from its new plugin
            indexes_to_remove.push(index);
          }
        }
      }

      // sort and remove elements in reverse order
      // otherwise, the vector will shift and remaining indexes
      // will be invalid
      indexes_to_remove.sort();
      for index in indexes_to_remove.into_iter().rev() {
        specifiers.remove(index);
      }
    }
  }

  // add new imports if needed
  for (source, import, import_kind) in imports_to_add.iter() {
    program.body.push(Statement::ImportDeclaration(Box::new_in(
      ImportDeclaration {
        span: Span::empty(0),
        import_kind: *import_kind,
        with_clause: None,
        source: StringLiteral::new(Span::empty(0), Atom::from(source.as_str())),
        specifiers: Some(oxc_allocator::Vec::from_iter_in(
          [ImportDeclarationSpecifier::ImportNamespaceSpecifier(
            Box::new_in(
              ImportNamespaceSpecifier {
                span: Span::empty(0),
                local: BindingIdentifier::new(Span::empty(0), Atom::from(*import)),
              },
              &allocator,
            ),
          )],
          &allocator,
        )),
      },
      &allocator,
    )));
  }

  Ok(CodeGenerator::new().build(&program).source_text)
}

#[cfg(test)]
mod tests {

  use std::collections::HashSet;

  use super::*;

  #[test]
  fn migrates() {
    let input = r#"
import { open } from '@tauri-apps/api/dialog'
import { read } from '@tauri-apps/api/clipboard'
import { register } from '@tauri-apps/api/globalShortcut'
import { appWindow } from '@tauri-apps/api/window'
import { convertFileSrc as convertProtocol, invoke } from '@tauri-apps/api/tauri'
import { event, invoke } from '@tauri-apps/api'
import { dialog as newDialog, clipboard, app } from '@tauri-apps/api'
import tauriApi from '@tauri-apps/api'
import noEvent from '@tauri-apps/api/event'
import { qwe } from '@tauri-apps/api/cli';
import * as noApp from '@tauri-apps/api/app'
import {
  window as noWindow,
  app,
  event,
  tauri,
} from '@tauri-apps/api'
"#;

    let output = r#"import { open } from '@tauri-apps/plugin-dialog';
import { read } from '@tauri-apps/plugin-clipboard-manager';
import { register } from '@tauri-apps/plugin-global-shortcut';
import { getCurrent as appWindow } from '@tauri-apps/api/webviewWindow';
import { convertFileSrc as convertProtocol, invoke } from '@tauri-apps/api/core';
import { event, invoke } from '@tauri-apps/api';
import { app } from '@tauri-apps/api';
import tauriApi from '@tauri-apps/api';
import noEvent from '@tauri-apps/api/event';
import { qwe } from '@tauri-apps/plugin-cli';
import * as noApp from '@tauri-apps/api/app';
import { webviewWindow as noWindow, app, event, core as tauri } from '@tauri-apps/api';
import * as newDialog from '@tauri-apps/plugin-dialog';
import * as clipboard from '@tauri-apps/plugin-clipboard-manager';
"#;

    let mut new_cargo_packages = HashSet::new();
    let mut new_npm_packages = HashSet::new();

    let migrated = migrate_imports(
      Path::new("file.js"),
      input,
      &mut new_cargo_packages,
      &mut new_npm_packages,
    )
    .unwrap();

    assert_eq!(migrated, output);

    assert_eq!(
      new_cargo_packages.iter().collect::<Vec<_>>(),
      vec![
        "tauri-plugin-cli",
        "tauri-plugin-dialog",
        "tauri-plugin-clipboard-manager",
        "tauri-plugin-global-shortcut",
      ]
    );

    assert_eq!(
      new_npm_packages.iter().collect::<Vec<_>>(),
      vec![
        "@tauri-apps/plugin-dialog",
        "@tauri-apps/plugin-cli",
        "@tauri-apps/plugin-global-shortcut",
        "@tauri-apps/plugin-clipboard-manager",
      ]
    );
  }
}
