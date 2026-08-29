use js_sys;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(
    inline_js = "export function invoke_tauri(cmd, args = {}) { return window.__TAURI__._invoke(cmd, args, window.__TAURI_INVOKE_KEY__) }"
)]
extern "C" {
    async fn invoke_tauri(cmd: &str, args: JsValue) -> JsValue;
}

pub async fn invoke<M: Serialize + Deserialize<'static>>(args: InvokeCommand<M>) -> JsValue {
    invoke_tauri("tauri".into(), JsValue::from_serde(&args).unwrap()).await
}

pub trait TauriMessage<M>
where
    M: Serialize + Deserialize<'static>,
{
    fn cmd(&self) -> M;
    fn module(&self) -> String;
}

#[derive(Serialize, Deserialize)]
pub struct InvokeCommand<M> {
    #[serde(rename = "__tauriModule")]
    pub tauri_module: String,
    pub message: M,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum App {
    GetAppVersion,
}

#[derive(Serialize, Deserialize)]
pub struct AppMessage {
    pub cmd: App,
}

impl TauriMessage<App> for AppMessage {
    fn cmd(&self) -> App {
        self.cmd.clone()
    }
    fn module(&self) -> String {
        "App".into()
    }
}

impl From<App> for InvokeCommand<AppMessage> {
    fn from(cmd: App) -> Self {
        let msg = AppMessage { cmd };
        InvokeCommand {
            tauri_module: msg.module().into(),
            message: msg,
        }
    }
}

#[test]
fn tauri_cmd() -> () {
    let args = InvokeCommand::from(App::GetAppVersion);
    wasm_bindgen_futures::spawn_local( async move {
        let result = invoke(args).await;
        log::info!("{}", result.as_string().unwrap())
    })
}
