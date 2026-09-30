// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{get_app, Target};
use crate::{
  helpers::app_paths::Dirs,
  helpers::{config::get_config as get_tauri_config, template::JsonMap},
  interface::AppInterface,
  ConfigValue, Result,
};
use cargo_mobile2::{
  config::app::App,
  reserved_names::KOTLIN_ONLY_KEYWORDS,
  util::{
    self,
    cli::{Report, TextWrapper},
  },
};
use handlebars::{
  Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError, RenderErrorReason,
};

use std::{env::var_os, path::PathBuf};

pub fn command(
  target: Target,
  ci: bool,
  reinstall_deps: bool,
  skip_targets_install: bool,
  config: Vec<ConfigValue>,
) -> Result<()> {
  let dirs = crate::helpers::app_paths::resolve_dirs();
  let wrapper = TextWrapper::default();

  exec(
    target,
    &wrapper,
    ci,
    reinstall_deps,
    skip_targets_install,
    config,
    dirs,
  )?;
  Ok(())
}

fn exec(
  target: Target,
  wrapper: &TextWrapper,
  #[allow(unused_variables)] non_interactive: bool,
  #[allow(unused_variables)] reinstall_deps: bool,
  skip_targets_install: bool,
  config: Vec<ConfigValue>,
  dirs: Dirs,
) -> Result<App> {
  let tauri_config = get_tauri_config(
    target.platform_target(),
    &config.iter().map(|conf| &conf.0).collect::<Vec<_>>(),
    dirs.tauri,
  )?;

  let app = get_app(
    target,
    &tauri_config,
    &AppInterface::new(&tauri_config, None, dirs.tauri)?,
    dirs.tauri,
  );

  let (handlebars, mut map) = handlebars(&app);

  let mut args = std::env::args_os();

  let (binary, mut build_args) = args
    .next()
    .map(|bin| {
      let bin_path = PathBuf::from(&bin);
      let mut build_args = vec!["tauri"];

      if let Some(bin_stem) = bin_path.file_stem() {
        let r = regex::Regex::new("(nodejs|node)\\-?([1-9]*)*$").unwrap();
        if r.is_match(&bin_stem.to_string_lossy()) {
          if var_os("PNPM_PACKAGE_NAME").is_some() {
            return ("pnpm".into(), build_args);
          } else if is_pnpm_dlx() {
            return ("pnpm".into(), vec!["dlx", "@tauri-apps/cli"]);
          } else if let Some(npm_execpath) = var_os("npm_execpath") {
            let manager_stem = PathBuf::from(&npm_execpath)
              .file_stem()
              .unwrap()
              .to_os_string();
            let is_npm = manager_stem == "npm-cli";
            let binary = if is_npm {
              "npm".into()
            } else if manager_stem == "npx-cli" {
              "npx".into()
            } else {
              manager_stem
            };

            if is_npm {
              build_args.insert(0, "run");
              build_args.insert(1, "--");
            }

            return (binary, build_args);
          }
        } else if bin_stem == "deno" {
          build_args.insert(0, "task");
          return (std::ffi::OsString::from("deno"), build_args);
        } else if !cfg!(debug_assertions) && bin_stem == "cargo-tauri" {
          return (std::ffi::OsString::from("cargo"), build_args);
        }
      }

      (bin, build_args)
    })
    .unwrap_or_else(|| (std::ffi::OsString::from("cargo"), vec!["tauri"]));

  build_args.push(target.command_name());
  build_args.push(target.ide_build_script_name());

  let mut binary = binary.to_string_lossy().to_string();
  if binary.ends_with(".exe") || binary.ends_with(".cmd") || binary.ends_with(".bat") {
    // remove Windows-only extension
    binary.pop();
    binary.pop();
    binary.pop();
    binary.pop();
  }

  // TODO: temp `\ -> /` conversion for ohos
  map.insert(
    "tauri-binary",
    dunce::simplified(std::path::Path::new(&binary))
      .to_string_lossy()
      .replace('\\', "/"),
  );
  map.insert("tauri-binary-args", &build_args);
  map.insert("tauri-binary-args-str", build_args.join(" "));

  let app = match target {
    // Generate Android Studio project
    Target::Android => {
      let _env = super::android::env(non_interactive)?;
      let (config, metadata) =
        super::android::get_config(&app, &tauri_config, &[], &Default::default());
      map.insert("android", &config);

      // Add application_id_suffix to the map for template access
      // The template will access it via a helper or we'll modify template to use root context
      if let Some(suffix) = &tauri_config.bundle.android.debug_application_id_suffix {
        map.insert("android-debug-application-id-suffix", suffix);
      }

      super::android::project::gen(
        &config,
        &metadata,
        (handlebars, map),
        wrapper,
        skip_targets_install,
      )?;
      app
    }
    #[cfg(target_os = "macos")]
    // Generate Xcode project
    Target::Ios => {
      let (config, metadata) =
        super::ios::get_config(&app, &tauri_config, &[], &Default::default(), dirs.tauri)?;
      map.insert("apple", &config);
      super::ios::project::gen(
        &tauri_config,
        &config,
        &metadata,
        (handlebars, map),
        wrapper,
        non_interactive,
        reinstall_deps,
        skip_targets_install,
      )?;
      app
    }
    Target::OpenHarmony => {
      let (config, _metadata) =
        super::open_harmony::get_config(&app, &tauri_config, &[], &Default::default());
      super::open_harmony::project::gen(&app, &config, (handlebars, map), skip_targets_install)?;
      app
    }
  };

  Report::victory(
    "Project generated successfully!",
    "Make cool apps! 🌻 🐕 🎉",
  )
  .print(wrapper);
  Ok(app)
}

