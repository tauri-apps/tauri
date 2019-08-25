#include "tauri-cocoa-webview.h"
#include "tauri.h"

#define NSAlertStyleWarning 0
#define NSAlertStyleCritical 2
#define NSWindowStyleMaskResizable 8
#define NSWindowStyleMaskMiniaturizable 4
#define NSWindowStyleMaskTitled 1
#define NSWindowStyleMaskClosable 2
#define NSWindowStyleMaskFullScreen (1 << 14)
#define NSViewWidthSizable 2
#define NSViewHeightSizable 16
#define NSBackingStoreBuffered 2
#define NSEventMaskAny ULONG_MAX
#define NSEventModifierFlagCommand (1 << 20)
#define NSEventModifierFlagOption (1 << 19)
#define NSAlertStyleInformational 1
#define NSAlertFirstButtonReturn 1000
#define WKNavigationActionPolicyDownload 2
#define NSModalResponseOK 1
#define WKNavigationActionPolicyDownload 2
#define WKNavigationResponsePolicyAllow 1
#define WKUserScriptInjectionTimeAtDocumentStart 0
#define NSApplicationActivationPolicyRegular 0

static id get_nsstring(const char *c_str) {
  return objc_msgSend((id)objc_getClass("NSString"),
                      sel_registerName("stringWithUTF8String:"), c_str);
}

static id create_menu_item(id title, const char *action, const char *key) {
  id item =
      objc_msgSend((id)objc_getClass("NSMenuItem"), sel_registerName("alloc"));
  objc_msgSend(item, sel_registerName("initWithTitle:action:keyEquivalent:"),
               title, sel_registerName(action), get_nsstring(key));
  objc_msgSend(item, sel_registerName("autorelease"));

  return item;
}

static void webview_window_will_close(id self, SEL cmd, id notification) {
  struct webview *w =
      (struct webview *)objc_getAssociatedObject(self, "webview");
  webview_terminate(w);
}

static void webview_external_invoke(id self, SEL cmd, id contentController,
                                    id message) {
  struct webview *w =
      (struct webview *)objc_getAssociatedObject(contentController, "webview");
  if (w == NULL || w->external_invoke_cb == NULL) {
    return;
  }

  w->external_invoke_cb(w, (const char *)objc_msgSend(
                               objc_msgSend(message, sel_registerName("body")),
                               sel_registerName("UTF8String")));
}

static void run_open_panel(id self, SEL cmd, id webView, id parameters,
                           id frame, void (^completionHandler)(id)) {

  id openPanel = objc_msgSend((id)objc_getClass("NSOpenPanel"),
                              sel_registerName("openPanel"));

  objc_msgSend(
      openPanel, sel_registerName("setAllowsMultipleSelection:"),
      objc_msgSend(parameters, sel_registerName("allowsMultipleSelection")));

  objc_msgSend(openPanel, sel_registerName("setCanChooseFiles:"), 1);
  objc_msgSend(
      openPanel, sel_registerName("beginWithCompletionHandler:"), ^(id result) {
        if (result == (id)NSModalResponseOK) {
          completionHandler(objc_msgSend(openPanel, sel_registerName("URLs")));
        } else {
          completionHandler(nil);
        }
      });
}

static void run_save_panel(id self, SEL cmd, id download, id filename,
                           void (^completionHandler)(int allowOverwrite,
                                                     id destination)) {
  id savePanel = objc_msgSend((id)objc_getClass("NSSavePanel"),
                              sel_registerName("savePanel"));
  objc_msgSend(savePanel, sel_registerName("setCanCreateDirectories:"), 1);
  objc_msgSend(savePanel, sel_registerName("setNameFieldStringValue:"),
               filename);
  objc_msgSend(savePanel, sel_registerName("beginWithCompletionHandler:"),
               ^(id result) {
                 if (result == (id)NSModalResponseOK) {
                   id url = objc_msgSend(savePanel, sel_registerName("URL"));
                   id path = objc_msgSend(url, sel_registerName("path"));
                   completionHandler(1, path);
                 } else {
                   completionHandler(NO, nil);
                 }
               });
}

