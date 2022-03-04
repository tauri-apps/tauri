{{#if license_header}}
{{ license_header }}
{{/if}}
use tauri::{plugin::{Builder, TauriPlugin}, runtime::Runtime};

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("{{ plugin_name }}").build()
}
