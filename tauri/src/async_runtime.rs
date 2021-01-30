use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

use std::sync::Mutex as StdMutex;

pub use tokio::sync::Mutex;

static RUNTIME: OnceCell<StdMutex<Runtime>> = OnceCell::new();

pub fn block_on<F: futures::Future>(future: F) -> F::Output {
  let runtime = RUNTIME.get_or_init(|| StdMutex::new(Runtime::new().unwrap()));
  runtime.lock().unwrap().block_on(future)
}

pub fn spawn<F>(future: F)
where
  F: futures::Future + Send + 'static,
  F::Output: Send + 'static,
{
  let runtime = RUNTIME.get_or_init(|| StdMutex::new(Runtime::new().unwrap()));
  runtime.lock().unwrap().spawn(future);
}
