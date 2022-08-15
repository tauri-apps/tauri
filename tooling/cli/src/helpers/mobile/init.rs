use cargo_mobile::android;
#[cfg(target_os = "macos")]
use cargo_mobile::apple;
use cargo_mobile::{
  config::{
    self,
    metadata::{self, Metadata},
    Config,
  },
  dot_cargo,
  init::{DOT_FIRST_INIT_CONTENTS, DOT_FIRST_INIT_FILE_NAME},
  opts,
  os::code_command,
  util::{
    self,
    cli::{Report, TextWrapper},
  },
};
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError};

use crate::helpers::template::JsonMap;

use std::{
  fs, io,
  path::{Path, PathBuf},
};

pub use opts::{NonInteractive, OpenInEditor, ReinstallDeps, SkipDevTools};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  ConfigLoadOrGenFailed(config::LoadOrGenError),
  #[error("failed to init first init file {path}: {cause}")]
  DotFirstInitWriteFailed { path: PathBuf, cause: io::Error },
  #[error("failed to create asset dir {asset_dir}: {cause}")]
  AssetDirCreationFailed {
    asset_dir: PathBuf,
    cause: io::Error,
  },
  #[error(transparent)]
  CodeCommandPresentFailed(bossy::Error),
  #[error(transparent)]
  LldbExtensionInstallFailed(bossy::Error),
  #[error(transparent)]
  DotCargoLoadFailed(dot_cargo::LoadError),
  #[error(transparent)]
  HostTargetTripleDetectionFailed(util::HostTargetTripleError),
  #[error(transparent)]
  MetadataFailed(metadata::Error),
  #[cfg(target_os = "macos")]
  #[error(transparent)]
  AppleInitFailed(apple::project::Error),
  #[error(transparent)]
  AndroidEnvFailed(android::env::Error),
  #[error(transparent)]
  AndroidInitFailed(super::android::project::Error),
  #[error(transparent)]
  DotCargoWriteFailed(dot_cargo::WriteError),
  #[error("failed to delete first init file {path}: {cause}")]
  DotFirstInitDeleteFailed { path: PathBuf, cause: io::Error },
  #[error(transparent)]
  OpenInEditorFailed(util::OpenInEditorError),
}

pub fn exec(
  wrapper: &TextWrapper,
  non_interactive: opts::NonInteractive,
  skip_dev_tools: opts::SkipDevTools,
  #[allow(unused_variables)] reinstall_deps: opts::ReinstallDeps,
  open_in_editor: opts::OpenInEditor,
  cwd: impl AsRef<Path>,
) -> Result<Config, Error> {
  let cwd = cwd.as_ref();
  let (config, config_origin) =
    Config::load_or_gen(cwd, non_interactive, wrapper).map_err(Error::ConfigLoadOrGenFailed)?;
  let dot_first_init_path = config.app().root_dir().join(DOT_FIRST_INIT_FILE_NAME);
  let dot_first_init_exists = {
    let dot_first_init_exists = dot_first_init_path.exists();
    if config_origin.freshly_minted() && !dot_first_init_exists {
      // indicate first init is ongoing, so that if we error out and exit
      // the next init will know to still use `WildWest` filtering
      log::info!("creating first init dot file at {:?}", dot_first_init_path);
      fs::write(&dot_first_init_path, DOT_FIRST_INIT_CONTENTS).map_err(|cause| {
        Error::DotFirstInitWriteFailed {
          path: dot_first_init_path.clone(),
          cause,
        }
      })?;
      true
    } else {
      dot_first_init_exists
    }
  };

  let asset_dir = config.app().asset_dir();
  if !asset_dir.is_dir() {
    fs::create_dir_all(&asset_dir)
      .map_err(|cause| Error::AssetDirCreationFailed { asset_dir, cause })?;
  }
  if skip_dev_tools.no()
    && util::command_present("code").map_err(Error::CodeCommandPresentFailed)?
  {
    let mut command = code_command();
    command.add_args(&["--install-extension", "vadimcn.vscode-lldb"]);
    if non_interactive.yes() {
      command.add_arg("--force");
    }
    command
      .run_and_wait()
      .map_err(Error::LldbExtensionInstallFailed)?;
  }
  let mut dot_cargo = dot_cargo::DotCargo::load(config.app()).map_err(Error::DotCargoLoadFailed)?;
  // Mysteriously, builds that don't specify `--target` seem to fight over
  // the build cache with builds that use `--target`! This means that
  // alternating between i.e. `cargo run` and `cargo apple run` would
  // result in clean builds being made each time you switched... which is
  // pretty nightmarish. Specifying `build.target` in `.cargo/config`
  // fortunately has the same effect as specifying `--target`, so now we can
  // `cargo run` with peace of mind!
  //
  // This behavior could be explained here:
  // https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags
  dot_cargo.set_default_target(
    util::host_target_triple().map_err(Error::HostTargetTripleDetectionFailed)?,
  );

  let metadata = Metadata::load(&config.app().root_dir()).map_err(Error::MetadataFailed)?;

  // Generate Xcode project
  #[cfg(target_os = "macos")]
  if metadata.apple().supported() {
    /*super::apple::project::gen(
      config.apple(),
      metadata.apple(),
      config.app().template_pack().submodule_path(),
      wrapper,
      non_interactive,
      skip_dev_tools,
      reinstall_deps,
    )
    .map_err(Error::AppleInitFailed)?;*/
  } else {
    println!("Skipping iOS init, since it's marked as unsupported in your Cargo.toml metadata");
  }

  // Generate Android Studio project
  if metadata.android().supported() {
    match android::env::Env::new() {
      Ok(env) => super::android::project::gen(
        config.android(),
        metadata.android(),
        &env,
        handlebars(&config),
        wrapper,
        &mut dot_cargo,
      )
      .map_err(Error::AndroidInitFailed)?,
      Err(err) => {
        if err.sdk_or_ndk_issue() {
          Report::action_request(
            "Failed to initialize Android environment; Android support won't be usable until you fix the issue below and re-run `cargo mobile init`!",
            err,
          )
          .print(wrapper);
        } else {
          Err(Error::AndroidEnvFailed(err))?;
        }
      }
    }
  } else {
    println!("Skipping Android init, since it's marked as unsupported in your Cargo.toml metadata");
  }

  dot_cargo
    .write(config.app())
    .map_err(Error::DotCargoWriteFailed)?;
  if dot_first_init_exists {
    log::info!("deleting first init dot file at {:?}", dot_first_init_path);
    fs::remove_file(&dot_first_init_path).map_err(|cause| Error::DotFirstInitDeleteFailed {
      path: dot_first_init_path,
      cause,
    })?;
  }
  Report::victory(
    "Project generated successfully!",
    "Make cool apps! 🌻 🐕 🎉",
  )
  .print(wrapper);
  if open_in_editor.yes() {
    util::open_in_editor(cwd).map_err(Error::OpenInEditorFailed)?;
  }
  Ok(config)
}

