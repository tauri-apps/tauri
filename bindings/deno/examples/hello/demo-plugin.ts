// A self-contained plugin — the kind of thing you'd ship as its own package
// (`import { definePlugin } from '@tauri-apps/deno/plugin'`). It bundles a
// frontend API (installed by the init script) with its backend command, so app
// code never touches the `plugin:demo|echo` wire format.
import { definePlugin } from '../../plugin.ts'

export default definePlugin('demo')
  .initScript(`
    window.__DEMO_PLUGIN__ = 'demo plugin init script ran'
    window.demo = {
      echo: (payload) => window.__TAURI__.core.invoke('plugin:demo|echo', payload)
    }
  `)
  .command('echo', (payload) => ({ echoed: payload }))
