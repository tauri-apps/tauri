// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  api::{
    command::{Command, CommandChild, CommandEvent},
    rpc::format_callback,
  },
  endpoints::InvokeResponse,
  Params, Window,
};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

type ChildId = u32;
type ChildStore = Arc<Mutex<HashMap<ChildId, CommandChild>>>;

fn command_childs() -> &'static ChildStore {
  static STORE: Lazy<ChildStore> = Lazy::new(Default::default);
  &STORE
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Buffer {
  Text(String),
  Raw(Vec<u8>),
}

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The execute script API.
  #[serde(rename_all = "camelCase")]
  Execute {
    program: String,
    args: Vec<String>,
    on_event_fn: String,
    #[serde(default)]
    sidecar: bool,
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
  pub fn run<M: Params>(self, window: Window<M>) -> crate::Result<InvokeResponse> {
    match self {
      Self::Execute {
        program,
        args,
        on_event_fn,
        sidecar,
      } => {
        #[cfg(shell_execute)]
        {
          let mut command = if sidecar {
            Command::new_sidecar(program)
          } else {
            Command::new(program)
          };
          command = command.args(args);
          let (mut rx, child) = command.spawn()?;

          let pid = child.pid();
          command_childs().lock().unwrap().insert(pid, child);

          crate::async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
              if matches!(event, CommandEvent::Terminated(_)) {
                command_childs().lock().unwrap().remove(&pid);
              }
              let js = format_callback(on_event_fn.clone(), &event)
                .expect("unable to serialize CommandEvent");

              let _ = window.eval(js.as_str());
            }
          });

          Ok(pid.into())
        }
        #[cfg(not(shell_execute))]
        Err(crate::Error::ApiNotAllowlisted(
          "shell > execute".to_string(),
        ))
      }
      Self::KillChild { pid } => {
        #[cfg(shell_execute)]
        {
          if let Some(child) = command_childs().lock().unwrap().remove(&pid) {
            child.kill()?;
          }
          Ok(().into())
        }
        #[cfg(not(shell_execute))]
        Err(crate::Error::ApiNotAllowlisted(
          "shell > execute".to_string(),
        ))
      }
      Self::StdinWrite { pid, buffer } => {
        #[cfg(shell_execute)]
        {
          if let Some(child) = command_childs().lock().unwrap().get_mut(&pid) {
            match buffer {
              Buffer::Text(t) => child.write(t.as_bytes())?,
              Buffer::Raw(r) => child.write(&r)?,
            }
          }
          Ok(().into())
        }
        #[cfg(not(shell_execute))]
        Err(crate::Error::ApiNotAllowlisted(
          "shell > execute".to_string(),
        ))
      }
      Self::Open { path, with } => {
        #[cfg(shell_open)]
        match crate::api::shell::open(path, with) {
          Ok(_) => Ok(().into()),
          Err(err) => Err(crate::Error::FailedToExecuteApi(err)),
        }

        #[cfg(not(shell_open))]
        Err(crate::Error::ApiNotAllowlisted("shell > open".to_string()))
      }
    }
  }
}
