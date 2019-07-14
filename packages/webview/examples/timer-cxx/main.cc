#include <chrono>
#include <iomanip>
#include <mutex>
#include <sstream>
#include <thread>

#include <cstdio>
#include <cstring>

#define WEBVIEW_IMPLEMENTATION
#include <webview.h>

class Timer {
public:
  int get() {
    this->mutex.lock();
    int n = this->ticks;
    this->mutex.unlock();
    return n;
  }
  void set(int ticks) {
    this->mutex.lock();
    this->ticks = ticks;
    this->mutex.unlock();
  }
  void incr(int n = 1) {
    this->mutex.lock();
    this->ticks = this->ticks + n;
    this->mutex.unlock();
  }
  void start(struct webview *w) {
    this->thread = std::thread(&Timer::run, this, w);
    this->thread.detach();
  }
  void render(struct webview *w) {
    auto n = this->get();
    std::ostringstream jscode;
    jscode << "updateTicks(" << n << ")";
    webview_eval(w, jscode.str().c_str());
  }

private:
  void run(struct webview *w) {
    for (;;) {
      std::this_thread::sleep_for(std::chrono::microseconds(100000));
      this->incr();
      webview_dispatch(w,
                       [](struct webview *w, void *arg) {
                         Timer *timer = static_cast<Timer *>(arg);
                         timer->render(w);
                       },
                       this);
    }
  }
  std::thread thread;
  std::mutex mutex;
  int ticks = 0;
  struct webview *w;
};

static std::string url_encode(const std::string &value) {
  const char hex[] = "0123456789ABCDEF";
  std::string escaped;
  for (char c : value) {
    if (isalnum(c) || c == '-' || c == '_' || c == '.' || c == '~' ||
        c == '=') {
      escaped = escaped + c;
    } else {
      escaped = escaped + '%' + hex[(c >> 4) & 0xf] + hex[c & 0xf];
    }
  }
  return escaped;
}

static const char *html = R"html(
<!doctype html>
<html>
<body>
  <p id="ticks"></p>
  <button onclick="external.invoke('reset')">reset</button>
  <button onclick="external.invoke('exit')">exit</button>
  <script type="text/javascript">
    function updateTicks(n) {
      document.getElementById('ticks').innerText = 'ticks ' + n;
    }
  </script>
</body>
</html>
)html";

static void timer_cb(struct webview *w, const char *arg) {
  Timer *timer = static_cast<Timer *>(w->userdata);
  if (strcmp(arg, "reset") == 0) {
    timer->set(0);
    timer->render(w);
  } else if (strcmp(arg, "exit") == 0) {
    webview_terminate(w);
  }
}

#ifdef WIN32
int WINAPI WinMain(HINSTANCE hInt, HINSTANCE hPrevInst, LPSTR lpCmdLine,
                   int nCmdShow) {
#else
int main() {
#endif
  Timer timer;
  struct webview webview = {};
  std::string html_data = "data:text/html," + url_encode(html);

  webview.title = "Timer";
  webview.url = html_data.c_str();
  webview.width = 400;
  webview.height = 300;
  webview.resizable = 0;
  webview.external_invoke_cb = timer_cb;
  webview.userdata = &timer;

  webview_init(&webview);
  timer.start(&webview);
  while (webview_loop(&webview, 1) == 0)
    ;
  webview_exit(&webview);
  return 0;
}
