use wasm_bindgen::prelude::*;
use std::sync::Arc;
use once_cell::sync::Lazy;
use futures_signals::signal::{Mutable, SignalExt};
use dominator::{Dom, html, class, clone, events};


#[derive(Debug)]
struct App {
    message: Mutable<String>,
}

impl App {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            message: Mutable::new("Hello world!".to_string()),
        })
    }

    fn render(app: Arc<Self>) -> Dom {
        // Define CSS styles
        static CLASS: Lazy<String> = Lazy::new(|| class! {
            .style("font-size", "20px")
            .style("color", "hsl(110, 70%, 70%)")
        });

        // Create the DOM nodes
        html!("div", {
            .class(&*CLASS)

            .text_signal(app.message.signal_cloned().map(|message| {
                format!("Message: {}", message)
            }))

            .event(clone!(app => move |_: events::Click| {
                app.message.set_neq("Goodbye!".to_string());
            }))
        })
    }
}


#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let app = App::new();
    dominator::append_dom(&dominator::body(), App::render(app));

    Ok(())
}
