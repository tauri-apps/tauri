// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(unused_imports)]

use super::InvokeContext;
use crate::{api::ipc::CallbackFn, Runtime};
#[cfg(shell_scope)]
use crate::{Manager, Scopes};
use serde::{Deserialize, Serialize};
use tauri_macros::{command_enum, module_command_handler, CommandModule};

#[cfg(shell_scope)]
use crate::ExecuteArgs;
#[cfg(not(shell_scope))]
type ExecuteArgs = ();

#[cfg(any(shell_execute, shell_sidecar))]
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, path::PathBuf};

type ChildId = u32;
#[cfg(any(shell_execute, shell_sidecar))]
type ChildStore = Arc<Mutex<HashMap<ChildId, crate::api::process::CommandChild>>>;

#[cfg(any(shell_execute, shell_sidecar))]
fn command_child_store() -> &'static ChildStore {
  use once_cell::sync::Lazy;
  static STORE: Lazy<ChildStore> = Lazy::new(Default::default);
  &STORE
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Buffer {
  Text(String),
  Raw(Vec<u8>),
}

#[allow(clippy::unnecessary_wraps)]
fn default_env() -> Option<HashMap<String, String>> {
  Some(HashMap::default())
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandOptions {
  #[serde(default)]
  sidecar: bool,
  cwd: Option<PathBuf>,
  // by default we don't add any env variables to the spawned process
  // but the env is an `Option` so when it's `None` we clear the env.
  #[serde(default = "default_env")]
  env: Option<HashMap<String, String>>,
  // Character encoding for stdout/stderr
  encoding: Option<String>,
}

/// The API descriptor.
#[command_enum]
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The execute and return script API.
  #[cmd(shell_script, "shell > execute or shell > sidecar")]
  #[serde(rename_all = "camelCase")]
  ExecuteAndReturn {
    program: String,
    args: ExecuteArgs,
    #[serde(default)]
    options: CommandOptions,
  },
  /// The execute script API.
  #[cmd(shell_script, "shell > execute or shell > sidecar")]
  #[serde(rename_all = "camelCase")]
  Execute {
    program: String,
    args: ExecuteArgs,
    on_event_fn: CallbackFn,
    #[serde(default)]
    options: CommandOptions,
  },
  #[cmd(shell_script, "shell > execute or shell > sidecar")]
  StdinWrite { pid: ChildId, buffer: Buffer },
  #[cmd(shell_script, "shell > execute or shell > sidecar")]
  KillChild { pid: ChildId },
  #[cmd(shell_open, "shell > open")]
  Open { path: String, with: Option<String> },
}

#[derive(Serialize)]
#[cfg(any(shell_execute, shell_sidecar))]
struct ChildProcessReturn {
  code: Option<i32>,
  signal: Option<i32>,
  stdout: String,
  stderr: String,
}

impl Cmd {
  #[module_command_handler(shell_script)]
  #[allow(unused_variables)]
  fn execute_and_return<R: Runtime>(
    context: InvokeContext<R>,
    program: String,
    args: ExecuteArgs,
    options: CommandOptions,
  ) -> super::Result<ChildProcessReturn> {
    let mut command = prepare_cmd(&context, &program, args, &options)?;

    #[cfg(any(shell_execute, shell_sidecar))]
    {
      if let Some(cwd) = options.cwd {
        command = command.current_dir(cwd);
      }
      if let Some(env) = options.env {
        command = command.envs(env);
      } else {
        command = command.env_clear();
      }

      let encoding = if let Some(encoding) = options.encoding {
        if let Some(encoding) = crate::api::process::Encoding::for_label(encoding.as_bytes()) {
          Some(encoding)
        } else {
          return Err(anyhow::anyhow!(format!("unknown encoding {encoding}")));
        }
      } else {
        None
      };

      let mut command: std::process::Command = command.into();
      let output = command.output()?;

      let (stdout, stderr) = match encoding {
        Some(encoding) => (
          encoding.decode_with_bom_removal(&output.stdout).0.into(),
          encoding.decode_with_bom_removal(&output.stderr).0.into(),
        ),
        None => (
          String::from_utf8(output.stdout)?,
          String::from_utf8(output.stderr)?,
        ),
      };

      #[cfg(unix)]
      use std::os::unix::process::ExitStatusExt;

      Ok(ChildProcessReturn {
        code: output.status.code(),
        #[cfg(windows)]
        signal: None,
        #[cfg(unix)]
        signal: output.status.signal(),
        stdout,
        stderr,
      })
    }
  }

