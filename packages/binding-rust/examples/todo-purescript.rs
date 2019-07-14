#![windows_subsystem = "windows"]

extern crate web_view;

use web_view::*;

fn main() {
    web_view::builder()
        .title("Rust / PureScript - Todo App")
        .content(Content::Html(include_str!("todo-ps/dist/bundle.html")))
        .size(320, 480)
        .resizable(false)
        .debug(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .run()
        .unwrap();
}