static void run_confirmation_panel(id self, SEL cmd, id webView, id message,
                                   id frame, void (^completionHandler)(bool)) {

  id alert =
      objc_msgSend((id)objc_getClass("NSAlert"), sel_registerName("new"));
  objc_msgSend(alert, sel_registerName("setIcon:"),
               objc_msgSend((id)objc_getClass("NSImage"),
                            sel_registerName("imageNamed:"),
                            get_nsstring("NSCaution")));
  objc_msgSend(alert, sel_registerName("setShowsHelp:"), 0);
  objc_msgSend(alert, sel_registerName("setInformativeText:"), message);
  objc_msgSend(alert, sel_registerName("addButtonWithTitle:"),
               get_nsstring("OK"));
  objc_msgSend(alert, sel_registerName("addButtonWithTitle:"),
               get_nsstring("Cancel"));
  if (objc_msgSend(alert, sel_registerName("runModal")) ==
      (id)NSAlertFirstButtonReturn) {
    completionHandler(true);
  } else {
    completionHandler(false);
  }
  objc_msgSend(alert, sel_registerName("release"));
}

static void run_alert_panel(id self, SEL cmd, id webView, id message, id frame,
                            void (^completionHandler)(void)) {
  id alert =
      objc_msgSend((id)objc_getClass("NSAlert"), sel_registerName("new"));
  objc_msgSend(alert, sel_registerName("setIcon:"),
               objc_msgSend((id)objc_getClass("NSImage"),
                            sel_registerName("imageNamed:"),
                            get_nsstring("NSCaution")));
  objc_msgSend(alert, sel_registerName("setShowsHelp:"), 0);
  objc_msgSend(alert, sel_registerName("setInformativeText:"), message);
  objc_msgSend(alert, sel_registerName("addButtonWithTitle:"),
               get_nsstring("OK"));
  objc_msgSend(alert, sel_registerName("runModal"));
  objc_msgSend(alert, sel_registerName("release"));
  completionHandler();
}

static void download_failed(id self, SEL cmd, id download, id error) {
  printf("%s",
         (const char *)objc_msgSend(
             objc_msgSend(error, sel_registerName("localizedDescription")),
             sel_registerName("UTF8String")));
}

static void make_nav_policy_decision(id self, SEL cmd, id webView, id response,
                                     void (^decisionHandler)(int)) {
  if (objc_msgSend(response, sel_registerName("canShowMIMEType")) == 0) {
    decisionHandler(WKNavigationActionPolicyDownload);
  } else {
    decisionHandler(WKNavigationResponsePolicyAllow);
  }
}

