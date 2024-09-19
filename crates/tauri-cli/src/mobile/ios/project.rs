// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{config::Config as TauriConfig, template},
  mobile::ios::LIB_OUTPUT_FILE_NAME,
  Result,
};
use anyhow::Context;
use cargo_mobile2::{
  apple::{
    config::{Config, Metadata},
    deps, rust_version_check,
    target::Target,
  },
  config::app::DEFAULT_ASSET_DIR,
  target::TargetTrait as _,
  util::{self, cli::TextWrapper},
};
use handlebars::Handlebars;
use include_dir::{include_dir, Dir};
use std::{
  ffi::OsString,
  fs::{create_dir_all, OpenOptions},
  path::{Component, PathBuf},
};

const TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/mobile/ios");

// unprefixed app_root seems pretty dangerous!!
// TODO: figure out what cargo-mobile meant by that
#[allow(clippy::too_many_arguments)]
pub fn gen(
  tauri_config: &TauriConfig,
  config: &Config,
  metadata: &Metadata,
  (handlebars, mut map): (Handlebars, template::JsonMap),
  wrapper: &TextWrapper,
  non_interactive: bool,
  reinstall_deps: bool,
  skip_targets_install: bool,
) -> Result<()> {
  if !skip_targets_install {
    let installed_targets =
      crate::interface::rust::installation::installed_targets().unwrap_or_default();
    let missing_targets = Target::all()
      .values()
      .filter(|t| !installed_targets.contains(&t.triple().into()))
      .collect::<Vec<&Target>>();

    if !missing_targets.is_empty() {
      println!("Installing iOS Rust toolchains...");
      for target in missing_targets {
        target
          .install()
          .context("failed to install target with rustup")?;
      }
    }
  }

  rust_version_check(wrapper)?;

  deps::install_all(wrapper, non_interactive, true, reinstall_deps)
    .with_context(|| "failed to install Apple dependencies")?;

  let dest = config.project_dir();
  let rel_prefix = util::relativize_path(config.app().root_dir(), &dest);
  let source_dirs = vec![rel_prefix.join("src")];

  let asset_catalogs = metadata.ios().asset_catalogs().unwrap_or_default();
  let ios_pods = metadata.ios().pods().unwrap_or_default();
  let macos_pods = metadata.macos().pods().unwrap_or_default();

  #[cfg(target_arch = "aarch64")]
  let default_archs = ["arm64", "arm64-sim"];
  #[cfg(not(target_arch = "aarch64"))]
  let default_archs = ["arm64", "x86_64"];

  map.insert("lib-output-file-name", LIB_OUTPUT_FILE_NAME);

  map.insert("file-groups", &source_dirs);
  map.insert("ios-frameworks", metadata.ios().frameworks());
  map.insert("ios-valid-archs", default_archs);
  map.insert("ios-vendor-frameworks", metadata.ios().vendor_frameworks());
  map.insert("ios-vendor-sdks", metadata.ios().vendor_sdks());
  map.insert("macos-frameworks", metadata.macos().frameworks());
  map.insert(
    "macos-vendor-frameworks",
    metadata.macos().vendor_frameworks(),
  );
  map.insert("macos-vendor-sdks", metadata.macos().vendor_frameworks());
  map.insert("asset-catalogs", asset_catalogs);
  map.insert("ios-pods", ios_pods);
  map.insert("macos-pods", macos_pods);
  map.insert(
    "ios-additional-targets",
    metadata.ios().additional_targets(),
  );
  map.insert(
    "macos-additional-targets",
    metadata.macos().additional_targets(),
  );
  map.insert("ios-pre-build-scripts", metadata.ios().pre_build_scripts());
  map.insert(
    "ios-post-compile-scripts",
    metadata.ios().post_compile_scripts(),
  );
  map.insert(
    "ios-post-build-scripts",
    metadata.ios().post_build_scripts(),
  );
  map.insert(
    "macos-pre-build-scripts",
    metadata.macos().pre_build_scripts(),
  );
  map.insert(
    "macos-post-compile-scripts",
    metadata.macos().post_compile_scripts(),
  );
  map.insert(
    "macos-post-build-scripts",
    metadata.macos().post_build_scripts(),
  );
  map.insert(
    "ios-command-line-arguments",
    metadata.ios().command_line_arguments(),
  );
  map.insert(
    "macos-command-line-arguments",
    metadata.macos().command_line_arguments(),
  );

  let mut created_dirs = Vec::new();
  template::render_with_generator(
    &handlebars,
    map.inner(),
    &TEMPLATE_DIR,
    &dest,
    &mut |path| {
      let mut components: Vec<_> = path.components().collect();
      let mut new_component = None;
      for component in &mut components {
        if let Component::Normal(c) = component {
          let c = c.to_string_lossy();
          if c.contains("{{app.name}}") {
            new_component.replace(OsString::from(
              &c.replace("{{app.name}}", config.app().name()),
            ));
            *component = Component::Normal(new_component.as_ref().unwrap());
            break;
          }
        }
      }
      let path = dest.join(components.iter().collect::<PathBuf>());

      let parent = path.parent().unwrap().to_path_buf();
      if !created_dirs.contains(&parent) {
        create_dir_all(&parent)?;
        created_dirs.push(parent);
      }

      let mut options = OpenOptions::new();
      options.write(true);

      if !path.exists() {
        options.create(true).open(path).map(Some)
      } else {
        Ok(None)
      }
    },
  )
  .with_context(|| "failed to process template")?;

  if let Some(template_path) = tauri_config.bundle.ios.template.as_ref() {
    let template = std::fs::read_to_string(template_path)
      .context("failed to read custom Xcode project template")?;
    let mut output_file = std::fs::File::create(dest.join("project.yml"))?;
    handlebars
      .render_template_to_write(&template, map.inner(), &mut output_file)
      .expect("Failed to render template");
  }

  let mut dirs_to_create = asset_catalogs.to_vec();
  dirs_to_create.push(dest.join(DEFAULT_ASSET_DIR));
  dirs_to_create.push(dest.join("Externals"));
  dirs_to_create.push(dest.join(format!("{}_iOS", config.app().name())));

  // Create all required project directories if they don't already exist
  for dir in &dirs_to_create {
    std::fs::create_dir_all(dir).map_err(|cause| {
      anyhow::anyhow!(
        "failed to create directory at {path}: {cause}",
        path = dir.display()
      )
    })?;
  }

  // Note that Xcode doesn't always reload the project nicely; reopening is
  // often necessary.
  println!("Generating Xcode project...");
  duct::cmd(
    "xcodegen",
    [
      "generate",
      "--spec",
      &dest.join("project.yml").to_string_lossy(),
    ],
  )
  .stdout_file(os_pipe::dup_stdout().unwrap())
  .stderr_file(os_pipe::dup_stderr().unwrap())
  .run()
  .with_context(|| "failed to run `xcodegen`")?;

  if !ios_pods.is_empty() || !macos_pods.is_empty() {
    duct::cmd(
      "pod",
      [
        "install",
        &format!("--project-directory={}", dest.display()),
      ],
    )
    .stdout_file(os_pipe::dup_stdout().unwrap())
    .stderr_file(os_pipe::dup_stderr().unwrap())
    .run()
    .with_context(|| "failed to run `pod install`")?;
  }
  Ok(())
}
