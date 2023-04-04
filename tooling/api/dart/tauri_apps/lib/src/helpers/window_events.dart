part of tauri_window;

/// @ignore
/// Events that are emitted right here instead of by the created webview.
const List<EventName> localTauriEvents = <EventName>[
  EventName(TauriEvent.fromString('tauri://created')),
  EventName(TauriEvent.fromString('tauri://error')),
];

/// @since 1.0.2
class CloseRequestedEvent {
  factory CloseRequestedEvent({
    required final Event<void> event,
  }) =>
      CloseRequestedEvent._(
        event: event.event,
        windowLabel: event.windowLabel,
        id: event.id,
      );

  CloseRequestedEvent._({
    required this.event,
    required this.windowLabel,
    required this.id,
  });

  /// Event name
  final EventName event;

  /// The label of the window that emitted this event.
  final String windowLabel;

  /// Event identifier used to unlisten
  final int id;

  bool _preventDefault = false;

  void preventDefault() => _preventDefault = true;

  bool get isPreventDefault => _preventDefault;
}

class FileDropEvent {
  const FileDropEvent({required this.type, this.paths = const <String>[]});

  final FileDropEventType type;

  final List<String> paths;
}

/// The payload for the `scaleChange` event.
///
/// @since 1.0.2
class ScaleFactorChanged {
  const ScaleFactorChanged({required this.scaleFactor, required this.size});

  /// The new window scale factor.
  final double scaleFactor;

  /// The new window size
  final PhysicalSize size;
}
