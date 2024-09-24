# `tauri-driver` _(pre-alpha)_

Cross-platform WebDriver server for Tauri applications.

This is a [WebDriver Intermediary Node] that wraps the native WebDriver server
for platforms that [Tauri] supports. Your WebDriver client will connect to the
running `tauri-driver` server, and `tauri-driver` will handle starting the
native WebDriver server for you behind the scenes. It requires two separate
ports to be used since two distinct [WebDriver Remote Ends] run.

You can configure the ports used with arguments when starting the binary:

- `--port` (default: `4444`)
- `--native-port` (default: `4445`)

Supported platforms:

- **[pre-alpha]** Linux w/ `WebKitWebDriver`
- **[pre-alpha]** Windows w/ [Microsoft Edge Driver]
- **[Todo]** macOS w/ [Appium Mac2 Driver] (probably)

_note: the (probably) items haven't been proof-of-concept'd yet, and if it is
not possible to use the listed native webdriver, then a custom implementation
will be used that wraps around [wry]._

## Trying it out

Check out the documentation at https://tauri.app/docs/testing/webdriver/introduction,
including a small example application with WebDriver tests.

[WebDriver Intermediary Node]: https://www.w3.org/TR/webdriver/#dfn-intermediary-nodes
[WebDriver Remote Ends]: https://www.w3.org/TR/webdriver/#dfn-remote-ends
[Microsoft Edge Driver]: https://developer.microsoft.com/en-us/microsoft-edge/tools/webdriver/
[Appium Mac2 Driver]: https://github.com/appium/appium-mac2-driver
[wry]: https://github.com/tauri-apps/wry
[Tauri]: https://github.com/tauri-apps/tauri
