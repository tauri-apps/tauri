#include <objc/objc-runtime.h>
#include <CoreGraphics/CoreGraphics.h>
#include <limits.h>

struct webview_priv {
  id pool;
  id window;
  id webview;
  id windowDelegate;
  int should_exit;
};
