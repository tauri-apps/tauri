use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;
pub use tokio::sync::Mutex;

use std::future::Future;

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub fn block_on<F: Future>(task: F) -> F::Output {
  let runtime = RUNTIME.get_or_init(|| Runtime::new().unwrap());
  runtime.block_on(task)
}

pub fn spawn<F>(task: F)
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  let runtime = RUNTIME.get_or_init(|| Runtime::new().unwrap());
  runtime.spawn(task);
}
