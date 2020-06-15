#[cfg(windows)]
use crate::config::get as get_config;
#[cfg(windows)]
use std::path::MAIN_SEPARATOR;

#[allow(dead_code)]
pub struct Notification {
  body: Option<String>,
  title: Option<String>,
  icon: Option<String>,
}

impl Notification {
  pub fn new() -> Self {
    Self {
      body: None,
      title: None,
      icon: None,
    }
  }

  pub fn body(&mut self, body: String) -> &mut Self {
    self.body = Some(body);
    self
  }

  pub fn title(&mut self, title: String) -> &mut Self {
    self.title = Some(title);
    self
  }

  pub fn icon(&mut self, icon: String) -> &mut Self {
    self.icon = Some(icon);
    self
  }

  pub fn show(self) -> crate::Result<()> {
    let mut notification = notify_rust::Notification::new();
    if let Some(body) = self.body {
      notification.body(&body);
    }
    if let Some(title) = self.title {
      notification.summary(&title);
    }
    if let Some(icon) = self.icon {
      notification.icon(&icon);
    }
    #[cfg(windows)]
    {
      let exe = std::env::current_exe()?;
      let exe_dir = exe.parent().expect("failed to get exe directory");
      let curr_dir = exe_dir.display().to_string();
      // set the notification's System.AppUserModel.ID only when running the installed app
      if !(curr_dir.ends_with(format!("{S}target{S}debug", S = MAIN_SEPARATOR).as_str())
        || curr_dir.ends_with(format!("{S}target{S}release", S = MAIN_SEPARATOR).as_str()))
      {
        let config = get_config()?;
        let identifier = config.tauri.bundle.identifier.clone();
        notification.app_id(&identifier);
      }
    }
    notification
      .show()
      .map(|_| ())
      .map_err(|e| anyhow::anyhow!(e.to_string()))
  }
}
