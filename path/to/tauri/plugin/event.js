import { Event } from 'tauri-plugin-base';

class EventPlugin extends Event {
  async emitTo(windowId, eventName, data) {
    try {
      const startTime = Date.now();
      const response = await this._emitTo(windowId, eventName, data);
      if (!response) {
        throw new Error('No response received from backend window');
      }
      if (Date.now() - startTime > 5000) { // 5-second timeout
        throw new Error('Timeout waiting for response from backend window');
      }
      return response;
    } catch (error) {
      console.error(error);
      return null;
    }
  }

  async _emitTo(windowId, eventName, data) {
    // existing implementation of emitTo
    return null;
  }
}

export default EventPlugin;