WEBVIEW_API int webview_init(struct webview *w) {
  w->priv.pool = objc_msgSend((id)objc_getClass("NSAutoreleasePool"),
                              sel_registerName("new"));
  objc_msgSend((id)objc_getClass("NSApplication"),
               sel_registerName("sharedApplication"));

  Class __WKScriptMessageHandler = objc_allocateClassPair(
      objc_getClass("NSObject"), "__WKScriptMessageHandler", 0);
  class_addMethod(
      __WKScriptMessageHandler,
      sel_registerName("userContentController:didReceiveScriptMessage:"),
      (IMP)webview_external_invoke, "v@:@@");
  objc_registerClassPair(__WKScriptMessageHandler);

  id scriptMessageHandler =
      objc_msgSend((id)__WKScriptMessageHandler, sel_registerName("new"));

  /***
   _WKDownloadDelegate is an undocumented/private protocol with methods called
   from WKNavigationDelegate
   References:
   https://github.com/WebKit/webkit/blob/master/Source/WebKit/UIProcess/API/Cocoa/_WKDownload.h
   https://github.com/WebKit/webkit/blob/master/Source/WebKit/UIProcess/API/Cocoa/_WKDownloadDelegate.h
   https://github.com/WebKit/webkit/blob/master/Tools/TestWebKitAPI/Tests/WebKitCocoa/Download.mm
   ***/

  Class __WKDownloadDelegate = objc_allocateClassPair(
      objc_getClass("NSObject"), "__WKDownloadDelegate", 0);
  class_addMethod(
      __WKDownloadDelegate,
      sel_registerName("_download:decideDestinationWithSuggestedFilename:"
                       "completionHandler:"),
      (IMP)run_save_panel, "v@:@@?");
  class_addMethod(__WKDownloadDelegate,
                  sel_registerName("_download:didFailWithError:"),
                  (IMP)download_failed, "v@:@@");
  objc_registerClassPair(__WKDownloadDelegate);
  id downloadDelegate =
      objc_msgSend((id)__WKDownloadDelegate, sel_registerName("new"));

  Class __WKPreferences = objc_allocateClassPair(objc_getClass("WKPreferences"),
                                                 "__WKPreferences", 0);
  objc_property_attribute_t type = {"T", "c"};
  objc_property_attribute_t ownership = {"N", ""};
  objc_property_attribute_t attrs[] = {type, ownership};
  class_replaceProperty(__WKPreferences, "developerExtrasEnabled", attrs, 2);
  objc_registerClassPair(__WKPreferences);
  id wkPref = objc_msgSend((id)__WKPreferences, sel_registerName("new"));
  objc_msgSend(wkPref, sel_registerName("setValue:forKey:"),
               objc_msgSend((id)objc_getClass("NSNumber"),
                            sel_registerName("numberWithBool:"), !!w->debug),
               objc_msgSend((id)objc_getClass("NSString"),
                            sel_registerName("stringWithUTF8String:"),
                            "developerExtrasEnabled"));

  id userController = objc_msgSend((id)objc_getClass("WKUserContentController"),
                                   sel_registerName("new"));
  objc_setAssociatedObject(userController, "webview", (id)(w),
                           OBJC_ASSOCIATION_ASSIGN);
  objc_msgSend(
      userController, sel_registerName("addScriptMessageHandler:name:"),
      scriptMessageHandler,
      objc_msgSend((id)objc_getClass("NSString"),
                   sel_registerName("stringWithUTF8String:"), "invoke"));

  /***
   In order to maintain compatibility with the other 'webviews' we need to
   override window.external.invoke to call
   webkit.messageHandlers.invoke.postMessage
   ***/

  id windowExternalOverrideScript = objc_msgSend(
      (id)objc_getClass("WKUserScript"), sel_registerName("alloc"));
  objc_msgSend(
      windowExternalOverrideScript,
      sel_registerName("initWithSource:injectionTime:forMainFrameOnly:"),
      get_nsstring("window.external = this; invoke = function(arg){ "
                   "webkit.messageHandlers.invoke.postMessage(arg); };"),
      WKUserScriptInjectionTimeAtDocumentStart, 0);

  objc_msgSend(userController, sel_registerName("addUserScript:"),
               windowExternalOverrideScript);

  id config = objc_msgSend((id)objc_getClass("WKWebViewConfiguration"),
                           sel_registerName("new"));
  id processPool = objc_msgSend(config, sel_registerName("processPool"));
  objc_msgSend(processPool, sel_registerName("_setDownloadDelegate:"),
               downloadDelegate);
  objc_msgSend(config, sel_registerName("setProcessPool:"), processPool);
  objc_msgSend(config, sel_registerName("setUserContentController:"),
               userController);
  objc_msgSend(config, sel_registerName("setPreferences:"), wkPref);

  Class __NSWindowDelegate = objc_allocateClassPair(objc_getClass("NSObject"),
                                                    "__NSWindowDelegate", 0);
  class_addProtocol(__NSWindowDelegate, objc_getProtocol("NSWindowDelegate"));
  class_replaceMethod(__NSWindowDelegate, sel_registerName("windowWillClose:"),
                      (IMP)webview_window_will_close, "v@:@");
  objc_registerClassPair(__NSWindowDelegate);

  w->priv.windowDelegate =
      objc_msgSend((id)__NSWindowDelegate, sel_registerName("new"));

  objc_setAssociatedObject(w->priv.windowDelegate, "webview", (id)(w),
                           OBJC_ASSOCIATION_ASSIGN);

  id nsTitle =
      objc_msgSend((id)objc_getClass("NSString"),
                   sel_registerName("stringWithUTF8String:"), w->title);

  CGRect r = CGRectMake(0, 0, w->width, w->height);

  unsigned int style = NSWindowStyleMaskTitled | NSWindowStyleMaskClosable |
                       NSWindowStyleMaskMiniaturizable;
  if (w->resizable) {
    style = style | NSWindowStyleMaskResizable;
  }

  w->priv.window =
      objc_msgSend((id)objc_getClass("NSWindow"), sel_registerName("alloc"));
  objc_msgSend(w->priv.window,
               sel_registerName("initWithContentRect:styleMask:backing:defer:"),
               r, style, NSBackingStoreBuffered, 0);

  objc_msgSend(w->priv.window, sel_registerName("autorelease"));
  objc_msgSend(w->priv.window, sel_registerName("setTitle:"), nsTitle);
  objc_msgSend(w->priv.window, sel_registerName("setDelegate:"),
               w->priv.windowDelegate);
  objc_msgSend(w->priv.window, sel_registerName("center"));

  Class __WKUIDelegate =
      objc_allocateClassPair(objc_getClass("NSObject"), "__WKUIDelegate", 0);
  class_addProtocol(__WKUIDelegate, objc_getProtocol("WKUIDelegate"));
  class_addMethod(__WKUIDelegate,
                  sel_registerName("webView:runOpenPanelWithParameters:"
                                   "initiatedByFrame:completionHandler:"),
                  (IMP)run_open_panel, "v@:@@@?");
  class_addMethod(__WKUIDelegate,
                  sel_registerName("webView:runJavaScriptAlertPanelWithMessage:"
                                   "initiatedByFrame:completionHandler:"),
                  (IMP)run_alert_panel, "v@:@@@?");
  class_addMethod(
      __WKUIDelegate,
      sel_registerName("webView:runJavaScriptConfirmPanelWithMessage:"
                       "initiatedByFrame:completionHandler:"),
      (IMP)run_confirmation_panel, "v@:@@@?");
  objc_registerClassPair(__WKUIDelegate);
  id uiDel = objc_msgSend((id)__WKUIDelegate, sel_registerName("new"));

  Class __WKNavigationDelegate = objc_allocateClassPair(
      objc_getClass("NSObject"), "__WKNavigationDelegate", 0);
  class_addProtocol(__WKNavigationDelegate,
                    objc_getProtocol("WKNavigationDelegate"));
  class_addMethod(
      __WKNavigationDelegate,
      sel_registerName(
          "webView:decidePolicyForNavigationResponse:decisionHandler:"),
      (IMP)make_nav_policy_decision, "v@:@@?");
  objc_registerClassPair(__WKNavigationDelegate);
  id navDel = objc_msgSend((id)__WKNavigationDelegate, sel_registerName("new"));

  w->priv.webview =
      objc_msgSend((id)objc_getClass("WKWebView"), sel_registerName("alloc"));
  objc_msgSend(w->priv.webview,
               sel_registerName("initWithFrame:configuration:"), r, config);
  objc_msgSend(w->priv.webview, sel_registerName("setUIDelegate:"), uiDel);
  objc_msgSend(w->priv.webview, sel_registerName("setNavigationDelegate:"),
               navDel);

  id nsURL = objc_msgSend((id)objc_getClass("NSURL"),
                          sel_registerName("URLWithString:"),
                          get_nsstring(webview_check_url(w->url)));

  objc_msgSend(w->priv.webview, sel_registerName("loadRequest:"),
               objc_msgSend((id)objc_getClass("NSURLRequest"),
                            sel_registerName("requestWithURL:"), nsURL));
  objc_msgSend(w->priv.webview, sel_registerName("setAutoresizesSubviews:"), 1);
  objc_msgSend(w->priv.webview, sel_registerName("setAutoresizingMask:"),
               (NSViewWidthSizable | NSViewHeightSizable));
  objc_msgSend(objc_msgSend(w->priv.window, sel_registerName("contentView")),
               sel_registerName("addSubview:"), w->priv.webview);
  objc_msgSend(w->priv.window, sel_registerName("orderFrontRegardless"));

  objc_msgSend(objc_msgSend((id)objc_getClass("NSApplication"),
                            sel_registerName("sharedApplication")),
               sel_registerName("setActivationPolicy:"),
               NSApplicationActivationPolicyRegular);

  objc_msgSend(objc_msgSend((id)objc_getClass("NSApplication"),
                            sel_registerName("sharedApplication")),
               sel_registerName("finishLaunching"));

  objc_msgSend(objc_msgSend((id)objc_getClass("NSApplication"),
                            sel_registerName("sharedApplication")),
               sel_registerName("activateIgnoringOtherApps:"), 1);

  id menubar =
      objc_msgSend((id)objc_getClass("NSMenu"), sel_registerName("alloc"));
  objc_msgSend(menubar, sel_registerName("initWithTitle:"), get_nsstring(""));
  objc_msgSend(menubar, sel_registerName("autorelease"));

  id appName = objc_msgSend(objc_msgSend((id)objc_getClass("NSProcessInfo"),
                                         sel_registerName("processInfo")),
                            sel_registerName("processName"));

  id appMenuItem =
      objc_msgSend((id)objc_getClass("NSMenuItem"), sel_registerName("alloc"));
  objc_msgSend(appMenuItem,
               sel_registerName("initWithTitle:action:keyEquivalent:"), appName,
               NULL, get_nsstring(""));

  id appMenu =
      objc_msgSend((id)objc_getClass("NSMenu"), sel_registerName("alloc"));
  objc_msgSend(appMenu, sel_registerName("initWithTitle:"), appName);
  objc_msgSend(appMenu, sel_registerName("autorelease"));

  objc_msgSend(appMenuItem, sel_registerName("setSubmenu:"), appMenu);
  objc_msgSend(menubar, sel_registerName("addItem:"), appMenuItem);

  id title =
      objc_msgSend(get_nsstring("Hide "),
                   sel_registerName("stringByAppendingString:"), appName);
  id item = create_menu_item(title, "hide:", "h");
  objc_msgSend(appMenu, sel_registerName("addItem:"), item);

  item = create_menu_item(get_nsstring("Hide Others"),
                          "hideOtherApplications:", "h");
  objc_msgSend(item, sel_registerName("setKeyEquivalentModifierMask:"),
               (NSEventModifierFlagOption | NSEventModifierFlagCommand));
  objc_msgSend(appMenu, sel_registerName("addItem:"), item);

  item =
      create_menu_item(get_nsstring("Show All"), "unhideAllApplications:", "");
  objc_msgSend(appMenu, sel_registerName("addItem:"), item);

  objc_msgSend(appMenu, sel_registerName("addItem:"),
               objc_msgSend((id)objc_getClass("NSMenuItem"),
                            sel_registerName("separatorItem")));

  title = objc_msgSend(get_nsstring("Quit "),
                       sel_registerName("stringByAppendingString:"), appName);
  item = create_menu_item(title, "terminate:", "q");
  objc_msgSend(appMenu, sel_registerName("addItem:"), item);

id editMenuItem =
      objc_msgSend((id)objc_getClass("NSMenuItem"), sel_registerName("alloc"));
  objc_msgSend(editMenuItem,
               sel_registerName("initWithTitle:action:keyEquivalent:"), get_nsstring("Edit"),
               NULL, get_nsstring(""));

  /***
   Edit menu
  ***/

id editMenu =
  objc_msgSend((id)objc_getClass("NSMenu"), sel_registerName("alloc"));
  objc_msgSend(editMenu, sel_registerName("initWithTitle:"), get_nsstring("Edit"));
  objc_msgSend(editMenu, sel_registerName("autorelease"));

  objc_msgSend(editMenuItem, sel_registerName("setSubmenu:"), editMenu);
  objc_msgSend(menubar, sel_registerName("addItem:"), editMenuItem);

  item = create_menu_item(get_nsstring("Undo"), "undo:", "z");
  objc_msgSend(editMenu, sel_registerName("addItem:"), item);

  item = create_menu_item(get_nsstring("Redo"), "redo:", "y");
  objc_msgSend(editMenu, sel_registerName("addItem:"), item);

  item = objc_msgSend((id)objc_getClass("NSMenuItem"), sel_registerName("separatorItem"));
  objc_msgSend(editMenu, sel_registerName("addItem:"), item);

  item = create_menu_item(get_nsstring("Cut"), "cut:", "x");
  objc_msgSend(editMenu, sel_registerName("addItem:"), item);

  item = create_menu_item(get_nsstring("Copy"), "copy:", "c");
  objc_msgSend(editMenu, sel_registerName("addItem:"), item);

  item = create_menu_item(get_nsstring("Paste"), "paste:", "v");
  objc_msgSend(editMenu, sel_registerName("addItem:"), item);

  item = create_menu_item(get_nsstring("Select All"), "selectAll:", "a");
  objc_msgSend(editMenu, sel_registerName("addItem:"), item);

  /***
   Finalize menubar
  ***/

  objc_msgSend(objc_msgSend((id)objc_getClass("NSApplication"),
                            sel_registerName("sharedApplication")),
               sel_registerName("setMainMenu:"), menubar);

  w->priv.should_exit = 0;
  return 0;
}

