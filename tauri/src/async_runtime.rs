use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

use std::sync::Mutex;

static RUNTIME: OnceCell<Mutex<Runtime>> = OnceCell::new();

pub(crate) fn block_on<F: futures::Future>(future: F) -> F::Output {
  let runtime = RUNTIME.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
  runtime.lock().unwrap().block_on(future)
}

pub(crate) fn spawn<F>(future: F)
where
  F: futures::Future + Send + 'static,
  F::Output: Send + 'static,
{
  let runtime = RUNTIME.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
  runtime.lock().unwrap().spawn(future);
}
