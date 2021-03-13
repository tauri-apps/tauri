use crate::app::InvokeResponse;
use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The execute script API.
  Execute {
    command: String,
    args: Vec<String>,
  },
  Open {
    path: String,
    with: Option<String>,
  },
}

impl Cmd {
  pub fn run(self) -> crate::Result<InvokeResponse> {
    match self {
      Self::Execute {
        command: _,
        args: _,
      } => {
        #[cfg(shell_execute)]
        {
          //TODO
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
