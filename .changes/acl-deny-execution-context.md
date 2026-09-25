---
tauri: patch:bug
---

Fix the runtime ACL denying a command for every origin when any capability denied it: deny permissions are now correctly scoped to the capability's execution context (local/remote) instead of denying the command for every origin. Previously a capability that denied a command for a remote URL also denied it for the local app (and vice-versa), because the origin match result was discarded. The debug message reporting an explicit denial now also only references the capabilities that actually deny the requesting origin.
