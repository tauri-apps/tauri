---
'tauri-runtime-cef': 'patch:bug'
---

`Settings::no_sandbox` is now derived from `Cef::linux_sandbox` on Linux and the BSDs instead of from the `sandbox` cargo feature. That feature is not a build input on those platforms — `cef-dll-sys` only acts on it for Windows and macOS, where it selects which sandbox library is linked — yet it still turned into Chromium's `--no-sandbox` switch before any of the policy code ran. A consumer depending on this crate with `default-features = false` therefore got a fully unsandboxed Chromium while `LinuxSandboxPolicy::Required` reported success and no warning was logged. Windows and macOS keep the previous feature-driven behaviour.
