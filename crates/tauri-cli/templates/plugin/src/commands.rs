{{#if license_header}}
{{ license_header }}
{{/if}}
use tauri::{AppHandle, command, Runtime};

use crate::models::*;
use crate::Result;
use crate::{{ plugin_name_pascal_case }}Ext;

#[command]
pub(crate) async fn ping<R: Runtime>(
    app: AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.{{ plugin_name_snake_case }}().ping(payload)
}
