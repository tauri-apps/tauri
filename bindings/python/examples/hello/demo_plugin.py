# A self-contained plugin — the kind of thing you'd ship as its own package.
# It bundles a frontend API (installed by the init script) with its backend
# command, so app code never touches the `plugin:demo|echo` wire format.

from tauri_ffi import Plugin

demo = Plugin("demo").init_script(
    """
    window.__DEMO_PLUGIN__ = 'demo plugin init script ran'
    window.demo = {
      echo: (payload) => window.__TAURI__.core.invoke('plugin:demo|echo', payload)
    }
    """
)


@demo.command("echo")
def echo(payload, _message):
    return {"echoed": payload}
