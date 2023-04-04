// ignore_for_file: constant_identifier_names

part of tauri_window;

typedef DynamicEventCallback = EventCallback<dynamic>;
typedef EventCallbackList = List<DynamicEventCallback>;
typedef EventCallbackRegistry = Map<EventName, EventCallbackList>;
typedef ThemeChangedEventHandler = EventCallback<Theme>;
typedef OnFocusedEventHandler = EventCallback<bool>;
typedef ScaleFactorChangedEventHandler = EventCallback<ScaleFactorChanged>;
typedef MenuClickedEventHandler = EventCallback<String>;
typedef CloseRequestedEventHandler = FutureOr<void> Function(
  CloseRequestedEvent event,
);
typedef TauriIPCCallback = void Function(dynamic message);
typedef IPCPostMessageCallback = void Function(String args);
typedef WindowLabel = String;

enum Theme {
  light,
  dark;
}

enum TitleBarStyle {
  visible,
  transparent,
  overlay;
}

/// The file drop event types.
enum FileDropEventType {
  hover,
  drop,
  cancel;
}

enum WindowType {
  Logical,
  Physical;
}

/// Attention type to request on a window.
///
/// @since 1.0.0
enum UserAttentionType {
  /// #### Platform-specific
  ///
  ///   - **macOS:** Bounces the dock icon until the application is in focus.
  ///   - **Windows:** Flashes both the window and the taskbar button until the
  /// application is in focus.
  Critical(1),

  /// #### Platform-specific
  ///
  ///   - **macOS:** Bounces the dock icon once.
  ///   - **Windows:** Flashes the taskbar button until the application is in
  /// focus.
  Informational();

  const UserAttentionType([this.id]);

  final int? id;
}

enum CursorIcon {
  defaultIcon,
  crosshair,
  hand,
  arrow,
  move,
  text,
  wait,
  help,
  progress,

  /// something cannot be done
  notAllowed,
  contextMenu,
  cell,
  verticalText,
  alias,
  copy,
  noDrop,

  /// something can be grabbed
  grab,

  /// something is grabbed
  grabbing,
  allScroll,
  zoomIn,
  zoomOut,

  /// edge is to be moved
  eResize,
  nResize,
  neResize,
  nwResize,
  sResize,
  seResize,
  swResize,
  wResize,
  ewResize,
  nsResize,
  neswResize,
  nwseResize,
  colResize,
  rowResize;

  String get stringValue {
    switch (this) {
      case defaultIcon:
        return 'default';
      case crosshair:
      case hand:
      case arrow:
      case move:
      case text:
      case wait:
      case help:
      case progress:
      case notAllowed:
      case contextMenu:
      case cell:
      case verticalText:
      case alias:
      case copy:
      case noDrop:
      case grab:
      case grabbing:
      case allScroll:
      case zoomIn:
      case zoomOut:
      case eResize:
      case nResize:
      case neResize:
      case nwResize:
      case sResize:
      case seResize:
      case swResize:
      case wResize:
      case ewResize:
      case nsResize:
      case neswResize:
      case nwseResize:
      case colResize:
      case rowResize:
        return name;
    }
  }
}

mixin SizedWindow on Size {
  abstract final WindowType type;
}

mixin PositionedWindow on Position {
  abstract final WindowType type;
}

abstract class Size {
  const Size({
    required this.width,
    required this.height,
  });

  final double width;
  final double height;
}

abstract class Position {
  const Position({
    required this.x,
    required this.y,
  });

  final double x;
  final double y;
}

/// A size represented in logical pixels.
///
/// @since 1.0.0
class LogicalSize extends Size with SizedWindow {
  const LogicalSize({
    required super.width,
    required super.height,
  }) : super();

  @override
  WindowType get type => WindowType.Logical;
}

/// A size represented in physical pixels.
///
/// @since 1.0.0
class PhysicalSize extends Size with SizedWindow {
  const PhysicalSize({
    required super.width,
    required super.height,
  }) : super();

  @override
  WindowType get type => WindowType.Physical;

