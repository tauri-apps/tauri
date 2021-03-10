use crate::app::InvokeResponse;
use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The execute script API.
  Execute { command: String, args: Vec<String> },
  /// Open path with specified or default system app
  Open { path: String, with: Option<String> },
}

impl Cmd {
  pub async fn run(self) -> crate::Result<InvokeResponse> {
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
        {
          open(path, with)
        }
        #[cfg(not(shell_open))]
        Err(crate::Error::ApiNotAllowlisted("shell > open".to_string()))
      }
    }
  }
}

#[cfg(shell_open)]
pub fn open(path: String, with: Option<String>) -> crate::Result<InvokeResponse> {
  #[cfg(test)]
  {
    assert!(path.contains("http://"));
    Ok(().into())
  }

  #[cfg(not(test))]
  {
    let exit_status = if let Some(with) = with {
      open::with(&path, &with)
    } else {
      open::that(&path)
    };
    match exit_status {
      Ok(status) => {
        if !status.success() {
          // TODO: handle this
        }
        Ok(().into())
      }
      Err(_) => Err(crate::Error::ApiNotAllowlisted(
        "TODO: MAKE THIS BETTER".into(),
      )),
    }
  }
}

#[cfg(test)]
mod test {
  use proptest::prelude::*;
  // Test the open func to see if proper uris can be opened by the browser.
  proptest! {
    #[cfg(shell_open)]
    #[test]
    fn check_open(uri in r"(http://)([\\w\\d\\.]+([\\w]{2,6})?)") {
      super::open(uri, None).unwrap();
    }
  }
}