fn handlebars(app: &App) -> (Handlebars<'static>, JsonMap) {
  let mut h = Handlebars::new();
  h.register_escape_fn(handlebars::no_escape);

  h.register_helper("html-escape", Box::new(html_escape));
  h.register_helper("join", Box::new(join));
  h.register_helper("quote-and-join", Box::new(quote_and_join));
  h.register_helper(
    "quote-and-join-colon-prefix",
    Box::new(quote_and_join_colon_prefix),
  );
  h.register_helper("snake-case", Box::new(snake_case));
  h.register_helper("escape-kotlin-keyword", Box::new(escape_kotlin_keyword));
  // don't mix these up or very bad things will happen to all of us
  h.register_helper("prefix-path", Box::new(prefix_path));
  h.register_helper("unprefix-path", Box::new(unprefix_path));

  let mut map = JsonMap::default();
  map.insert("app", app);

  (h, map)
}

fn get_str<'a>(helper: &'a Helper) -> &'a str {
  helper
    .param(0)
    .and_then(|v| v.value().as_str())
    .unwrap_or("")
}

fn get_str_array(helper: &Helper, formatter: impl Fn(&str) -> String) -> Option<Vec<String>> {
  helper.param(0).and_then(|v| {
    v.value()
      .as_array()
      .and_then(|arr| arr.iter().map(|val| val.as_str().map(&formatter)).collect())
  })
}

fn html_escape(
  helper: &Helper,
  _: &Handlebars,
  _ctx: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(&handlebars::html_escape(get_str(helper)))
    .map_err(Into::into)
}

fn join(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(
      &get_str_array(helper, |s| s.to_string())
        .ok_or_else(|| {
          RenderErrorReason::ParamTypeMismatchForName("join", "0".to_owned(), "array".to_owned())
        })?
        .join(", "),
    )
    .map_err(Into::into)
}

fn quote_and_join(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(
      &get_str_array(helper, |s| format!("{s:?}"))
        .ok_or_else(|| {
          RenderErrorReason::ParamTypeMismatchForName(
            "quote-and-join",
            "0".to_owned(),
            "array".to_owned(),
          )
        })?
        .join(", "),
    )
    .map_err(Into::into)
}

fn quote_and_join_colon_prefix(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(
      &get_str_array(helper, |s| format!("{:?}", format!(":{s}")))
        .ok_or_else(|| {
          RenderErrorReason::ParamTypeMismatchForName(
            "quote-and-join-colon-prefix",
            "0".to_owned(),
            "array".to_owned(),
          )
        })?
        .join(", "),
    )
    .map_err(Into::into)
}

fn snake_case(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  use heck::ToSnekCase as _;
  out
    .write(&get_str(helper).to_snek_case())
    .map_err(Into::into)
}

fn escape_kotlin_keyword(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  let escaped_result = get_str(helper)
    .split('.')
    .map(|s| {
      if KOTLIN_ONLY_KEYWORDS.contains(&s) {
        format!("`{s}`")
      } else {
        s.to_string()
      }
    })
    .collect::<Vec<_>>()
    .join(".");

  out.write(&escaped_result).map_err(Into::into)
}

fn app_root(ctx: &Context) -> std::result::Result<&str, RenderError> {
  let app_root = ctx
    .data()
    .get("app")
    .ok_or_else(|| RenderErrorReason::Other("`app` missing from template data.".to_owned()))?
    .get("root-dir")
    .ok_or_else(|| {
      RenderErrorReason::Other("`app.root-dir` missing from template data.".to_owned())
    })?;
  app_root.as_str().ok_or_else(|| {
    RenderErrorReason::Other("`app.root-dir` contained invalid UTF-8.".to_owned()).into()
  })
}