  #[module_command_handler(shell_script)]
  #[allow(unused_variables)]
  fn execute<R: Runtime>(
    context: InvokeContext<R>,
    program: String,
    args: ExecuteArgs,
    on_event_fn: CallbackFn,
    options: CommandOptions,
  ) -> super::Result<ChildId> {
    use std::future::Future;
    use std::pin::Pin;

    let mut command = prepare_cmd(&context, &program, args, &options)?;

    #[cfg(any(shell_execute, shell_sidecar))]
    {
      if let Some(cwd) = options.cwd {
        command = command.current_dir(cwd);
      }
      if let Some(env) = options.env {
        command = command.envs(env);
      } else {
        command = command.env_clear();
      }
      if let Some(encoding) = options.encoding {
        if let Some(encoding) = crate::api::process::Encoding::for_label(encoding.as_bytes()) {
          command = command.encoding(encoding);
        } else {
          return Err(anyhow::anyhow!(format!("unknown encoding {encoding}")));
        }
      }
      let (mut rx, child) = command.spawn()?;

      let pid = child.pid();
      command_child_store().lock().unwrap().insert(pid, child);

      crate::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
          if matches!(event, crate::api::process::CommandEvent::Terminated(_)) {
            command_child_store().lock().unwrap().remove(&pid);
          }
          let js = crate::api::ipc::format_callback(on_event_fn, &event)
            .expect("unable to serialize CommandEvent");

          let _ = context.window.eval(js.as_str());
        }
      });

      Ok(pid)
    }
  }

  #[module_command_handler(shell_script)]
  fn stdin_write<R: Runtime>(
    _context: InvokeContext<R>,
    pid: ChildId,
    buffer: Buffer,
  ) -> super::Result<()> {
    if let Some(child) = command_child_store().lock().unwrap().get_mut(&pid) {
      match buffer {
        Buffer::Text(t) => child.write(t.as_bytes())?,
        Buffer::Raw(r) => child.write(&r)?,
      }
    }
    Ok(())
  }

  #[module_command_handler(shell_script)]
  fn kill_child<R: Runtime>(_context: InvokeContext<R>, pid: ChildId) -> super::Result<()> {
    if let Some(child) = command_child_store().lock().unwrap().remove(&pid) {
      child.kill()?;
    }
    Ok(())
  }

  /// Open a (url) path with a default or specific browser opening program.
  ///
  /// See [`crate::api::shell::open`] for how it handles security-related measures.
  #[module_command_handler(shell_open)]
  fn open<R: Runtime>(
    context: InvokeContext<R>,
    path: String,
    with: Option<String>,
  ) -> super::Result<()> {
    use std::str::FromStr;

    with
      .as_deref()
      // only allow pre-determined programs to be specified
      .map(crate::api::shell::Program::from_str)
      .transpose()
      .map_err(Into::into)
      // validate and open path
      .and_then(|with| {
        crate::api::shell::open(&context.window.state::<Scopes>().shell, path, with)
          .map_err(Into::into)
      })
  }
}

fn prepare_cmd<R: Runtime>(
  context: &InvokeContext<R>,
  program: &String,
  args: ExecuteArgs,
  options: &CommandOptions,
) -> super::Result<crate::api::process::Command> {
  if options.sidecar {
    #[cfg(not(shell_sidecar))]
    return Err(crate::Error::ApiNotAllowlisted("shell > sidecar".to_string()).into_anyhow());
    #[cfg(shell_sidecar)]
    {
      let program = PathBuf::from(program);
      let program_as_string = program.display().to_string();
      let program_no_ext_as_string = program.with_extension("").display().to_string();
      let configured_sidecar = context
        .config
        .tauri
        .bundle
        .external_bin
        .as_ref()
        .map(|bins| {
          bins
            .iter()
            .find(|b| b == &&program_as_string || b == &&program_no_ext_as_string)
        })
        .unwrap_or_default();
      if let Some(sidecar) = configured_sidecar {
        context
          .window
          .state::<Scopes>()
          .shell
          .prepare_sidecar(&program.to_string_lossy(), sidecar, args)
          .map_err(crate::error::into_anyhow)
      } else {
        Err(crate::Error::SidecarNotAllowed(program).into_anyhow())
      }
    }
  } else {
    #[cfg(not(shell_execute))]
    return Err(crate::Error::ApiNotAllowlisted("shell > execute".to_string()).into_anyhow());
    #[cfg(shell_execute)]
    match context
      .window
      .state::<Scopes>()
      .shell
      .prepare(program, args)
    {
      Ok(cmd) => Ok(cmd),
      Err(e) => {
        #[cfg(debug_assertions)]
        eprintln!("{e}");
        Err(crate::Error::ProgramNotAllowed(PathBuf::from(program)).into_anyhow())
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{Buffer, ChildId, CommandOptions, ExecuteArgs};
  use crate::api::ipc::CallbackFn;
  use quickcheck::{Arbitrary, Gen};

  impl Arbitrary for CommandOptions {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        sidecar: false,
        cwd: Option::arbitrary(g),
        env: Option::arbitrary(g),
        encoding: Option::arbitrary(g),
      }
    }
  }

  impl Arbitrary for Buffer {
    fn arbitrary(g: &mut Gen) -> Self {
      Buffer::Text(String::arbitrary(g))
    }
  }

  #[cfg(shell_scope)]
  impl Arbitrary for ExecuteArgs {
    fn arbitrary(_: &mut Gen) -> Self {
      ExecuteArgs::None
    }
  }

  #[tauri_macros::module_command_test(shell_execute, "shell > execute")]
  #[quickcheck_macros::quickcheck]
  fn execute(
    _program: String,
    _args: ExecuteArgs,
    _on_event_fn: CallbackFn,
    _options: CommandOptions,
  ) {
  }

  #[tauri_macros::module_command_test(shell_execute, "shell > execute or shell > sidecar")]
  #[quickcheck_macros::quickcheck]
  fn stdin_write(_pid: ChildId, _buffer: Buffer) {}

  #[tauri_macros::module_command_test(shell_execute, "shell > execute or shell > sidecar")]
  #[quickcheck_macros::quickcheck]
  fn kill_child(_pid: ChildId) {}

  #[tauri_macros::module_command_test(shell_open, "shell > open")]
  #[quickcheck_macros::quickcheck]
  fn open(_path: String, _with: Option<String>) {}
}