WEBVIEW_API int webview_loop(struct webview *w, int blocking) {
  id until = (blocking ? objc_msgSend((id)objc_getClass("NSDate"),
                                      sel_registerName("distantFuture"))
                       : objc_msgSend((id)objc_getClass("NSDate"),
                                      sel_registerName("distantPast")));

  id event = objc_msgSend(
      objc_msgSend((id)objc_getClass("NSApplication"),
                   sel_registerName("sharedApplication")),
      sel_registerName("nextEventMatchingMask:untilDate:inMode:dequeue:"),
      ULONG_MAX, until,
      objc_msgSend((id)objc_getClass("NSString"),
                   sel_registerName("stringWithUTF8String:"),
                   "kCFRunLoopDefaultMode"),
      true);

  if (event) {
    objc_msgSend(objc_msgSend((id)objc_getClass("NSApplication"),
                              sel_registerName("sharedApplication")),
                 sel_registerName("sendEvent:"), event);
  }

  return w->priv.should_exit;
}

WEBVIEW_API int webview_eval(struct webview *w, const char *js) {
  objc_msgSend(w->priv.webview,
               sel_registerName("evaluateJavaScript:completionHandler:"),
               get_nsstring(js), NULL);

  return 0;
}

WEBVIEW_API void webview_set_title(struct webview *w, const char *title) {
  objc_msgSend(w->priv.window, sel_registerName("setTitle"),
               get_nsstring(title));
}