  /// Converts the physical size to a logical one.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// Future<LogicalSize> get windowLogicalSize async {
  ///   final double factor = await appWindow.scaleFactor();
  ///   final PhysicalSize size = await appWindow.innerSize();
  ///   final LogicalSize logical = size.toLogical(factor);
  ///
  ///   return logical;
  /// }
  /// ```
  LogicalSize toLogical(final double scaleFactor) => LogicalSize(
        width: width / scaleFactor,
        height: height / scaleFactor,
      );
}

///  A position represented in logical pixels.
///
/// @since 1.0.0
class LogicalPosition extends Position with PositionedWindow {
  const LogicalPosition({
    required super.x,
    required super.y,
  }) : super();

  @override
  WindowType get type => WindowType.Logical;
}

///  A position represented in physical pixels.
///
/// @since 1.0.0
class PhysicalPosition extends Position with PositionedWindow {
  const PhysicalPosition({
    required super.x,
    required super.y,
  }) : super();

  @override
  WindowType get type => WindowType.Physical;

  /// Converts the physical position to a logical one.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// Future<LogicalPosition> get windowLogicalPosition async {
  ///   final double factor = await appWindow.scaleFactor();
  ///   final PhysicalPosition position = await appWindow.innerPosition();
  ///   final LogicalPosition logical = position.toLogical(factor);
  ///
  ///   return logical;
  /// }
  /// ```
  LogicalPosition toLogical(final double scaleFactor) => LogicalPosition(
        x: x / scaleFactor,
        y: y / scaleFactor,
      );
}

/// Allows you to retrieve information about a given monitor.
///
/// @since 1.0.0
class Monitor {
  const Monitor({
    required this.size,
    required this.position,
    required this.scaleFactor,
    this.name,
  });

  /// Human-readable name of the monitor
  final String? name;

  /// The monitor's resolution.
  final PhysicalSize size;

  /// The Top-left corner position of the monitor relative to the larger full
  /// screen area.
  final PhysicalPosition position;

  /// The scale factor that can be used to map physical pixels to logical
  /// pixels.
  final double scaleFactor;
}

class WindowOptions {
  const WindowOptions({
    this.url,
    this.center,
    this.x,
    this.y,
    this.width,
    this.height,
    this.minWidth,
    this.minHeight,
    this.maxWidth,
    this.maxHeight,
    this.resizable,
    this.title,
    this.fullscreen,
    this.focus,
    this.transparent,
    this.maximized,
    this.visible,
    this.decorations,
    this.alwaysOnTop,
    this.contentProtected,
    this.skipTaskbar,
    this.fileDropEnabled,
    this.theme,
    this.titleBarStyle,
    this.hiddenTitle,
    this.acceptFirstMouse,
    this.tabbingIdentifier,
    this.userAgent,
    this.additionalBrowserArguments,
  });

  /// Remote URL or local file path to open.
  ///
  /// - URL such as `https://github.com/tauri-apps` is opened directly on a
  ///   Tauri window.
  /// - data: URL such as `data:text/html,<html>...` is only supported with the
  ///   `window-data-url` Cargo feature for the `tauri` dependency.
  /// - local file path or route such as `/path/to/page.html` or `/users` is
  ///   appended to the application URL (the devServer URL on development, or
  ///   `tauri://localhost/` and `https://tauri.localhost/` on production).
  final String? url;

  /// Show window in the center of the screen..
  final bool? center;

  /// The initial vertical position. Only applies if `y` is also set.
  final num? x;

  /// The initial horizontal position. Only applies if `x` is also set.
  final num? y;

  /// The initial width.
  final num? width;

  /// The initial height.
  final num? height;

  /// The minimum width. Only applies if `minHeight` is also set.
  final num? minWidth;

  /// The minimum height. Only applies if `minWidth` is also set.
  final num? minHeight;

  /// The maximum width. Only applies if `maxHeight` is also set.
  final num? maxWidth;

  /// The maximum height. Only applies if `maxWidth` is also set.
  final num? maxHeight;

  /// Whether the window is resizable or not.
  final bool? resizable;

