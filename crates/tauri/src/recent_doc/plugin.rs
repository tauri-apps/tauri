use crate::{
  plugin::{Builder, TauriPlugin},
  Runtime,
};
use crate::recent_doc::windows::*;

pub(crate) fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("recent_doc")
    .invoke_handler(crate::generate_handler![
      #![plugin(recent_doc)]
      add_recent_document,
      clear_recent_documents,
      get_recent_documents
    ])
    .build()
}
