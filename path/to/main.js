import EventPlugin from 'tauri-plugin-event';

const eventPlugin = new EventPlugin();

async function main() {
    const windowId = 1; // replace with actual window ID
    const eventName = 'test_event';
    const data = 'test_data';
    const response = await eventPlugin.emitTo(windowId, eventName, data);
    if (response !== null) {
        console.log(response);
    }
}

main();