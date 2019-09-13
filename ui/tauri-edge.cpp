#define WIN32_LEAN_AND_MEAN

#include "tauri-edge.h"

#include <windows.h>
#include <objbase.h>
#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Web.UI.Interop.h>

#pragma comment(lib, "user32.lib")
#pragma comment(lib, "windowsapp")

namespace webview
{
using namespace winrt;
using namespace Windows::Foundation;
using namespace Windows::Web::UI;
using namespace Windows::Web::UI::Interop;

class browser_window
{
public:
    browser_window(msg_cb_t cb, void *window) : m_cb(cb)
    {
        if (window == nullptr)
        {
            WNDCLASSEX wc;
            ZeroMemory(&wc, sizeof(WNDCLASSEX));
            wc.cbSize = sizeof(WNDCLASSEX);
            wc.hInstance = GetModuleHandle(nullptr);
            wc.lpszClassName = "webview";
            wc.lpfnWndProc =
                (WNDPROC)(+[](HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) -> int {
                    auto w = (browser_window *)GetWindowLongPtr(hwnd, GWLP_USERDATA);
                    switch (msg)
                    {
                    case WM_SIZE:
                        w->resize();
                        break;
                    case WM_CLOSE:
                        DestroyWindow(hwnd);
                        break;
                    case WM_DESTROY:
                        w->terminate();
                        break;
                    default:
                        return DefWindowProc(hwnd, msg, wp, lp);
                    }
                    return 0;
                });
            RegisterClassEx(&wc);
            m_window = CreateWindow("webview", "", WS_OVERLAPPEDWINDOW, CW_USEDEFAULT,
                                    CW_USEDEFAULT, 640, 480, nullptr, nullptr,
                                    GetModuleHandle(nullptr), nullptr);
            SetWindowLongPtr(m_window, GWLP_USERDATA, (LONG_PTR)this);
        }
        else
        {
            m_window = *(static_cast<HWND *>(window));
        }

        ShowWindow(m_window, SW_SHOW);
        UpdateWindow(m_window);
        SetFocus(m_window);
    }

    void run()
    {
        MSG msg;
        BOOL res;
        while ((res = GetMessage(&msg, nullptr, 0, 0)) != -1)
        {
            if (msg.hwnd)
            {
                TranslateMessage(&msg);
                DispatchMessage(&msg);
                continue;
            }
            if (msg.message == WM_APP)
            {
                auto f = (dispatch_fn_t *)(msg.lParam);
                (*f)();
                delete f;
            }
            else if (msg.message == WM_QUIT)
            {
                return;
            }
        }
    }

    void terminate() { PostQuitMessage(0); }
    void dispatch(dispatch_fn_t f)
    {
        PostThreadMessage(m_main_thread, WM_APP, 0, (LPARAM) new dispatch_fn_t(f));
    }

    void set_title(const char *title) { SetWindowText(m_window, title); }

    void set_size(int width, int height, bool resizable)
    {
        RECT r;
        r.left = 50;
        r.top = 50;
        r.right = width;
        r.bottom = height;
        AdjustWindowRect(&r, WS_OVERLAPPEDWINDOW, 0);
        SetWindowPos(m_window, NULL, r.left, r.top, r.right - r.left,
                     r.bottom - r.top,
                     SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED);
    }

protected:
    virtual void resize() {}
    HWND m_window;
    DWORD m_main_thread = GetCurrentThreadId();
    msg_cb_t m_cb;
}; // browser_window

class browser_engine : public browser_window
{
public:
    browser_engine(msg_cb_t cb, bool debug, void *window)
        : browser_window(cb, window)
    {
        init_apartment(winrt::apartment_type::single_threaded);
        m_process = WebViewControlProcess();
        auto op = m_process.CreateWebViewControlAsync(
            reinterpret_cast<int64_t>(m_window), Rect());
        if (op.Status() != AsyncStatus::Completed)
        {
            handle h(CreateEvent(nullptr, false, false, nullptr));
            op.Completed([h = h.get()](auto, auto) { SetEvent(h); });
            HANDLE hs[] = {h.get()};
            DWORD i;
            CoWaitForMultipleHandles(COWAIT_DISPATCH_WINDOW_MESSAGES |
                                         COWAIT_DISPATCH_CALLS |
                                         COWAIT_INPUTAVAILABLE,
                                     INFINITE, 1, hs, &i);
        }
        m_webview = op.GetResults();
        m_webview.Settings().IsScriptNotifyAllowed(true);
        m_webview.IsVisible(true);
        m_webview.ScriptNotify([=](auto const &sender, auto const &args) {
            std::string s = winrt::to_string(args.Value());
            m_cb(s.c_str());
        });
        m_webview.NavigationStarting([=](auto const &sender, auto const &args) {
            m_webview.AddInitializeScript(winrt::to_hstring(init_js));
        });
        init("window.external.invoke = s => window.external.notify(s)");
        resize();
    }