WEBVIEW_API void webview_set_fullscreen(struct webview *w, int fullscreen) {
  unsigned long windowStyleMask = (unsigned long)objc_msgSend(
      w->priv.window, sel_registerName("styleMask"));
  int b = (((windowStyleMask & NSWindowStyleMaskFullScreen) ==
            NSWindowStyleMaskFullScreen)
               ? 1
               : 0);
  if (b != fullscreen) {
    objc_msgSend(w->priv.window, sel_registerName("toggleFullScreen:"), NULL);
  }
}

WEBVIEW_API void webview_set_color(struct webview *w, uint8_t r, uint8_t g,
                                   uint8_t b, uint8_t a) {

  id color = objc_msgSend((id)objc_getClass("NSColor"),
                          sel_registerName("colorWithRed:green:blue:alpha:"),
                          (float)r / 255.0, (float)g / 255.0, (float)b / 255.0,
                          (float)a / 255.0);

  objc_msgSend(w->priv.window, sel_registerName("setBackgroundColor:"), color);

  if (0.5 >= ((r / 255.0 * 299.0) + (g / 255.0 * 587.0) + (b / 255.0 * 114.0)) /
                 1000.0) {
    objc_msgSend(w->priv.window, sel_registerName("setAppearance:"),
                 objc_msgSend((id)objc_getClass("NSAppearance"),
                              sel_registerName("appearanceNamed:"),
                              get_nsstring("NSAppearanceNameVibrantDark")));
  } else {
    objc_msgSend(w->priv.window, sel_registerName("setAppearance:"),
                 objc_msgSend((id)objc_getClass("NSAppearance"),
                              sel_registerName("appearanceNamed:"),
                              get_nsstring("NSAppearanceNameVibrantLight")));
  }
  objc_msgSend(w->priv.window, sel_registerName("setOpaque:"), 0);
  objc_msgSend(w->priv.window,
               sel_registerName("setTitlebarAppearsTransparent:"), 1);
               objc_msgSend(w->priv.webview, sel_registerName("_setDrawsBackground:"), 0);
}

