---
"@tauri-apps/api": minor:feat
---

Add `Channel.deserialize`, which registers a transform for the messages a channel receives and returns the same channel. It lets a caller hand the messages to `onmessage` in a richer type than the JSON value the Rust side serialized, without replacing the channel and so without a second IPC callback.