    void navigate(const char *url)
    {
        Uri uri(winrt::to_hstring(url));
        // TODO: if url starts with 'data:text/html,' prefix then use it as a string
        m_webview.Navigate(uri);
        // m_webview.NavigateToString(winrt::to_hstring(url));
    }
    void init(const char *js)
    {
        init_js = init_js + "(function(){" + js + "})();";
    }
    void eval(const char *js)
    {
        m_webview.InvokeScriptAsync(
            L"eval", single_threaded_vector<hstring>({winrt::to_hstring(js)}));
    }

private:
    void resize()
    {
        RECT r;
        GetClientRect(m_window, &r);
        Rect bounds(r.left, r.top, r.right - r.left, r.bottom - r.top);
        m_webview.Bounds(bounds);
    }
    WebViewControlProcess m_process;
    WebViewControl m_webview = nullptr;
    std::string init_js = "";
}; // browser_engine

class webview : public browser_engine
{
public:
    webview(bool debug = false, void *wnd = nullptr)
        : browser_engine(
              std::bind(&webview::on_message, this, std::placeholders::_1), debug,
              wnd) {}

    void *window() { return (void *)m_window; }

    void navigate(const char *url)
    {
        std::string html = html_from_uri(url);
        if (html != "")
        {
            browser_engine::navigate(("data:text/html," + url_encode(html)).c_str());
        }
        else
        {
            browser_engine::navigate(url);
        }
    }

    using binding_t = std::function<std::string(std::string)>;

    void bind(const char *name, binding_t f)
    {
        auto js = "(function() { var name = '" + std::string(name) + "';" + R"(
      window[name] = function() {
        var me = window[name];
        var errors = me['errors'];
        var callbacks = me['callbacks'];
        if (!callbacks) {
          callbacks = {};
          me['callbacks'] = callbacks;
        }
        if (!errors) {
          errors = {};
          me['errors'] = errors;
        }
        var seq = (me['lastSeq'] || 0) + 1;
        me['lastSeq'] = seq;
        var promise = new Promise(function(resolve, reject) {
          callbacks[seq] = resolve;
          errors[seq] = reject;
        });
        window.external.invoke(JSON.stringify({
          name: name,
          seq:seq,
          args: Array.prototype.slice.call(arguments),
        }));
        return promise;
      }
    })())";
        init(js.c_str());
        bindings[name] = new binding_t(f);
    }

private:
    void on_message(const char *msg)
    {
        auto seq = json_parse(msg, "seq", 0);
        auto name = json_parse(msg, "name", 0);
        auto args = json_parse(msg, "args", 0);
        auto fn = bindings[name];
        if (fn == nullptr)
        {
            return;
        }
        std::async(std::launch::async, [=]() {
            auto result = (*fn)(args);
            dispatch([=]() {
                eval(("var b = window['" + name + "'];b['callbacks'][" + seq + "](" +
                      result + ");b['callbacks'][" + seq +
                      "] = undefined;b['errors'][" + seq + "] = undefined;")
                         .c_str());
            });
        });
    }
    std::map<std::string, binding_t *> bindings;
}; // webview

} // namespace webview

WEBVIEW_API webview_t webview_create(int debug, void *wnd)
{
    return new webview::webview(debug, wnd);
}

WEBVIEW_API void webview_destroy(webview_t w)
{
    delete static_cast<webview::webview *>(w);
}

WEBVIEW_API void webview_run(webview_t w)
{
    static_cast<webview::webview *>(w)->run();
}

WEBVIEW_API void webview_terminate(webview_t w)
{
    static_cast<webview::webview *>(w)->terminate();
}

WEBVIEW_API void
webview_dispatch(webview_t w, void (*fn)(webview_t w, void *arg), void *arg)
{
    static_cast<webview::webview *>(w)->dispatch([=]() { fn(w, arg); });
}

WEBVIEW_API void *webview_get_window(webview_t w)
{
    return static_cast<webview::webview *>(w)->window();
}

WEBVIEW_API void webview_set_title(webview_t w, const char *title)
{
    static_cast<webview::webview *>(w)->set_title(title);
}

WEBVIEW_API void webview_set_bounds(webview_t w, int x, int y, int width,
                                    int height, int flags)
{
    // TODO: x, y, flags
    static_cast<webview::webview *>(w)->set_size(width, height, true);
}

WEBVIEW_API void webview_get_bounds(webview_t w, int *x, int *y, int *width,
                                    int *height, int *flags)
{
    // TODO
}

WEBVIEW_API void webview_navigate(webview_t w, const char *url)
{
    static_cast<webview::webview *>(w)->navigate(url);
}

WEBVIEW_API void webview_init(webview_t w, const char *js)
{
    static_cast<webview::webview *>(w)->init(js);
}

WEBVIEW_API void webview_eval(webview_t w, const char *js)
{
    static_cast<webview::webview *>(w)->eval(js);
}