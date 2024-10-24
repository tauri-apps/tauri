---
'tauri-utils': 'minor:feat'
'tauri':       'minor:feat'
---
# Feature
Adds a new configuration option for the tauri configuration file.This being `headers` in the app>security. Headers defined the are added to every http response from tauri to the web view. This doesn't include IPC messages and error responses. The header names are limited to:
  - `Access-Control-Allow-Credentials`
  - `Access-Control-Allow-Headers`
  - `Access-Control-Allow-Methods`
  - `Access-Control-Expose-Headers`
  - `Access-Control-Max-Age`
  - `Cross-Origin-Embedder-Policy`
  - `Cross-Origin-Opener-Policy`
  - `Cross-Origin-Resource-Policy`
  - `Permissions-Policy`
  - `Timing-Allow-Origin`
  - `X-Content-Type-Options`
  - `Tauri-Custom-Header`

I primarily wanted to use [`SharedArrayBuffer`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer),
which requires cross-origin isolation. Since there was no effort in adding more headers I looked for the ones, that would make the most sense.
The `Content-Security-Policy`(CSP) remains untouched. I tried to implement a unified way to define headers, including the CSP, but to no avail.
Since it's a very dynamic header, with grave implications for security, it's better to remain untouched.

## Example configuration
```javascript
{
 //..
  app:{
    //..
    security: {
      headers: {
        "Cross-Origin-Opener-Policy": "same-origin",
        "Cross-Origin-Embedder-Policy": "require-corp",
        "Timing-Allow-Origin": [
          "https://developer.mozilla.org",
          "https://example.com",
        ],
        "Access-Control-Expose-Headers": "Tauri-Custom-Header",
        "Tauri-Custom-Header": {
          "key1": "'value1' 'value2'",
          "key2": "'value3'"
        }
      },
      csp: "default-src 'self'; connect-src ipc: http://ipc.localhost",
    }
    //..
  }
 //..
}
```
In this example `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` are set to allow for the use of [`SharedArrayBuffer`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer). The result is, that those headers are then set on every response sent via the `get_response` function in crates/tauri/src/protocol/tauri.rs. The `Content-Security-Policy` header is defined separately, because it is also handled separately. For the helloworld example, this config translates into those response headers:
```http
access-control-allow-origin:  http://tauri.localhost
access-control-expose-headers: Tauri-Custom-Header
content-security-policy: default-src 'self'; connect-src ipc: http://ipc.localhost; script-src 'self' 'sha256-Wjjrs6qinmnr+tOry8x8PPwI77eGpUFR3EEGZktjJNs='
content-type: text/html
cross-origin-embedder-policy: require-corp
cross-origin-opener-policy: same-origin
tauri-custom-header: key1 'value1' 'value2'; key2 'value3'
timing-allow-origin: https://developer.mozilla.org, https://example.com
```
Since the resulting header values are always 'string-like'. So depending on the what data type the HeaderSource is, they need to be converted.
 - `String`(JS/Rust): stay the same for the resulting header value
 - `Array`(JS)/`Vec\<String\>`(Rust): Item are joined by ", " for the resulting header value
 - `Object`(JS)/ `Hashmap\<String,String\>`(Rust): Items are composed from: key + space + value. Item are then joined by "; " for the resulting header value
