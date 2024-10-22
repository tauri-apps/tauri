// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{get_app, Target};
use crate::{
  helpers::{config::get as get_tauri_config, template::JsonMap},
  interface::{AppInterface, Interface},
  Result,
};
use cargo_mobile2::{
  android::env::Env as AndroidEnv,
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
) -> Result<()> {
  let wrapper = TextWrapper::default();

  exec(target, &wrapper, ci, reinstall_deps, skip_targets_install)
    .map_err(|e| anyhow::anyhow!("{:#}", e))?;
  Ok(())
}

pub fn exec(
  target: Target,
  wrapper: &TextWrapper,
  #[allow(unused_variables)] non_interactive: bool,
  #[allow(unused_variables)] reinstall_deps: bool,
  skip_targets_install: bool,
) -> Result<App> {
  let tauri_config = get_tauri_config(target.platform_target(), None)?;

  let tauri_config_guard = tauri_config.lock().unwrap();
  let tauri_config_ = tauri_config_guard.as_ref().unwrap();

  let app = get_app(
    target,
    tauri_config_,
    &AppInterface::new(tauri_config_, None)?,
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

  map.insert("tauri-binary", binary.to_string_lossy());
  map.insert("tauri-binary-args", &build_args);
  map.insert("tauri-binary-args-str", build_args.join(" "));

  let app = match target {
    // Generate Android Studio project
    Target::Android => match AndroidEnv::new() {
      Ok(_env) => {
        let (config, metadata) =
          super::android::get_config(&app, tauri_config_, None, &Default::default());
        map.insert("android", &config);
        super::android::project::gen(
          &config,
          &metadata,
          (handlebars, map),
          wrapper,
          skip_targets_install,
        )?;
        app
      }
      Err(err) => {
        if err.sdk_or_ndk_issue() {
          Report::action_request(
            " to initialize Android environment; Android support won't be usable until you fix the issue below and re-run `tauri android init`!",
            err,
          )
          .print(wrapper);
          app
        } else {
          return Err(err.into());
        }
      }
    },
    #[cfg(target_os = "macos")]
    // Generate Xcode project
    Target::Ios => {
      let (config, metadata) =
        super::ios::get_config(&app, tauri_config_, None, &Default::default());
      map.insert("apple", &config);
      super::ios::project::gen(
        tauri_config_,
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
        format!("`{}`", s)
      } else {
        s.to_string()
      }
    })
    .collect::<Vec<_>>()
    .join(".");

  out.write(&escaped_result).map_err(Into::into)
}

fn app_root(ctx: &Context) -> Result<&str, RenderError> {
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