WEBVIEW_API void webview_dialog(struct webview *w,
                                enum webview_dialog_type dlgtype, int flags,
                                const char *title, const char *arg,
                                char *result, size_t resultsz) {
  if (dlgtype == WEBVIEW_DIALOG_TYPE_OPEN ||
      dlgtype == WEBVIEW_DIALOG_TYPE_SAVE) {
    id panel = (id)objc_getClass("NSSavePanel");
    if (dlgtype == WEBVIEW_DIALOG_TYPE_OPEN) {
      id openPanel = objc_msgSend((id)objc_getClass("NSOpenPanel"),
                                  sel_registerName("openPanel"));
      if (flags & WEBVIEW_DIALOG_FLAG_DIRECTORY) {
        objc_msgSend(openPanel, sel_registerName("setCanChooseFiles:"), 0);
        objc_msgSend(openPanel, sel_registerName("setCanChooseDirectories:"),
                     1);
      } else {
        objc_msgSend(openPanel, sel_registerName("setCanChooseFiles:"), 1);
        objc_msgSend(openPanel, sel_registerName("setCanChooseDirectories:"),
                     0);
      }
      objc_msgSend(openPanel, sel_registerName("setResolvesAliases:"), 0);
      objc_msgSend(openPanel, sel_registerName("setAllowsMultipleSelection:"),
                   0);
      panel = openPanel;
    } else {
      panel = objc_msgSend((id)objc_getClass("NSSavePanel"),
                           sel_registerName("savePanel"));
    }

    objc_msgSend(panel, sel_registerName("setCanCreateDirectories:"), 1);
    objc_msgSend(panel, sel_registerName("setShowsHiddenFiles:"), 1);
    objc_msgSend(panel, sel_registerName("setExtensionHidden:"), 0);
    objc_msgSend(panel, sel_registerName("setCanSelectHiddenExtension:"), 0);
    objc_msgSend(panel, sel_registerName("setTreatsFilePackagesAsDirectories:"),
                 1);
    objc_msgSend(
        panel, sel_registerName("beginSheetModalForWindow:completionHandler:"),
        w->priv.window, ^(id result) {
          objc_msgSend(objc_msgSend((id)objc_getClass("NSApplication"),
                                    sel_registerName("sharedApplication")),
                       sel_registerName("stopModalWithCode:"), result);
        });

    if (objc_msgSend(objc_msgSend((id)objc_getClass("NSApplication"),
                                  sel_registerName("sharedApplication")),
                     sel_registerName("runModalForWindow:"),
                     panel) == (id)NSModalResponseOK) {
      id url = objc_msgSend(panel, sel_registerName("URL"));
      id path = objc_msgSend(url, sel_registerName("path"));
      const char *filename =
          (const char *)objc_msgSend(path, sel_registerName("UTF8String"));
      strlcpy(result, filename, resultsz);
    }
  } else if (dlgtype == WEBVIEW_DIALOG_TYPE_ALERT) {
    id a = objc_msgSend((id)objc_getClass("NSAlert"), sel_registerName("new"));
    switch (flags & WEBVIEW_DIALOG_FLAG_ALERT_MASK) {
    case WEBVIEW_DIALOG_FLAG_INFO:
      objc_msgSend(a, sel_registerName("setAlertStyle:"),
                   NSAlertStyleInformational);
      break;
    case WEBVIEW_DIALOG_FLAG_WARNING:
      printf("Warning\n");
      objc_msgSend(a, sel_registerName("setAlertStyle:"), NSAlertStyleWarning);
      break;
    case WEBVIEW_DIALOG_FLAG_ERROR:
      printf("Error\n");
      objc_msgSend(a, sel_registerName("setAlertStyle:"), NSAlertStyleCritical);
      break;
    }
    objc_msgSend(a, sel_registerName("setShowsHelp:"), 0);
    objc_msgSend(a, sel_registerName("setShowsSuppressionButton:"), 0);
    objc_msgSend(a, sel_registerName("setMessageText:"), get_nsstring(title));
    objc_msgSend(a, sel_registerName("setInformativeText:"), get_nsstring(arg));
    objc_msgSend(a, sel_registerName("addButtonWithTitle:"),
                 get_nsstring("OK"));
    objc_msgSend(a, sel_registerName("runModal"));
    objc_msgSend(a, sel_registerName("release"));
  }
}

static void webview_dispatch_cb(void *arg) {
  struct webview_dispatch_arg *context = (struct webview_dispatch_arg *)arg;
  (context->fn)(context->w, context->arg);
  free(context);
}

WEBVIEW_API void webview_dispatch(struct webview *w, webview_dispatch_fn fn,
                                  void *arg) {
  struct webview_dispatch_arg *context = (struct webview_dispatch_arg *)malloc(
      sizeof(struct webview_dispatch_arg));
  context->w = w;
  context->arg = arg;
  context->fn = fn;
  dispatch_async_f(dispatch_get_main_queue(), context, webview_dispatch_cb);
}

WEBVIEW_API void webview_terminate(struct webview *w) {
  w->priv.should_exit = 1;
}

WEBVIEW_API void webview_exit(struct webview *w) {
  id app = objc_msgSend((id)objc_getClass("NSApplication"),
                        sel_registerName("sharedApplication"));
  objc_msgSend(app, sel_registerName("terminate:"), app);
}

WEBVIEW_API void webview_print_log(const char *s) { printf("%s\n", s); }