fn prefix_path(
  helper: &Helper,
  _: &Handlebars,
  ctx: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(
      util::prefix_path(app_root(ctx)?, get_str(helper))
        .to_str()
        .ok_or_else(|| {
          RenderErrorReason::Other(
            "Either the `app.root-dir` or the specified path contained invalid UTF-8.".to_owned(),
          )
        })?,
    )
    .map_err(Into::into)
}

fn unprefix_path(
  helper: &Helper,
  _: &Handlebars,
  ctx: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(
      util::unprefix_path(app_root(ctx)?, get_str(helper))
        .map_err(|_| {
          RenderErrorReason::Other(
            "Attempted to unprefix a path that wasn't in the app root dir.".to_owned(),
          )
        })?
        .to_str()
        .ok_or_else(|| {
          RenderErrorReason::Other(
            "Either the `app.root-dir` or the specified path contained invalid UTF-8.".to_owned(),
          )
        })?,
    )
    .map_err(Into::into)
}

fn is_pnpm_dlx() -> bool {
  var_os("NODE_PATH")
    .map(PathBuf::from)
    .is_some_and(|node_path| {
      let mut iter = node_path.components().peekable();
      while let Some(c) = iter.next() {
        if c.as_os_str() == "pnpm" && iter.peek().is_some_and(|c| c.as_os_str() == "dlx") {
          return true;
        }
      }
      false
    })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::mobile::open_harmony::project;
  use cargo_mobile2::{config::app::Raw, open_harmony::config::Config as OpenHarmonyConfig};
  use clap::Parser;
  use serde_json::Value;
  use std::{fs, path::Path};

  fn generate_open_harmony(root: &Path) -> Result<PathBuf> {
    let app = App::from_raw(
      root.to_path_buf(),
      Raw {
        name: "reader".into(),
        lib_name: Some("reader_lib".into()),
        stylized_name: Some("Reader & 阅读".into()),
        identifier: "com.example.reader".into(),
        asset_dir: None,
        template_pack: None,
      },
    )
    .unwrap();
    let config = OpenHarmonyConfig::from_raw(app.clone(), None).unwrap();
    let (handlebars, mut map) = handlebars(&app);
    map.insert(
      "tauri-binary-args",
      ["tauri", "ohos", "dev-eco-studio-script"],
    );
    project::gen(&app, &config, (handlebars, map), true)?;
    Ok(config.project_dir())
  }

  fn read_json(path: impl AsRef<Path>) -> Value {
    json5::from_str(&fs::read_to_string(path).unwrap()).unwrap()
  }

  #[test]
  fn open_harmony_launcher_uses_app_name_and_generated_icon() {
    let root = tempfile::tempdir().unwrap();
    let project = generate_open_harmony(root.path()).unwrap();
    let app = read_json(project.join("AppScope/app.json5"));
    let module = read_json(project.join("entry/src/main/module.json5"));
    let ability = &module["module"]["abilities"][0];
    assert_eq!(app["app"]["label"], ability["label"]);
    let name = ability["label"]
      .as_str()
      .unwrap()
      .strip_prefix("$string:")
      .unwrap();
    let strings = read_json(project.join("AppScope/resources/base/element/string.json"));
    let name_resource = strings["string"]
      .as_array()
      .unwrap()
      .iter()
      .find(|resource| resource["name"] == name)
      .unwrap();
    assert_eq!(name_resource["value"], "Reader & 阅读");

    let resource = project.join("AppScope/resources/base/media/app_icon.png");
    // A fresh project has a valid default even before running `tauri icon`.
    image::open(&resource).unwrap();
    for reference in [
      &app["app"]["icon"],
      &ability["icon"],
      &ability["startWindowIcon"],
    ] {
      assert_eq!(reference, "$media:app_icon");
    }

    let source = root.path().join("source.png");
    image::RgbaImage::from_pixel(1024, 1024, image::Rgba([255, 0, 0, 255]))
      .save(&source)
      .unwrap();
    let output = root.path().join("icons");
    let options = crate::icon::Options::try_parse_from([
      std::ffi::OsStr::new("icon"),
      source.as_os_str(),
      std::ffi::OsStr::new("--output"),
      output.as_os_str(),
    ])
    .unwrap();
    crate::icon::command(options).unwrap();
    let icon = image::open(&resource).unwrap().to_rgba8();
    assert_eq!(icon.dimensions(), (512, 512));
    assert_eq!(icon.get_pixel(256, 256).0, [255, 0, 0, 255]);
    let generated = fs::read(&resource).unwrap();
    generate_open_harmony(root.path()).unwrap();
    assert_eq!(fs::read(resource).unwrap(), generated);
  }
}
