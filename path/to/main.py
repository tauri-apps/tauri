from tauri_plugin_event import EventPlugin

event_plugin = EventPlugin()

def main():
    window_id = 1  # replace with actual window ID
    event_name = 'test_event'
    data = 'test_data'
    response = event_plugin.emit_to(window_id, event_name, data)
    if response is not None:
        print(response)

if __name__ == '__main__':
    main()