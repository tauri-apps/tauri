// ignore_for_file: non_constant_identifier_names

part of tauri_window;

TauriWindow get window => TauriWindow();

/// The WebviewWindow for the current window.
WebviewWindow appWindow = _initAppWindow();

/// Extends [html.Window] to register tauri callbacks on it and also provide
/// them as dart functions directly.
class TauriWindow extends html.Window {
  factory TauriWindow() {
    _initProperties();
    return html.window as TauriWindow;
  }

  static TauriIPCCallback get _defaultTauriIpcCallback =>
      (final dynamic message) {};

  static IPCPostMessageCallback get _defaultIpcPostMessageCallback =>
      (final String args) {};

  static const WindowDef _defaultWindowDef = WindowDef(label: '');

  static final TauriIPCPostMessageProxy _defaultIpcProxy =
      TauriIPCPostMessageProxy(postMessage: _defaultIpcPostMessageCallback);

  static final TauriMetadataProxy _defaultMetadataProxy = TauriMetadataProxy(
    windows: <WindowDef>[_defaultWindowDef],
    currentWindow: _defaultWindowDef,
  );

  static void _initProperties() {
    if (!js.context.hasProperty(TauriWindowDefinition.tauriIpc.def)) {
      js.context[TauriWindowDefinition.tauriIpc.def] = js.JsFunction.withThis(
        _defaultTauriIpcCallback,
      );
    }

    if (!js.context.hasProperty(TauriWindowDefinition.ipc.def)) {
      js.context[TauriWindowDefinition.ipc.def] = _defaultIpcProxy.jsify;
    }

    if (!js.context.hasProperty(TauriWindowDefinition.tauriMetadata.def)) {
      js.context[TauriWindowDefinition.tauriMetadata.def] =
          _defaultMetadataProxy.jsify;
    }
  }

  TauriIPCCallback __TAURI_IPC__ = _defaultTauriIpcCallback;

  TauriIPCPostMessageProxy _ipc = _defaultIpcProxy;

  TauriMetadataProxy _metadata = _defaultMetadataProxy;

  TauriIPCCallback get tauriIpc => __TAURI_IPC__;
  set tauriIpc(final TauriIPCCallback callback) {
    js.context[TauriWindowDefinition.tauriIpc.def] =
        js.JsFunction.withThis(callback);
    __TAURI_IPC__ = callback;
  }

  TauriIPCPostMessageProxy get ipc => _ipc;
  set ipc(final TauriIPCPostMessageProxy proxy) {
    js.context[TauriWindowDefinition.ipc.def] = proxy.jsify;
    _ipc = proxy;
  }

  TauriMetadataProxy get tauriMetadata => _metadata;
  set tauriMetadata(final TauriMetadataProxy meta) {
    js.context[TauriWindowDefinition.tauriMetadata.def] = meta.jsify;
    _metadata = meta;
  }

  void removeProperty(final TauriWindowDefinition prop) {
    switch (prop) {
      case TauriWindowDefinition.tauriIpc:
        js.context.deleteProperty(prop.def);
        tauriIpc = _defaultTauriIpcCallback;
        break;
      case TauriWindowDefinition.tauriMetadata:
        js.context.deleteProperty(prop.def);
        tauriMetadata = _defaultMetadataProxy;
        break;
      case TauriWindowDefinition.metadataWindows:
        js.context.deleteProperty(
          <String, dynamic>{
            TauriWindowDefinition.tauriMetadata.def: prop.def,
          },
        );
        tauriMetadata = TauriMetadataProxy(
          windows: <WindowDef>[_defaultWindowDef],
          currentWindow: tauriMetadata.currentWindow,
        );
        break;
      case TauriWindowDefinition.metadataCurrentWindow:
        js.context.deleteProperty(
          <String, dynamic>{
            TauriWindowDefinition.tauriMetadata.def: prop.def,
          },
        );
        tauriMetadata = TauriMetadataProxy(
          windows: tauriMetadata.windows,
          currentWindow: _defaultWindowDef,
        );
        break;
      case TauriWindowDefinition.ipc:
        js.context.deleteProperty(prop.def);
        ipc = _defaultIpcProxy;
        break;
      case TauriWindowDefinition.ipcPostMessage:
        js.context.deleteProperty(
          <String, dynamic>{
            TauriWindowDefinition.ipc.def: prop.def,
          },
        );
        ipc = _defaultIpcProxy;
        break;
    }
  }
}

enum TauriWindowDefinition {
  tauriIpc('__TAURI_IPC__'),
  tauriMetadata('__TAURI_METADATA__'),
  metadataWindows('__windows'),
  metadataCurrentWindow('__currentWindow'),
  ipc('ipc'),
  ipcPostMessage('postMessage');

  const TauriWindowDefinition(this.def);

  final String def;
}
