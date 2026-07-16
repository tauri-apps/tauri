use tauri_plugin_event::EventPlugin;

fn main() {
    let event_plugin = EventPlugin::new();
    let window_id = 1; // replace with actual window ID
    let event_name = String::from("test_event");
    let data = String::from("test_data");
    let response = event_plugin.emit_to(window_id, event_name, data);
    if let Some(response) = response {
        println!("{}", response);
    }
}