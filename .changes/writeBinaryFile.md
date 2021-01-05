---
"tauri.js": patch
"tauri": minor
---

fix writeBinaryFile in the js api to call the correct function
on the rust side. Before, files were sent in base64 encoding, but
the save-as-text `WriteFile` fs command was reused, without any 
indication that the contents were base64 encoded. So the WriteFile
command had no way of checking and decoding when necessary. 
WriteBinaryFile on the rust side expects a base64 encoded string, then
decodes it.

