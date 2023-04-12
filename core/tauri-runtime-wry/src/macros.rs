#[macro_export]
macro_rules! getter {
  ($self: ident, $rx: expr, $message: expr) => {{
    $crate::send_user_message(&$self.context, $message)?;
    $rx
      .recv()
      .map_err(|_| $crate::Error::FailedToReceiveMessage)
  }};
}

macro_rules! window_getter {
  ($self: ident, $message: expr) => {{
    let (tx, rx) = std::sync::mpsc::channel();
    getter!($self, rx, Message::Window($self.webview_id, $message(tx)))
  }};
}

pub(crate) use {getter, window_getter};
