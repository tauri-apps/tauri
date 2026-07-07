// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// Converts from muda mnemonic to gtk mnemonic
///
/// gtk uses underline (_) for mnemonic
/// and two underlines (__) to escape it into a single underline
/// while we use (&) and (&&), so we have to do a few conversions
pub fn to_gtk_mnemonic<S: AsRef<str>>(string: S) -> String {
    string
        .as_ref()
        // escape underlines
        .replace("_", "__")
        // perserve &&
        .replace("&&", "[~~]")
        // transfrom & -> _
        .replace('&', "_")
        // revert back && to unsecaped &
        .replace("[~~]", "&")
}

/// Converts from gtk mnemonic to muda mnemonic
///
/// gtk uses underline (_) for mnemonic
/// and two underlines (__) to escape it into a single underline
/// while we use (&) and (&&), so we have to do a few conversions
#[cfg(test)]
pub fn from_gtk_mnemonic<S: AsRef<str>>(string: S) -> String {
    string
        .as_ref()
        // transform escaped & to unescaped &&
        .replace("&", "&&")
        // perserve __
        .replace("__", "[~~]")
        // transfrom _ -> &
        .replace('_', "&")
        // revert back __ to unescaped _
        .replace("[~~]", "_")
}

#[cfg(test)]
mod tests {
    use crate::platform_impl::platform::mnemonic::{from_gtk_mnemonic, to_gtk_mnemonic};

    #[test]
    fn it_converts() {
        assert_eq!(to_gtk_mnemonic("H&ello"), "H_ello");
        assert_eq!(to_gtk_mnemonic("H&&ello"), "H&ello");
        assert_eq!(to_gtk_mnemonic("H&&&ello"), "H&_ello");
        assert_eq!(to_gtk_mnemonic("H_ello"), "H__ello");
        assert_eq!(to_gtk_mnemonic("H__ello"), "H____ello");
    }

    #[test]
    fn it_converts_back() {
        let str = "H&ello";
        assert_eq!(from_gtk_mnemonic(to_gtk_mnemonic(str)), str);

        let str = "H&&ello";
        assert_eq!(from_gtk_mnemonic(to_gtk_mnemonic(str)), str);

        let str = "H&&&ello";
        assert_eq!(from_gtk_mnemonic(to_gtk_mnemonic(str)), str);

        let str = "H_ello";
        assert_eq!(from_gtk_mnemonic(to_gtk_mnemonic(str)), str);

        let str = "H__ello";
        assert_eq!(from_gtk_mnemonic(to_gtk_mnemonic(str)), str);
    }
}
