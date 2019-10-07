#include "webview-edge.h"
#ifdef __cplusplus
extern "C"
{
#endif

    void wrapper_webview_free(webview_t w)
    {
        webview_destroy(w);
    }

    webview_t wrapper_webview_new(const char *title, const char *url, int width, int height, int resizable, int debug, webview_external_invoke_cb_t external_invoke_cb, void *userdata)
    {
        webview_t w = webview_create(external_invoke_cb, title, width, height, resizable, debug);
        webview_set_userdata(w, userdata);
        webview_navigate(w, url);
        return w;
    }

    void *wrapper_webview_get_userdata(webview_t w)
    {
        return webview_get_userdata(w);
    }

    void webview_exit(webview_t w)
    {
        webview_terminate(w);
    }

#ifdef __cplusplus
}
#endif