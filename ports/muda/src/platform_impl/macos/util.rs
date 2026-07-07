// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::str;

/// Strips single `&` characters from the string.
///
/// `&` can be escaped as `&&` to prevent stripping, in which case a single `&` will be output.
pub fn strip_mnemonic<S: AsRef<str>>(string: S) -> String {
    let string = string.as_ref();
    let mut result = String::with_capacity(string.len());
    let mut chars = string.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '&' && chars.peek() == Some(&'&') {
            result.push('&');
            chars.next();
        } else if c != '&' {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::strip_mnemonic;

    #[test]
    fn strips_mnemonics_without_placeholder_collisions() {
        assert_eq!(strip_mnemonic("&File && Edit [~~]"), "File & Edit [~~]");
    }
}
