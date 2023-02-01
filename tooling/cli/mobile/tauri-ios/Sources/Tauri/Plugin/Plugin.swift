import WebKit
import os.log

@objc public protocol Plugin {
    @objc func load(webview: WKWebView)
}
