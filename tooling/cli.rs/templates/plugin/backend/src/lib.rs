{{#if license_header}}
{{ license_header }}
{{/if}}
use tauri::{plugin::Plugin, Runtime};

#[derive(Default)]
pub struct YourPlugin {}

impl<R: Runtime> Plugin<R> for YourPlugin {
  fn name(&self) -> &'static str {
    "{{ plugin_name }}"
  }
}
