// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeContext;
use crate::{api::ipc::CallbackFn, Runtime};
#[cfg(shell_scope)]
use crate::{Manager, Scopes};
use serde::Deserialize;
use tauri_macros::{module_command_handler, CommandModule};

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
fn command_childs() -> &'static ChildStore {
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
}

/// The API descriptor.
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The execute script API.
  #[serde(rename_all = "camelCase")]
  Execute {
    program: String,
    args: ExecuteArgs,
    on_event_fn: CallbackFn,
    #[serde(default)]
    options: CommandOptions,
  },
  StdinWrite {
    pid: ChildId,
    buffer: Buffer,
  },
  KillChild {
    pid: ChildId,
  },
  Open {
    path: String,
    with: Option<String>,
  },
}

impl Cmd {
  #[allow(unused_variables)]
  fn execute<R: Runtime>(
    context: InvokeContext<R>,
    program: String,
    args: ExecuteArgs,
    on_event_fn: CallbackFn,
    options: CommandOptions,
  ) -> crate::Result<ChildId> {
    let mut command = if options.sidecar {
      #[cfg(not(shell_sidecar))]
      return Err(crate::Error::ApiNotAllowlisted(
        "shell > sidecar".to_string(),
      ));
      #[cfg(shell_sidecar)]
      {
        let program = PathBuf::from(program);
        let program_as_string = program.display().to_string();
        let program_no_ext_as_string = program.with_extension("").display().to_string();
        let is_configured = context
          .config
          .tauri
          .bundle
          .external_bin
          .as_ref()
          .map(|bins| {
            bins
              .iter()
              .any(|b| b == &program_as_string || b == &program_no_ext_as_string)
          })
          .unwrap_or_default();
        if is_configured {
          context
            .window
            .state::<Scopes>()
            .shell
            .prepare(&program.to_string_lossy(), args, true)
            .map_err(Box::new)?
        } else {
          return Err(crate::Error::SidecarNotAllowed(program));
        }
      }
    } else {
      #[cfg(not(shell_execute))]
      return Err(crate::Error::ApiNotAllowlisted(
        "shell > execute".to_string(),
      ));
      #[cfg(shell_execute)]
      match context
        .window
        .state::<Scopes>()
        .shell
        .prepare(&program, args, false)
      {
        Ok(cmd) => cmd,
        Err(e) => {
          #[cfg(debug_assertions)]
          eprintln!("{}", e);
          return Err(crate::Error::ProgramNotAllowed(PathBuf::from(program)));
        }
      }
    };
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
      let (mut rx, child) = command.spawn()?;

      let pid = child.pid();
      command_childs().lock().unwrap().insert(pid, child);

      crate::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
          if matches!(event, crate::api::process::CommandEvent::Terminated(_)) {
            command_childs().lock().unwrap().remove(&pid);
          }
          let js = crate::api::ipc::format_callback(on_event_fn, &event)
            .expect("unable to serialize CommandEvent");

          let _ = context.window.eval(js.as_str());
        }
      });

      Ok(pid)
    }
  }

  #[cfg(any(shell_execute, shell_sidecar))]
  fn stdin_write<R: Runtime>(
    _context: InvokeContext<R>,
    pid: ChildId,
    buffer: Buffer,
  ) -> crate::Result<()> {
    if let Some(child) = command_childs().lock().unwrap().get_mut(&pid) {
      match buffer {
        Buffer::Text(t) => child.write(t.as_bytes())?,
        Buffer::Raw(r) => child.write(&r)?,
      }
    }
    Ok(())
  }

  #[cfg(not(any(shell_execute, shell_sidecar)))]
  fn stdin_write<R: Runtime>(
    _context: InvokeContext<R>,
    _pid: ChildId,
    _buffer: Buffer,
  ) -> crate::Result<()> {
    Err(crate::Error::ApiNotAllowlisted(
      "shell > execute or shell > sidecar".into(),
    ))
  }

  #[cfg(any(shell_execute, shell_sidecar))]
  fn kill_child<R: Runtime>(_context: InvokeContext<R>, pid: ChildId) -> crate::Result<()> {
    if let Some(child) = command_childs().lock().unwrap().remove(&pid) {
      child.kill()?;
    }
    Ok(())
  }

  #[cfg(not(any(shell_execute, shell_sidecar)))]
  fn kill_child<R: Runtime>(_context: InvokeContext<R>, _pid: ChildId) -> crate::Result<()> {
    Err(crate::Error::ApiNotAllowlisted(
      "shell > execute or shell > sidecar".into(),
    ))
  }

  /// Open a (url) path with a default or specific browser opening program.
  ///
  /// See [`crate::api::shell::open`] for how it handles security-related measures.
  #[module_command_handler(shell_open, "shell > open")]
  fn open<R: Runtime>(
    context: InvokeContext<R>,
    path: String,
    with: Option<String>,
  ) -> crate::Result<()> {
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
