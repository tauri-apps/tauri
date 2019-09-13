/*
 * MIT License
 *
 * Copyright (c) 2017 Serge Zaitsev
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
#ifndef WEBVIEW_H
#define WEBVIEW_H

#ifndef WEBVIEW_API
#define WEBVIEW_API extern
#endif

#ifdef __cplusplus
extern "C"
{
#endif

    typedef void *webview_t;

    // Create a new webview instance
    WEBVIEW_API webview_t webview_create(int debug, void *wnd);

    // Destroy a webview
    WEBVIEW_API void webview_destroy(webview_t w);

    // Run the main loop
    WEBVIEW_API void webview_run(webview_t w);

    // Stop the main loop
    WEBVIEW_API void webview_terminate(webview_t w);

    // Post a function to be executed on the main thread
    WEBVIEW_API void
    webview_dispatch(webview_t w, void (*fn)(webview_t w, void *arg), void *arg);

    WEBVIEW_API void *webview_get_window(webview_t w);

    WEBVIEW_API void webview_set_title(webview_t w, const char *title);

    WEBVIEW_API void webview_set_bounds(webview_t w, int x, int y, int width,
                                        int height, int flags);
    WEBVIEW_API void webview_get_bounds(webview_t w, int *x, int *y, int *width,
                                        int *height, int *flags);

    WEBVIEW_API void webview_navigate(webview_t w, const char *url);
    WEBVIEW_API void webview_init(webview_t w, const char *js);
    WEBVIEW_API void webview_eval(webview_t w, const char *js);

#ifdef __cplusplus
}
#endif

#ifndef WEBVIEW_HEADER

#include <atomic>
#include <functional>
#include <future>
#include <map>
#include <string>
#include <vector>

#include <cstring>

namespace webview
{
using dispatch_fn_t = std::function<void()>;
using msg_cb_t = std::function<void(const char *msg)>;

inline std::string url_encode(std::string s)
{
    std::string encoded;
    for (unsigned int i = 0; i < s.length(); i++)
    {
        auto c = s[i];
        if (isalnum(c) || c == '-' || c == '_' || c == '.' || c == '~')
        {
            encoded = encoded + c;
        }
        else
        {
            char hex[4];
            snprintf(hex, sizeof(hex), "%%%02x", c);
            encoded = encoded + hex;
        }
    }
    return encoded;
}

inline std::string url_decode(std::string s)
{
    std::string decoded;
    for (unsigned int i = 0; i < s.length(); i++)
    {
        if (s[i] == '%')
        {
            int n;
            sscanf(s.substr(i + 1, 2).c_str(), "%x", &n);
            decoded = decoded + static_cast<char>(n);
            i = i + 2;
        }
        else if (s[i] == '+')
        {
            decoded = decoded + ' ';
        }
        else
        {
            decoded = decoded + s[i];
        }
    }
    return decoded;
}

inline std::string html_from_uri(std::string s)
{
    if (s.substr(0, 15) == "data:text/html,")
    {
        return url_decode(s.substr(15));
    }
    return "";
}

inline int json_parse_c(const char *s, size_t sz, const char *key, size_t keysz,
                        const char **value, size_t *valuesz)
{
    enum
    {
        JSON_STATE_VALUE,
        JSON_STATE_LITERAL,
        JSON_STATE_STRING,
        JSON_STATE_ESCAPE,
        JSON_STATE_UTF8
    } state = JSON_STATE_VALUE;
    const char *k = NULL;
    int index = 1;
    int depth = 0;
    int utf8_bytes = 0;

    if (key == NULL)
    {
        index = keysz;
        keysz = 0;
    }

    *value = NULL;
    *valuesz = 0;

    for (; sz > 0; s++, sz--)
    {
        enum
        {
            JSON_ACTION_NONE,
            JSON_ACTION_START,
            JSON_ACTION_END,
            JSON_ACTION_START_STRUCT,
            JSON_ACTION_END_STRUCT
        } action = JSON_ACTION_NONE;
        unsigned char c = *s;
        switch (state)
        {
        case JSON_STATE_VALUE:
            if (c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == ',' ||
                c == ':')
            {
                continue;
            }
            else if (c == '"')
            {
                action = JSON_ACTION_START;
                state = JSON_STATE_STRING;
            }
            else if (c == '{' || c == '[')
            {
                action = JSON_ACTION_START_STRUCT;
            }
            else if (c == '}' || c == ']')
            {
                action = JSON_ACTION_END_STRUCT;
            }
            else if (c == 't' || c == 'f' || c == 'n' || c == '-' ||
                     (c >= '0' && c <= '9'))
            {
                action = JSON_ACTION_START;
                state = JSON_STATE_LITERAL;
            }
            else
            {
                return -1;
            }
            break;
        case JSON_STATE_LITERAL:
            if (c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == ',' ||
                c == ']' || c == '}' || c == ':')
            {
                state = JSON_STATE_VALUE;
                s--;
                sz++;
                action = JSON_ACTION_END;
            }
            else if (c < 32 || c > 126)
            {
                return -1;
            } // fallthrough
        case JSON_STATE_STRING:
            if (c < 32 || (c > 126 && c < 192))
            {
                return -1;
            }
            else if (c == '"')
            {
                action = JSON_ACTION_END;
                state = JSON_STATE_VALUE;
            }
            else if (c == '\\')
            {
                state = JSON_STATE_ESCAPE;
            }
            else if (c >= 192 && c < 224)
            {
                utf8_bytes = 1;
                state = JSON_STATE_UTF8;
            }
            else if (c >= 224 && c < 240)
            {
                utf8_bytes = 2;
                state = JSON_STATE_UTF8;
            }
            else if (c >= 240 && c < 247)
            {
                utf8_bytes = 3;
                state = JSON_STATE_UTF8;
            }
            else if (c >= 128 && c < 192)
            {
                return -1;
            }
            break;
        case JSON_STATE_ESCAPE:
            if (c == '"' || c == '\\' || c == '/' || c == 'b' || c == 'f' ||
                c == 'n' || c == 'r' || c == 't' || c == 'u')
            {
                state = JSON_STATE_STRING;
            }
            else
            {
                return -1;
            }
            break;
        case JSON_STATE_UTF8:
            if (c < 128 || c > 191)
            {
                return -1;
            }
            utf8_bytes--;
            if (utf8_bytes == 0)
            {
                state = JSON_STATE_STRING;
            }
            break;
        default:
            return -1;
        }

        if (action == JSON_ACTION_END_STRUCT)
        {
            depth--;
        }

        if (depth == 1)
        {
            if (action == JSON_ACTION_START || action == JSON_ACTION_START_STRUCT)
            {
                if (index == 0)
                {
                    *value = s;
                }
                else if (keysz > 0 && index == 1)
                {
                    k = s;
                }
                else
                {
                    index--;
                }
            }
            else if (action == JSON_ACTION_END ||
                     action == JSON_ACTION_END_STRUCT)
            {
                if (*value != NULL && index == 0)
                {
                    *valuesz = (size_t)(s + 1 - *value);
                    return 0;
                }
                else if (keysz > 0 && k != NULL)
                {
                    if (keysz == (size_t)(s - k - 1) && memcmp(key, k + 1, keysz) == 0)
                    {
                        index = 0;
                    }
                    else
                    {
                        index = 2;
                    }
                    k = NULL;
                }
            }
        }

        if (action == JSON_ACTION_START_STRUCT)
        {
            depth++;
        }
    }
    return -1;
}

inline std::string json_escape(std::string s)
{
    std::string r = "\"";

    r.reserve(s.size() + 4);

    static const char *h = "0123456789abcdef";

    const unsigned char *d = reinterpret_cast<const unsigned char *>(s.data());

    for (size_t i = 0; i < s.size(); ++i)
    {
        switch (const auto c = d[i])
        {
        case '\b':
            r += "\\b";
            break;
        case '\f':
            r += "\\f";
            break;
        case '\n':
            r += "\\n";
            break;
        case '\r':
            r += "\\r";
            break;
        case '\t':
            r += "\\t";
            break;
        case '\\':
            r += "\\\\";
            break;
        case '\"':
            r += "\\\"";
            break;
        default:
            if ((c < 32) || (c == 127))
            {
                r += "\\u00";
                r += h[(c & 0xf0) >> 4];
                r += h[c & 0x0f];
                continue;
            }
            r += c; // Assume valid UTF-8.
            break;
        }
    }
    r += '"';
    return r;
}

inline int json_unescape(const char *s, size_t n, char *out)
{
    int r = 0;
    if (*s++ != '"')
    {
        return -1;
    }
    while (n > 2)
    {
        char c = *s;
        if (c == '\\')
        {
            s++;
            n--;
            switch (*s)
            {
            case 'b':
                c = '\b';
                break;
            case 'f':
                c = '\f';
                break;
            case 'n':
                c = '\n';
                break;
            case 'r':
                c = '\r';
                break;
            case 't':
                c = '\t';
                break;
            case '\\':
                c = '\\';
                break;
            case '/':
                c = '/';
                break;
            case '\"':
                c = '\"';
                break;
            default: // TODO: support unicode decoding
                return -1;
            }
        }
        if (out != NULL)
        {
            *out++ = c;
        }
        s++;
        n--;
        r++;
    }
    if (*s != '"')
    {
        return -1;
    }
    if (out != NULL)
    {
        *out = '\0';
    }
    return r;
}

inline std::string json_parse(std::string s, std::string key, int index)
{
    const char *value;
    size_t value_sz;
    if (key == "")
    {
        json_parse_c(s.c_str(), s.length(), nullptr, index, &value, &value_sz);
    }
    else
    {
        json_parse_c(s.c_str(), s.length(), key.c_str(), key.length(), &value,
                     &value_sz);
    }
    if (value != nullptr)
    {
        if (value[0] != '"')
        {
            return std::string(value, value_sz);
        }
        int n = json_unescape(value, value_sz, nullptr);
        if (n > 0)
        {
            char *decoded = new char[n];
            json_unescape(value, value_sz, decoded);
            auto result = std::string(decoded, n);
            delete[] decoded;
            return result;
        }
    }
    return "";
}

} // namespace webview

#endif /* WEBVIEW_HEADER */

#endif /* WEBVIEW_H */