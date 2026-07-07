---
'tao': patch
---

fix(linux): map JIS keyboard specific keys (`Zenkaku_Hankaku`, `Hiragana_Katakana`, `Henkan`, `Muhenkan`) in `raw_key_to_key` to prevent them from becoming `Key::Unidentified`.
