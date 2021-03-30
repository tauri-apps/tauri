use crate::{
  api::{
    command::{Command, CommandChild, CommandEvent},
    rpc::format_callback,
  },
  app::{ApplicationExt, InvokeResponse},
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
  KillChild {
    pid: ChildId,
  },
  Open {
    path: String,
    with: Option<String>,
  },
}

impl Cmd {
  pub fn run<A: ApplicationExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<A>,
  ) -> crate::Result<InvokeResponse> {
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

          let webview_manager = webview_manager.clone();
          crate::async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
              if matches!(event, CommandEvent::Finish(_)) {
                command_childs().lock().unwrap().remove(&pid);
              }
              let js = format_callback(on_event_fn.clone(), serde_json::to_value(event).unwrap());
              if let Ok(dispatcher) = webview_manager.current_webview() {
                let _ = dispatcher.eval(js.as_str());
              }
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
