// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
if (window.location.pathname.startsWith("/page2")) {
  console.log("hello from javascript in page2");
} else {
  console.log("hello from javascript in page1");

  if (typeof WebAssembly.instantiateStreaming !== "undefined") {
    WebAssembly.instantiateStreaming(fetch("/wasm.wasm")).then((wasm) => {
      console.log(wasm.instance.exports.main()); // should log 42
    });
  } else {
    // Older WKWebView may not support `WebAssembly.instantiateStreaming` yet.
    fetch("/wasm.wasm")
      .then((response) => response.arrayBuffer())
      .then((bytes) => WebAssembly.instantiate(bytes))
      .then((wasm) => {
        console.log(wasm.instance.exports.main()); // should log 42
      });
  }
}
