const { invoke } = window.__TAURI__.core;
const smokeMode = invoke('smoke_enabled');
const result = document.querySelector('#result');
const status = document.querySelector('#status');
const draft = document.querySelector('#draft');
let busy = false;
let previousLabel;
let countBeforeReload = 0;
const documentIdentity = crypto.randomUUID();
window.reloadDocumentIdentity = documentIdentity;

async function update(command = 'read') {
  if (busy) return;
  busy = true;
  try {
    const value = await invoke('reload_call', { command });
    result.textContent = JSON.stringify(value, null, 2);
    status.textContent = 'Connected';
    // Used by the headless WebView acceptance test; harmless in normal usage.
    if (await smokeMode && previousLabel === undefined) {
      previousLabel = value.label;
      const first = await invoke('reload_call', { command: 'increment' });
      countBeforeReload = first.count;
      draft.value = 'preserved across Rust reload';
      await invoke('smoke_report', { phase: 'ready' });
    } else if (await smokeMode && value.label !== previousLabel) {
      const passed = value.count >= countBeforeReload &&
        draft.value === 'preserved across Rust reload' &&
        window.reloadDocumentIdentity === documentIdentity;
      await invoke('smoke_report', { phase: passed ? 'passed' : 'failed' });
      previousLabel = value.label;
    }
  } catch (error) {
    status.textContent = String(error);
  } finally {
    busy = false;
  }
}
document.querySelector('#increment').onclick = () => update('increment');
setInterval(() => update(), 300);
update();
