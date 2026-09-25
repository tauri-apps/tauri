---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

When rewriting a localhost `devUrl` to the network address for mobile development, the query string and fragment are now preserved, and the IPv6 loopback (`[::1]`) and unspecified (`[::]`) addresses are now treated as localhost.
