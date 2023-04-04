part of tauri_window;

Monitor? mapMonitor(final Monitor? m) => m == null
    ? null
    : Monitor(
        name: m.name,
        scaleFactor: m.scaleFactor,
        position: PhysicalPosition(x: m.position.x, y: m.position.y),
        size: PhysicalSize(width: m.size.width, height: m.size.height),
      );

/// Returns the monitor on which the window currently resides.
/// Returns `null` if current monitor can't be detected.
///
/// ```dart
/// import 'package:tauri_apps/api/window.dart' show currentMonitor;
///
/// currentMonitor().then((final Monitor? monitor) => print(monitor));
/// ```
///
/// @since 1.0.0
Future<Monitor?> currentMonitor() async => invokeTauriCommand<Monitor?>(
      const TauriCommand(
        tauriModule: TauriModule.Window,
        message: TauriCommandMessage(
          cmd: 'manage',
          data: <String, dynamic>{
            'cmd': <String, dynamic>{
              'type': 'currentMonitor',
            },
          },
        ),
      ),
    ).then(mapMonitor);

/// Returns the primary monitor of the system.
/// Returns `null` if it can't identify any monitor as a primary one.
///
/// ```dart
/// import 'package:tauri_apps/api/window.dart' show primaryMonitor;
///
/// primaryMonitor().then((final Monitor? monitor) => print(monitor));
/// ```
///
/// @since 1.0.0
Future<Monitor?> primaryMonitor() async => invokeTauriCommand<Monitor?>(
      const TauriCommand(
        tauriModule: TauriModule.Window,
        message: TauriCommandMessage(
          cmd: 'manage',
          data: <String, dynamic>{
            'cmd': <String, dynamic>{
              'type': 'primaryMonitor',
            },
          },
        ),
      ),
    ).then(mapMonitor);

/// Returns the list of all the monitors available on the system.
///
/// ```dart
/// import 'package:tauri_apps/api/window.dart' show availableMonitors;
///
/// availableMonitors().then((final List<Monitor> monitors) => print(monitors));
/// ```
///
/// @since 1.0.0
Future<List<Monitor>> availableMonitors() async =>
    invokeTauriCommand<List<Monitor>>(
      const TauriCommand(
        tauriModule: TauriModule.Window,
        message: TauriCommandMessage(
          cmd: 'manage',
          data: <String, dynamic>{
            'cmd': <String, dynamic>{
              'type': 'availableMonitors',
            },
          },
        ),
      ),
    ).then(
      (final List<Monitor> ms) => ms.map(mapMonitor).whereNotNull().toList(),
    );

/// Get an instance of [WebviewWindow] for the current webview window.
///
/// @since 1.0.0
WebviewWindow getCurrent() => WebviewWindow._internal(
      label: window.tauriMetadata.currentWindow.label,
    );

/// Gets a list of instances of [WebviewWindow] for all available webview
/// windows.
///
/// @since 1.0.0
List<WebviewWindow> getAll() => window.tauriMetadata.windows
    .map((final WindowDef w) => WebviewWindow._internal(label: w.label))
    .toList();

WebviewWindow _initAppWindow() {
  if (js.context.hasProperty(TauriWindowDefinition.tauriMetadata.def)) {
    return WebviewWindow._internal(
      label: window.tauriMetadata.currentWindow.label,
    );
  } else {
    print(
      'Could not find "window.__TAURI_METADATA__". The "appWindow" value will '
      'reference the "main" window label.\n'
      'Note that this is not an issue if running this frontend on a browser '
      'instead of a Tauri window.',
    );

    return const WebviewWindow._internal(label: 'main');
  }
}
