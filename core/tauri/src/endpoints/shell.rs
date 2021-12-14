// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeContext;
use crate::{api::ipc::CallbackFn, Runtime};
#[cfg(shell_execute)]
use crate::{Manager, Scopes};
use serde::Deserialize;
use tauri_macros::{module_command_handler, CommandModule};

#[cfg(shell_execute)]
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, path::PathBuf};

type ChildId = u32;
#[cfg(shell_execute)]
type ChildStore = Arc<Mutex<HashMap<ChildId, crate::api::process::CommandChild>>>;

#[cfg(shell_execute)]
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
    program: PathBuf,
    args: Vec<String>,
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
    program: PathBuf,
    args: Vec<String>,
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
          crate::api::process::Command::new_sidecar(program_as_string)?
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
      if context.window.state::<Scopes>().shell.is_allowed(&program) {
        crate::api::process::Command::new(program.display().to_string())
      } else {
        return Err(crate::Error::ProgramNotAllowed(program));
      }
    };
    #[cfg(any(shell_execute, shell_sidecar))]
    {
      command = command.args(args);
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

  #[module_command_handler(shell_open, "shell > open")]
  fn open<R: Runtime>(
    _context: InvokeContext<R>,
    path: String,
    with: Option<String>,
  ) -> crate::Result<()> {
    match crate::api::shell::open(
      path,
      if let Some(w) = with {
        use std::str::FromStr;
        Some(crate::api::shell::Program::from_str(&w)?)
      } else {
        None
      },
    ) {
      Ok(_) => Ok(()),
      Err(err) => Err(crate::Error::FailedToExecuteApi(err)),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{Buffer, ChildId, CommandOptions};
  use crate::api::ipc::CallbackFn;
  use std::path::PathBuf;

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

  #[tauri_macros::module_command_test(shell_execute, "shell > execute")]
  #[quickcheck_macros::quickcheck]
  fn execute(
    _program: PathBuf,
    _args: Vec<String>,
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
