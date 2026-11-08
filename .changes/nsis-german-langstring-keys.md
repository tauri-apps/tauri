---
"tauri-bundler": patch:bug
---

Fixed the German NSIS installer defining the `older` and `unknown` strings under translated identifiers (`älter` and `unbekannt`), which made makensis emit `warning 6040: LangString "older" is not set in language table of language German` and left the version kind blank in the "An $R4 version of ... is installed on your system" prompt.
