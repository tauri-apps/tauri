import WebKit

public protocol Plugin {
    func load(webview: WKWebView)
}

public extension Plugin {
    func load(webview: WKWebView) { }
}