fn handlebars(config: &Config) -> (Handlebars<'static>, JsonMap) {
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
  h.register_helper("reverse-domain", Box::new(reverse_domain));
  h.register_helper(
    "reverse-domain-snake-case",
    Box::new(reverse_domain_snake_case),
  );
  // don't mix these up or very bad things will happen to all of us
  h.register_helper("prefix-path", Box::new(prefix_path));
  h.register_helper("unprefix-path", Box::new(unprefix_path));

  let mut map = JsonMap::default();
  map.insert("app", config.app());
  #[cfg(target_os = "macos")]
  map.insert("apple", config.apple());
  map.insert("android", config.android());

  (h, map)
}

fn get_str<'a>(helper: &'a Helper) -> &'a str {
  helper
    .param(0)
    .and_then(|v| v.value().as_str())
    .unwrap_or_else(|| "")
}

fn get_str_array<'a>(
  helper: &'a Helper,
  formatter: impl Fn(&str) -> String,
) -> Option<Vec<String>> {
  helper.param(0).and_then(|v| {
    v.value().as_array().and_then(|arr| {
      arr
        .iter()
        .map(|val| val.as_str().map(|s| formatter(s)))
        .collect()
    })
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
      &get_str_array(helper, |s| format!("{}", s))
        .ok_or_else(|| RenderError::new("`join` helper wasn't given an array"))?
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
      &get_str_array(helper, |s| format!("{:?}", s))
        .ok_or_else(|| RenderError::new("`quote-and-join` helper wasn't given an array"))?
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
      &get_str_array(helper, |s| format!("{:?}", format!(":{}", s)))
        .ok_or_else(|| {
          RenderError::new("`quote-and-join-colon-prefix` helper wasn't given an array")
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

fn reverse_domain(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(&util::reverse_domain(get_str(helper)))
    .map_err(Into::into)
}

fn reverse_domain_snake_case(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  use heck::ToSnekCase as _;
  out
    .write(&util::reverse_domain(get_str(helper)).to_snek_case())
    .map_err(Into::into)
}

fn app_root<'a>(ctx: &'a Context) -> Result<&'a str, RenderError> {
  let app_root = ctx
    .data()
    .get("app")
    .ok_or_else(|| RenderError::new("`app` missing from template data."))?
    .get("root-dir")
    .ok_or_else(|| RenderError::new("`app.root-dir` missing from template data."))?;
  app_root
    .as_str()
    .ok_or_else(|| RenderError::new("`app.root-dir` contained invalid UTF-8."))
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
          RenderError::new(
            "Either the `app.root-dir` or the specified path contained invalid UTF-8.",
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
          RenderError::new("Attempted to unprefix a path that wasn't in the app root dir.")
        })?
        .to_str()
        .ok_or_else(|| {
          RenderError::new(
            "Either the `app.root-dir` or the specified path contained invalid UTF-8.",
          )
        })?,
    )
    .map_err(Into::into)
}