  /// Window title.
  final String? title;

  /// Whether the window is in fullscreen mode or not.
  final bool? fullscreen;

  /// Whether the window will be initially focused or not.
  final bool? focus;

  /// Whether the window is transparent or not.
  /// Note that on `macOS` this requires the `macos-private-api` feature flag,
  /// enabled under `tauri.conf.json > tauri > macOSPrivateApi`.
  /// WARNING: Using private APIs on `macOS` prevents your application from
  /// being accepted to the `App Store`.
  final bool? transparent;

  /// Whether the window should be maximized upon creation or not.
  final bool? maximized;

  /// Whether the window should be immediately visible upon creation or not.
  final bool? visible;

  /// Whether the window should have borders and bars or not.
  final bool? decorations;

  /// Whether the window should always be on top of other windows or not.
  final bool? alwaysOnTop;

  /// Prevents the window contents from being captured by other apps.
  final bool? contentProtected;

  /// Whether or not the window icon should be added to the taskbar.
  final bool? skipTaskbar;

  /// Whether the file drop is enabled or not on the webview. By default it is
  /// enabled.
  ///
  /// Disabling it is required to use drag and drop on the frontend on Windows.
  final bool? fileDropEnabled;

  /// The initial window theme. Defaults to the system theme.
  ///
  /// Only implemented on Windows and macOS 10.14+.
  final Theme? theme;

  /// The style of the macOS title bar.
  final TitleBarStyle? titleBarStyle;

  /// If `true`, sets the window title to be hidden on macOS.
  final bool? hiddenTitle;

  /// Whether clicking an inactive window also clicks through to the webview on
  /// macOS.
  final bool? acceptFirstMouse;

  /// Defines the window
  /// [tabbing identifier](https://developer.apple.com/documentation/appkit/nswindow/1644704-tabbingidentifier)
  /// on macOS.
  ///
  /// Windows with the same tabbing identifier will be grouped together.
  /// If the tabbing identifier is not set, automatic tabbing will be disabled.
  final String? tabbingIdentifier;

  /// The user agent for the webview.
  final String? userAgent;

  /// Additional arguments for the webview.///*Windows Only**
  final String? additionalBrowserArguments;
}

/// @ignore
class WindowDef {
  const WindowDef({required this.label});

  final WindowLabel label;

  Map<String, dynamic> toJSON() => <String, dynamic>{'label': label};
}

class TauriMetadataProxy {
  TauriMetadataProxy({
    required final List<WindowDef> windows,
    required final WindowDef currentWindow,
  })  : __windows = windows,
        __currentWindow = currentWindow;

  List<WindowDef> __windows;
  WindowDef __currentWindow;

  List<WindowDef> get windows => __windows;
  set windows(final List<WindowDef> defs) {
    __windows = defs;
    window.tauriMetadata = TauriMetadataProxy(
      windows: __windows,
      currentWindow: __currentWindow,
    );
  }

  WindowDef get currentWindow => __currentWindow;
  set currentWindow(final WindowDef def) {
    __currentWindow = def;
    window.tauriMetadata = TauriMetadataProxy(
      windows: __windows,
      currentWindow: __currentWindow,
    );
  }

  js.JsObject get jsify => js.JsObject.jsify(
        <String, dynamic>{
          TauriWindowDefinition.metadataWindows.def:
              js.JsArray<js.JsObject>.from(
            __windows.map(
              (final WindowDef window) => js.JsObject.jsify(window.toJSON()),
            ),
          ),
          TauriWindowDefinition.metadataCurrentWindow.def: js.JsObject.jsify(
            __currentWindow.toJSON(),
          ),
        },
      );
}

class TauriIPCPostMessageProxy {
  const TauriIPCPostMessageProxy({required this.postMessage});

  final IPCPostMessageCallback postMessage;

  js.JsObject get jsify => js.JsObject.jsify(
        <String, dynamic>{
          TauriWindowDefinition.ipcPostMessage.def:
              js.JsFunction.withThis(postMessage),
        },
      );
}
