extern crate threadpool;
#[macro_use]
extern crate serde_derive;

#[macro_use]
mod macros;

pub mod command;
pub mod file_system;
pub mod dir;
pub mod file;
pub mod rpc;
pub mod tcp;
pub mod http;
pub mod platform;
pub mod version;
pub mod updater;
pub mod process;
pub mod api;

extern crate proton_ui;
use proton_ui::WebView;

use threadpool::ThreadPool;

thread_local!(static POOL: ThreadPool = ThreadPool::new(4));

pub fn run_async<T: 'static, F: FnOnce() -> Result<String, String> + Send + 'static>(webview: &mut WebView<T>, what: F, callback: String, error: String) {
    let handle = webview.handle();
     POOL.with(|thread| {
        thread.execute(move || {
            let callback_string = rpc::format_callback_result(what(), callback, error);
            handle.dispatch(move |_webview| {
                _webview.eval(callback_string.as_str())
            }).unwrap()
    });
    });
}