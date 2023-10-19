{#if license_header}}
{{ license_header }}
{{/if}}
import { invoke } from '@tauri-apps/api/primitives'

export async function execute() {
  await invoke('plugin:{{ plugin_name }}|execute')
}
