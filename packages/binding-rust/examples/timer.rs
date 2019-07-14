//#![windows_subsystem = "windows"]

extern crate web_view;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use web_view::*;

fn main() {
    let counter = Arc::new(Mutex::new(0));

    let counter_inner = counter.clone();
    let webview = web_view::builder()
        .title("Timer example")
        .content(Content::Html(HTML))
        .size(800, 600)
        .resizable(true)
        .debug(true)
        .user_data(0)
        .invoke_handler(|webview, arg| {
            match arg {
                "reset" => {
                    *webview.user_data_mut() += 10;
                    let mut counter = counter.lock().unwrap();
                    *counter = 0;
                    render(webview, *counter)?;
                }
                "exit" => {
                    webview.terminate();
                }
                _ => unimplemented!(),
            };
            Ok(())
        })
        .build()
        .unwrap();

    let handle = webview.handle();
    thread::spawn(move || loop {
        {
            let mut counter = counter_inner.lock().unwrap();
            *counter += 1;
            let count = *counter;
            handle
                .dispatch(move |webview| {
                    *webview.user_data_mut() -= 1;
                    render(webview, count)
                })
                .unwrap();
        }
        thread::sleep(Duration::from_secs(1));
    });

    webview.run().unwrap();
}

fn render(webview: &mut WebView<i32>, counter: u32) -> WVResult {
    let user_data = *webview.user_data();
    println!("counter: {}, userdata: {}", counter, user_data);
    webview.eval(&format!("updateTicks({}, {})", counter, user_data))
}

const HTML: &str = r#"
<!doctype html>
<html>
	<body>
		<p id="ticks"></p>
		<button onclick="external.invoke('reset')">reset</button>
		<button onclick="external.invoke('exit')">exit</button>
		<script type="text/javascript">
			function updateTicks(n, u) {
				document.getElementById('ticks').innerHTML = 'ticks ' + n + '<br>' + 'userdata ' + u;
			}
		</script>
	</body>
</html>
"#;
