// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{Target, get_app};
use crate::{
  ConfigValue, Result,
  helpers::app_paths::Dirs,
  helpers::{config::get_config as get_tauri_config, template::JsonMap},
  interface::AppInterface,
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

use std::{
  env::var_os,
  path::{Path, PathBuf},
};

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

  let cwd = match std::env::current_dir() {
    Ok(dir) => Some(dir),
    Err(err) => {
      log::warn!("failed to resolve the current directory: {err}");
      None
    }
  };

  let package_manager = PackageManagerContext {
    pnpm_package_name: var_os("PNPM_PACKAGE_NAME").is_some(),
    pnpm_dlx: is_pnpm_dlx(),
    pnpm_run: is_pnpm_run(),
    npm_execpath: var_os("npm_execpath"),
  };

  let (binary, mut build_args) =
    command_from_argv(&mut std::env::args_os(), cwd.as_deref(), &package_manager)?;

  build_args.push(target.command_name().into());
  build_args.push(target.ide_build_script_name().into());

  let mut binary = binary.to_string_lossy().to_string();
  if binary.ends_with(".exe") || binary.ends_with(".cmd") || binary.ends_with(".bat") {
    // remove Windows-only extension
    binary.pop();
    binary.pop();
    binary.pop();
    binary.pop();
  }

  map.insert("tauri-binary", binary);
  map.insert("tauri-binary-args", &build_args);
  map.insert(
    "tauri-binary-args-str",
    build_args
      .iter()
      .map(|arg| shell_escape(arg))
      .collect::<Vec<_>>()
      .join(" "),
  );

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

      super::android::project::generate(
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
      super::ios::project::generate(
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
  h.register_helper("shell-escape", Box::new(shell_escape_helper));
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

/// Quotes a string so it is interpreted as a single word by a POSIX shell.
fn shell_escape(s: &str) -> String {
  if !s.is_empty()
    && s
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || "-_./:@%+=,".contains(c))
  {
    s.to_string()
  } else {
    format!("'{}'", s.replace('\'', r"'\''"))
  }
}

fn shell_escape_helper(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(&shell_escape(get_str(helper)))
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

/// pnpm's native binary (pnpm >= 11) runs package scripts without setting `PNPM_PACKAGE_NAME`,
/// and `npm_execpath` points at the binary itself, which is not necessarily named `pnpm`
/// (corepack downloads it as `pnpm-native`), so the user agent is what identifies it.
fn is_pnpm_run() -> bool {
  std::env::var("npm_config_user_agent").is_ok_and(|user_agent| user_agent.starts_with("pnpm/"))
}

/// The package-manager context the invoking process environment provides, read
/// once by `exec` and injected so `command_from_argv` stays testable without
/// process-global env mutation.
struct PackageManagerContext {
  pnpm_package_name: bool,
  pnpm_dlx: bool,
  pnpm_run: bool,
  npm_execpath: Option<std::ffi::OsString>,
}

/// Resolve the `(binary, args)` pair written into the mobile build tasks from
/// the invoking command line: `argv[0]` becomes the binary, and the arguments
/// are picked so that the generated Gradle/Xcode task can actually run (see
/// the node fallback below and tauri#9536).
fn command_from_argv(
  argv: &mut impl Iterator<Item = std::ffi::OsString>,
  cwd: Option<&Path>,
  pm: &PackageManagerContext,
) -> crate::Result<(std::ffi::OsString, Vec<String>)> {
  let Some(bin) = argv.next() else {
    return Ok((std::ffi::OsString::from("cargo"), vec!["tauri".into()]));
  };

  let bin_path = PathBuf::from(&bin);
  let mut build_args: Vec<String> = vec!["tauri".into()];

  if let Some(bin_stem) = bin_path.file_stem() {
    let r = regex::Regex::new("(nodejs|node)\\-?([1-9]*)*$").unwrap();
    if r.is_match(&bin_stem.to_string_lossy()) {
      if pm.pnpm_package_name {
        return Ok(("pnpm".into(), build_args));
      } else if pm.pnpm_dlx {
        return Ok(("pnpm".into(), vec!["dlx".into(), "@tauri-apps/cli".into()]));
      } else if pm.pnpm_run {
        return Ok(("pnpm".into(), build_args));
      } else if let Some(manager_stem) = pm.npm_execpath.as_ref().and_then(|path| {
        PathBuf::from(path)
          .file_stem()
          .map(|stem| stem.to_os_string())
      }) {
        let is_npm = manager_stem == "npm-cli";
        let binary = if is_npm {
          "npm".into()
        } else if manager_stem == "npx-cli" {
          "npx".into()
        } else {
          manager_stem
        };

        if is_npm {
          build_args.insert(0, "run".into());
          build_args.insert(1, "--".into());
        }

        return Ok((binary, build_args));
      }

      // No package-manager context: the npm CLI runs as a plain node script, so
      // emit `<node> <entry-script> <args…>` — argv[1] is the JS entry that node
      // loaded for this CLI, and the JS CLI parses subcommands directly. The old
      // fallthrough emitted `<node> tauri …`, which can never run: node would try
      // to load a file or module named "tauri" (tauri#9536). The entry is adopted
      // only when it is valid UTF-8 and resolves (against this working directory)
      // to an existing .js/.cjs/.mjs file — the script node just executed must be
      // on disk, and .bin/tauri-style extension-less symlinks canonicalize to it —
      // and a relative entry is absolutized because the generated Gradle task runs
      // from a different working directory. Anything else — flags before the
      // script, stale paths, subcommand-shaped words, hosts embedding the CLI —
      // aborts the generation with a diagnostic instead of silently emitting a
      // build task that is guaranteed to fail at the first Gradle/Xcode build,
      // since scanning argv cannot tell flag values from the entry. The entry
      // must also be the Tauri CLI itself (`tauri.js`), not an arbitrary host
      // script that loads it.
      let entry = match argv.next().map(PathBuf::from) {
        Some(entry) => entry,
        None => crate::error::bail!(
          "no package-manager context was detected and no first argument was provided; the mobile \
           build task would run `<node> tauri …`, which cannot work"
        ),
      };
      let relative = !entry.is_absolute();
      let entry_is_utf8 = entry.to_str().is_some();
      let absolute = if entry.is_absolute() {
        entry
      } else {
        match cwd {
          Some(dir) => dir.join(&entry),
          None => crate::error::bail!(
            "failed to resolve the current directory, so the CLI entry script cannot be absolutized; \
             the mobile build task would run `<node> tauri …`, which cannot work. Regenerate from a \
             valid working directory or invoke the tauri CLI binary directly"
          ),
        }
      };
      // a non-UTF-8 working directory can corrupt an otherwise valid relative
      // entry; a non-UTF-8 absolute entry is the host's own fault. Either way it
      // cannot be embedded verbatim.
      if absolute.to_str().is_none() {
        if relative && entry_is_utf8 {
          crate::error::bail!(
            "the current directory is unavailable or not valid UTF-8, so the CLI entry script \
             cannot be absolutized; the mobile build task would run `<node> tauri …`, which cannot \
             work"
          );
        }
        crate::error::bail!(
          "the CLI entry script path is not valid UTF-8, so it cannot be embedded in the generated \
           build task"
        );
      }
      if !looks_like_entry_script(&absolute) {
        crate::error::bail!(
          "no usable package-manager context was detected under node, and `{}` could not be \
           adopted as the CLI entry script; the mobile build task would run `<node> tauri …`, \
           which cannot work. Regenerate through a package-manager context (for example `pnpm tauri \
           …`) or invoke the tauri CLI binary directly",
          absolute.display()
        );
      }
      // the generated task runs from the app's src-tauri dir, so a binary that
      // was invoked through a relative path (e.g. ./node) would not resolve
      // there; bare names keep relying on PATH, like the npm/pnpm branches do
      let bin = match bin_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => match cwd {
          Some(dir) => dir.join(&bin_path).into_os_string(),
          None => crate::error::bail!(
            "failed to resolve the current directory, so the relative binary path `{}` cannot be \
             absolutized; the mobile build task would not be able to run it. Regenerate from a \
             valid working directory or invoke the tauri CLI binary directly",
            bin_path.display()
          ),
        },
        _ => bin,
      };
      build_args[0] = absolute.to_string_lossy().into_owned();
      return Ok((bin, build_args));
    } else if bin_stem == "deno" {
      build_args.insert(0, "task".into());
      return Ok((std::ffi::OsString::from("deno"), build_args));
    } else if !cfg!(debug_assertions) && bin_stem == "cargo-tauri" {
      return Ok((std::ffi::OsString::from("cargo"), build_args));
    }
  }

  Ok((bin, build_args))
}

/// Whether the resolved CLI entry is the Tauri CLI script itself: an existing
/// .js/.cjs/.mjs file (after canonicalization, so `.bin/tauri`-style
/// extension-less symlinks resolve to `tauri.js`) whose file stem is `tauri`.
/// The entry script node just executed must be on disk, so flags, stale paths
/// and exotic hosts that embed the CLI in some other way are rejected and a
/// diagnostic is emitted instead.
fn looks_like_entry_script(entry: &Path) -> bool {
  // follow symlinks: npm installs `node_modules/.bin/tauri` as an extension-less
  // symlink to `tauri.js`, and node reports that symlink path in argv[1]
  let resolved = entry.canonicalize().unwrap_or_else(|_| entry.to_path_buf());
  resolved.is_file()
    && resolved.extension().is_some_and(|ext| {
      ext.eq_ignore_ascii_case("js")
        || ext.eq_ignore_ascii_case("cjs")
        || ext.eq_ignore_ascii_case("mjs")
    })
    && resolved
      .file_stem()
      .is_some_and(|stem| stem.eq_ignore_ascii_case("tauri"))
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
  use super::{PackageManagerContext, command_from_argv, looks_like_entry_script, shell_escape};
  use std::ffi::{OsStr, OsString};
  use std::path::Path;

  #[test]
  fn shell_escape_words() {
    assert_eq!(shell_escape("cargo"), "cargo");
    assert_eq!(shell_escape("xcode-script"), "xcode-script");
    assert_eq!(
      shell_escape("/Users/me/node_modules/.bin/tauri"),
      "/Users/me/node_modules/.bin/tauri"
    );
    assert_eq!(
      shell_escape("/Users/me/My Projects/tauri"),
      "'/Users/me/My Projects/tauri'"
    );
    assert_eq!(shell_escape("it's $HOME"), r"'it'\''s $HOME'");
    assert_eq!(shell_escape(""), "''");
  }

  fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(|i| OsStr::new(i).to_os_string()).collect()
  }

  /// The no-package-manager context the node-fallback tests need.
  fn pm_none() -> PackageManagerContext {
    PackageManagerContext {
      pnpm_package_name: false,
      pnpm_dlx: false,
      pnpm_run: false,
      npm_execpath: None,
    }
  }

  #[test]
  fn entry_script_requires_an_existing_script_file() {
    let dir = tempfile::tempdir().unwrap();
    // the predicate also requires the canonical CLI entry name (tauri)
    let probe = dir.path().join("tauri.js");
    std::fs::write(&probe, b"// probe").unwrap();
    assert!(looks_like_entry_script(&probe));
    // missing paths, directories and flag-like names do not exist as scripts
    assert!(!looks_like_entry_script(&dir.path().join("missing.js")));
    assert!(!looks_like_entry_script(dir.path()));
    assert!(!looks_like_entry_script(Path::new("--help")));
    // an existing script with a different stem (e.g. a host script that loads
    // the CLI) is rejected as well
    let host = dir.path().join("main.js");
    std::fs::write(&host, b"// host").unwrap();
    assert!(!looks_like_entry_script(&host));
  }

  #[test]
  fn node_fallback_adopts_and_absolutizes_the_entry_script() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("tauri.js"), b"// entry").unwrap();
    let node = dir.path().join("node");
    // a RELATIVE argv[1] is resolved against the injected working directory
    let (binary, args) = command_from_argv(
      &mut argv(&[node.to_str().unwrap(), "tauri.js"]).into_iter(),
      Some(dir.path()),
      &pm_none(),
    )
    .unwrap();
    assert_eq!(binary, node.into_os_string());
    assert_eq!(
      args,
      vec![dir.path().join("tauri.js").to_string_lossy().into_owned()]
    );
  }

  #[test]
  fn node_fallback_adopts_absolute_entries_verbatim() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("tauri.js"), b"// entry").unwrap();
    let node = dir.path().join("node");
    let entry = dir.path().join("tauri.js");
    let (binary, args) = command_from_argv(
      &mut argv(&[node.to_str().unwrap(), entry.to_str().unwrap()]).into_iter(),
      Some(dir.path()),
      &pm_none(),
    )
    .unwrap();
    assert_eq!(binary, node.into_os_string());
    assert_eq!(args, vec![entry.to_string_lossy().into_owned()]);
  }

  #[test]
  fn relative_node_binary_is_absolutized() {
    let dir = tempfile::tempdir().unwrap();
    let entry = dir.path().join("tauri.js");
    std::fs::write(&entry, b"// entry").unwrap();
    // a RELATIVE argv[0] with a directory component is absolutized like the entry
    let (binary, args) = command_from_argv(
      &mut argv(&["sub/node", entry.to_str().unwrap()]).into_iter(),
      Some(dir.path()),
      &pm_none(),
    )
    .unwrap();
    assert_eq!(binary, dir.path().join("sub/node").into_os_string());
    assert_eq!(args, vec![entry.to_string_lossy().into_owned()]);
  }

  #[test]
  fn relative_node_binary_without_a_cwd_bails() {
    let dir = tempfile::tempdir().unwrap();
    let entry = dir.path().join("tauri.js");
    std::fs::write(&entry, b"// entry").unwrap();
    // argv[0] is relative and cwd=None models the current_dir() failure path;
    // the ENTRY is absolute so the bin-absolutization bail is what fires
    let err = command_from_argv(
      &mut argv(&["sub/node", entry.to_str().unwrap()]).into_iter(),
      None,
      &pm_none(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("cannot be absolutized"));
  }

  #[test]
  #[cfg(unix)]
  fn extensionless_bin_symlinks_resolve_to_the_script() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("tauri.js"), b"// entry").unwrap();
    let shim = dir.path().join("tauri");
    symlink(dir.path().join("tauri.js"), &shim).unwrap();
    let node = dir.path().join("node");
    // node reports the extension-less SYMLINK path in argv[1]; canonicalization
    // must resolve it to tauri.js or the .bin/tauri case is rejected. The adopted
    // entry is the symlink path as node reported it (node follows the link).
    let (binary, args) = command_from_argv(
      &mut argv(&[node.to_str().unwrap(), shim.to_str().unwrap()]).into_iter(),
      Some(dir.path()),
      &pm_none(),
    )
    .unwrap();
    assert_eq!(binary, node.into_os_string());
    assert_eq!(args, vec![shim.to_string_lossy().into_owned()]);
  }

  #[test]
  fn node_fallback_bails_for_non_script_entries() {
    let dir = tempfile::tempdir().unwrap();
    let node = dir.path().join("node");
    let err = command_from_argv(
      &mut argv(&[node.to_str().unwrap(), "android"]).into_iter(),
      Some(dir.path()),
      &pm_none(),
    )
    .unwrap_err();
    assert!(
      err
        .to_string()
        .contains("could not be adopted as the CLI entry script")
    );
  }

  #[test]
  fn relative_entries_without_a_cwd_bail_with_a_diagnostic() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("tauri.js"), b"// entry").unwrap();
    let node = dir.path().join("node");
    // argv[1] is relative and cwd=None models the current_dir() failure path
    let err = command_from_argv(
      &mut argv(&[node.to_str().unwrap(), "tauri.js"]).into_iter(),
      None,
      &pm_none(),
    )
    .unwrap_err();
    assert!(
      err
        .to_string()
        .contains("failed to resolve the current directory")
    );
  }

  #[test]
  fn pnpm_package_name_branch() {
    let pm = PackageManagerContext {
      pnpm_package_name: true,
      ..pm_none()
    };
    let (binary, args) = command_from_argv(
      &mut argv(&["/usr/local/bin/node", "android"]).into_iter(),
      None,
      &pm,
    )
    .unwrap();
    assert_eq!(binary, OsString::from("pnpm"));
    assert_eq!(args, vec!["tauri".to_string()]);
  }

  #[test]
  fn pnpm_dlx_branch() {
    let pm = PackageManagerContext {
      pnpm_dlx: true,
      ..pm_none()
    };
    let (binary, args) = command_from_argv(
      &mut argv(&["/usr/local/bin/node", "android"]).into_iter(),
      None,
      &pm,
    )
    .unwrap();
    assert_eq!(binary, OsString::from("pnpm"));
    // the dlx form is `pnpm dlx @tauri-apps/cli <command>` — no leading "tauri"
    assert_eq!(args, vec!["dlx".to_string(), "@tauri-apps/cli".to_string()]);
  }

  #[test]
  fn pnpm_run_branch() {
    let pm = PackageManagerContext {
      pnpm_run: true,
      ..pm_none()
    };
    let (binary, args) = command_from_argv(
      &mut argv(&["/usr/local/bin/node", "android"]).into_iter(),
      None,
      &pm,
    )
    .unwrap();
    assert_eq!(binary, OsString::from("pnpm"));
    assert_eq!(args, vec!["tauri".to_string()]);
  }

  #[test]
  fn unusable_npm_execpath_falls_through_to_the_node_fallback() {
    // a value with no file_stem (like "/") yields None; the old code panicked
    // there. Note: on Windows an empty value deletes the variable entirely, so
    // this test uses "/" to exercise the present-but-unusable path everywhere.
    let pm = PackageManagerContext {
      npm_execpath: Some(OsString::from("/")),
      ..pm_none()
    };
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("tauri.js"), b"// entry").unwrap();
    let node = dir.path().join("node");
    let (binary, args) = command_from_argv(
      &mut argv(&[node.to_str().unwrap(), "tauri.js"]).into_iter(),
      Some(dir.path()),
      &pm,
    )
    .unwrap();
    assert_eq!(binary, node.into_os_string());
    assert_eq!(
      args,
      vec![dir.path().join("tauri.js").to_string_lossy().into_owned()]
    );
  }

  #[test]
  fn usable_npm_execpath_resolves_the_package_manager() {
    let pm = PackageManagerContext {
      npm_execpath: Some(OsString::from("/usr/local/lib/node_modules/npm-cli")),
      ..pm_none()
    };
    let (binary, args) = command_from_argv(
      &mut argv(&["/usr/local/bin/node", "android"]).into_iter(),
      None,
      &pm,
    )
    .unwrap();
    assert_eq!(binary, OsString::from("npm"));
    assert_eq!(
      args,
      vec!["run".to_string(), "--".to_string(), "tauri".to_string()]
    );
  }

  #[test]
  fn deno_branch() {
    let (binary, args) = command_from_argv(
      &mut argv(&["deno", "android"]).into_iter(),
      None,
      &pm_none(),
    )
    .unwrap();
    assert_eq!(binary, OsString::from("deno"));
    assert_eq!(args, vec!["task".to_string(), "tauri".to_string()]);
  }

  #[test]
  fn non_node_invocations_pass_through() {
    let (binary, args) = command_from_argv(
      &mut argv(&["cargo-tauri", "android"]).into_iter(),
      None,
      &pm_none(),
    )
    .unwrap();
    // `cargo-tauri` is rewritten to `cargo` only in release builds
    let expected = if cfg!(debug_assertions) {
      OsString::from("cargo-tauri")
    } else {
      OsString::from("cargo")
    };
    assert_eq!(binary, expected);
    assert_eq!(args, vec!["tauri".to_string()]);
  }

  #[test]
  #[cfg(unix)]
  fn non_utf8_paths_are_attributed_accurately() {
    use std::os::unix::ffi::OsStringExt;

    let dir = tempfile::tempdir().unwrap();
    let node = dir.path().join("node");
    // a non-UTF-8 ABSOLUTE entry is the host's own fault
    let bad_entry =
      std::ffi::OsString::from_vec(b"/tmp/tauri-cli-".iter().copied().chain([0xFF]).collect());
    let err = command_from_argv(
      &mut vec![node.clone().into_os_string(), bad_entry].into_iter(),
      Some(dir.path()),
      &pm_none(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("not valid UTF-8"));
    // a valid relative entry under a non-UTF-8 cwd is attributed to the cwd
    let bad_cwd =
      std::ffi::OsString::from_vec(b"/tmp/tauri-cli-".iter().copied().chain([0xFE]).collect());
    let err = command_from_argv(
      &mut argv(&[node.to_str().unwrap(), "tauri.js"]).into_iter(),
      Some(Path::new(&bad_cwd)),
      &pm_none(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("current directory"));
  }
}
