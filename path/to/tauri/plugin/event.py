import time
from tauri_plugin_base import Event

class EventPlugin(Event):
    def emit_to(self, window_id, event_name, data):
        try:
            start_time = time.time()
            response = self._emit_to(window_id, event_name, data)
            if response is None:
                raise TimeoutError("No response received from backend window")
            if time.time() - start_time > 5:  # 5-second timeout
                raise TimeoutError("Timeout waiting for response from backend window")
            return response
        except TimeoutError as e:
            print(f"Error: {e}")
            return None
        except Exception as e:
            print(f"Error: {e}")
            return None

    def _emit_to(self, window_id, event_name, data):
        # existing implementation of emit_to
        pass