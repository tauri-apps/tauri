// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// ignore_for_file: constant_identifier_names, avoid_returning_this

/// Access the system shell.
/// Allows you to spawn child processes and manage files and URLs using their
/// default application.
///
/// This package is also accessible with `window.__TAURI__.shell` when
/// [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri)
/// in `tauri.conf.json` is set to `true`.
///
/// The APIs must be added to
/// [`tauri.allowlist.shell`](https://tauri.app/v1/api/config/#allowlistconfig.shell)
/// in `tauri.conf.json`:
/// ```json
/// {
///   "tauri": {
///     "allowlist": {
///       "shell": {
///         "all": true, // enable all shell APIs
///         "execute": true, // enable process spawn APIs
///         "sidecar": true, // enable spawning sidecars
///         "open": true // enable opening files/URLs using the default program
///       }
///     }
///   }
/// }
/// ```
/// It is recommended to allowlist only the APIs you use for optimal bundle size
/// and security.
///
/// ## Security
///
/// This API has a scope configuration that forces you to restrict the programs
/// and arguments that can be used.
///
/// ### Restricting access to the {@link open | `open`} API
///
/// On the allowlist, `open: true` means that the {@link open} API can be used
/// with any URL, as the argument is validated with the
/// `^((mailto:\w+)|(tel:\w+)|(https?://\w+)).+` regex. You can change that
/// regex by changing the bool value to a String, e.g.
/// `open: ^https://github.com/`.
///
/// ### Restricting access to the {@link Command | `Command`} APIs
///
/// The `shell` allowlist object has a `scope` field that defines an array of
/// CLIs that can be used. Each CLI is a configuration object
/// `{
///   final String name,
///   final String cmd,
///   final bool? sidecar,
///   final List<Arg> | bool? args
/// }`.
///
///   - `name`: the unique identifier of the command, passed to the
///     {@link Command.constructor | Command constructor}.
///     If it's a sidecar, this must be the value defined on `tauri.conf.json >
///     tauri > bundle > externalBin`.
///   - `cmd`: the program that is executed on this configuration. If it's a
///     sidecar, this value is ignored.
///   - `sidecar`: whether the object configures a sidecar or a system program.
///   - `args`: the arguments that can be passed to the program. By default no
///     arguments are allowed.
///     - `true` means that any argument list is allowed.
///     - `false` means that no arguments are allowed.
///     - otherwise an array can be configured. Each item is either a String
///       representing the fixed argument value or a `{final String validator}`
///       that defines a regex validating the argument value.
///
/// #### Example scope configuration
///
/// CLI: `git commit -m "the commit message"`
///
/// Configuration:
/// ```json
/// {
///   "scope": [
///     {
///       "name": "run-git-commit",
///       "cmd": "git",
///       "args": ["commit", "-m", { "validator": "\\S+" }]
///     }
///   ]
/// }
/// ```
/// Usage:
/// ```dart
/// import 'package:tauri_apps/api/shell.dart' show Command;
///
/// final Command cmd = Command(
///   program: 'run-git-commit',
///   args: Union<String, List<String>>.right(
///     <String>['commit', '-m', 'the commit message'],
///   ),
/// );
/// ```
///
/// Trying to execute any API with a program not configured on the scope results
/// in a future rejection due to denied access.
library tauri_shell;

import 'dart:async';
import 'dart:typed_data';

import './helpers/tauri.dart';
import 'event.dart';
import 'helpers/shell.dart' as helpers;
import 'http.dart';

export 'helpers/shell.dart';

/// @since 1.0.0
class SpawnOptions {
  const SpawnOptions({
    this.cwd,
    this.env,
    this.encoding,
  });

  /// Current working directory.
  final String? cwd;

  /// Environment variables. set to `null` to clear the process env.
  final Map<String, String>? env;

  /// Character encoding for stdout/stderr
  ///
  /// @since 1.1.0
  final String? encoding;
}

class InternalSpawnOptions extends SpawnOptions {
  const InternalSpawnOptions({
    this.sidecar,
    super.cwd,
    super.env,
    super.encoding,
  }) : super();

  final bool? sidecar;
}

/// @since 1.0.0
class ChildProcess {
  const ChildProcess({
    required this.stdout,
    required this.stderr,
    this.code,
    this.signal,
  });

  /// Exit code of the process. `null` if the process was terminated by a signal
  /// on Unix.
  final int? code;

  /// If the process was terminated by a signal, represents that signal.
  final int? signal;

  /// The data that the process wrote to `stdout`.
  final String stdout;

  /// The data that the process wrote to `stderr`.
  final String stderr;
}

enum TauriActionEvent implements TauriEventInterface {
  Stdout,
  Stderr,
  Terminated,
  Error;

  @override
  String get stringValue => name;
}

/// Payload for the `Terminated` command event.
class TerminatedPayload {
  const TerminatedPayload({
    this.code,
    this.signal,
  });

  /// Exit code of the process. `null` if the process was terminated by a signal
  /// on Unix.
  final int? code;

  /// If the process was terminated by a signal, represents that signal.
  final int? signal;
}

class InvalidPayload implements Exception {
  const InvalidPayload();

  @override
  String toString() => 'Payload must be of type TerminatedOverload, '
      'if it is TerminatedEvent, or of type String in any other case.';
}

/// Describes the event message received from the command.
class CommandEvent<T> extends Event<T> {
  factory CommandEvent({
    required final TauriActionEvent event,
    required final Union<String, TerminatedPayload> payload,
    final int? id,
    final String? windowLabel,
  }) {
    if (event == TauriActionEvent.Terminated) {
      if (payload.deref is! TerminatedPayload) {
        throw const InvalidPayload();
      }
    } else {
      if (payload.deref is! String) {
        throw const InvalidPayload();
      }
    }

    return CommandEvent<T>._(
      event: EventName(event),
      payload: payload.deref as T,
      id: id,
      windowLabel: windowLabel,
    );
  }

  const CommandEvent._({
    required super.event,
    required super.payload,
    final String? windowLabel,
    final int? id,
  }) : super(id: id ?? 0, windowLabel: windowLabel ?? '');
}

/// Opens a path or URL with the system's default app,
/// or the one specified with `openWith`.
///
/// The `openWith` value must be one of `firefox`, `google chrome`, `chromium`
/// `safari`, `open`, `start`, `xdg-open`, `gio`, `gnome-open`, `kde-open` or
/// `wslview`.
///
///
/// ```dart
/// import 'package:tauri_apps/api/shell.dart' show open;
/// // opens the given URL on the default browser:
/// open('https://github.com/tauri-apps/tauri').then(
///   () => print('Tauri is awesome!),
/// );
/// // opens the given URL using `firefox`:
/// open('https://github.com/tauri-apps/tauri', 'firefox').then(
///   () => print('Tauri is awesome!),
/// );
/// // opens a file using the default program:
/// open('/path/to/file').then(() => print('Tauri is awesome!));
/// ```
///
///   * [path]: The path or URL to open.
///     This value is matched against the String regex defined on
///     `tauri.conf.json > tauri > allowlist > shell > open`, which defaults to
///     `^((mailto:\w+)|(tel:\w+)|(https?://\w+)).+`.
///   * [openWith]: The app to open the file or URL with.
///     Defaults to the system default application for the specified path type.
///
/// @since 1.0.0
Future<void> open(final String path, final String? openWith) async =>
    invokeTauriCommand<void>(
      TauriCommand(
        tauriModule: TauriModule.Shell,
        message: TauriCommandMessage(
          cmd: 'open',
          path: path,
          openWith: openWith,
        ),
      ),
    );

typedef EventListener<T> = void Function(T args);
typedef DynamicEventListener = EventListener<dynamic>;

/// @since 1.0.0
class EventEmitter<E extends TauriEventInterface> {
  EventEmitter() : _eventListeners = <E, List<EventListener<dynamic>>>{};

  /// @ignore
  final Map<E, List<EventListener<dynamic>>> _eventListeners;

  /// Alias for `emitter.on(eventName: eventName, listener: listener)`.
  ///
  /// @since 1.1.0
  EventEmitter<E> addListener<T>({
    required final E eventName,
    required final EventListener<T> listener,
  }) {
    on<T>(eventName: eventName, listener: listener);
    return this;
  }

  /// Alias for `emitter.off(eventName: eventName, listener: listener)`.
  ///
  /// @since 1.1.0
  EventEmitter<E> removeListener<T>({
    required final E eventName,
    required final EventListener<T> listener,
  }) {
    off<T>(eventName: eventName, listener: listener);
    return this;
  }

  /// Adds the `listener` function to the end of the listeners array for the
  /// event named `eventName`. No checks are made to see if the `listener` has
  /// already been added. Multiple calls passing the same combination of
  /// `eventName`and `listener` will result in the `listener` being added, and
  /// called, multiple times.
  ///
  /// Returns a reference to the `EventEmitter`, so that calls can be chained.
  ///
  /// @since 1.0.0
  EventEmitter<E> on<T>({
    required final E eventName,
    required final EventListener<T> listener,
  }) {
    if (_eventListeners.containsKey(eventName)) {
      _eventListeners[eventName]!.add(listener as DynamicEventListener);
    } else {
      _eventListeners[eventName] = <EventListener>[
        listener as DynamicEventListener,
      ];
    }
    return this;
  }

  /// Adds a **one-time**`listener` function for the event named `eventName`.
  /// The next time `eventName` is triggered, this listener is removed and then
  /// invoked.
  ///
  /// Returns a reference to the `EventEmitter`, so that calls can be chained.
  ///
  /// @since 1.1.0
  EventEmitter<E> once<T>({
    required final E eventName,
    required final EventListener<T> listener,
  }) {
    void wrapper(final T args) {
      removeListener(eventName: eventName, listener: wrapper);
      listener(args);
    }

    addListener(eventName: eventName, listener: wrapper);
    return this;
  }

  /// Removes the all specified listener from the listener array for the event
  /// eventName.
  ///
  /// Returns a reference to the `EventEmitter`, so that calls can be chained.
  ///
  /// @since 1.1.0
  EventEmitter<E> off<T>({
    required final E eventName,
    required final EventListener<T> listener,
  }) {
    if (_eventListeners.containsKey(eventName)) {
      _eventListeners[eventName] = _eventListeners[eventName]!
          .where((final EventListener<T> l) => l != listener)
          .toList();
    }
    return this;
  }

  /// Removes all listeners, or those of the specified eventName.
  ///
  /// Returns a reference to the `EventEmitter`, so that calls can be chained.
  ///
  /// @since 1.1.0
  EventEmitter<E> removeAllListeners<T>([final E? event]) {
    if (event != null) {
      _eventListeners.remove(event);
    } else {
      _eventListeners.clear();
    }
    return this;
  }

  /// @ignore
  /// Synchronously calls each of the listeners registered for the event named
  /// `eventName`, in the order they were registered, passing the supplied
  /// arguments to each.
  ///
  /// @returns `true` if the event had listeners, `false` otherwise.
  bool emit<T>(final E eventName, final T args) {
    if (_eventListeners.containsKey(eventName)) {
      final List<EventListener<T>> listeners = _eventListeners[eventName]!;
      for (final EventListener<T> listener in listeners) {
        listener(args);
      }
      return true;
    }
    return false;
  }

  /// Returns the number of listeners listening to the event named `eventName`.
  ///
  /// @since 1.1.0
  int listenerCount(final E eventName) =>
      _eventListeners[eventName]?.length ?? 0;

  /// Adds the `listener` function to the _beginning_ of the listeners array for
  /// the event named `eventName`. No checks are made to see if the `listener`
  /// has already been added. Multiple calls passing the same combination of
  /// `eventName`and `listener` will result in the `listener` being added, and
  /// called, multiple times.
  ///
  /// Returns a reference to the `EventEmitter`, so that calls can be chained.
  ///
  /// @since 1.1.0
  EventEmitter<E> prependListener<T>({
    required final E eventName,
    required final EventListener<T> listener,
  }) {
    if (_eventListeners.containsKey(eventName)) {
      _eventListeners[eventName]!.unshift(listener as DynamicEventListener);
    } else {
      _eventListeners[eventName] = <EventListener>[
        listener as DynamicEventListener,
      ];
    }
    return this;
  }

  /// Adds a **one-time**`listener` function for the event named `eventName` to
  /// the_beginning_ of the listeners array. The next time `eventName` is
  /// triggered, this listener is removed, and then invoked.
  ///
  /// Returns a reference to the `EventEmitter`, so that calls can be chained.
  ///
  /// @since 1.1.0
  EventEmitter<E> prependOnceListener<T>({
    required final E eventName,
    required final EventListener<T> listener,
  }) {
    void wrapper(final T args) {
      removeListener<T>(eventName: eventName, listener: wrapper);
      listener(args);
    }

    prependListener<T>(eventName: eventName, listener: wrapper);
    return this;
  }
}

/// @since 1.1.0
class Child {
  const Child(this.pid);

  /// The child process `pid`.
  final int pid;

  /// Writes `data` to the `stdin`.
  ///
  ///   * [data]: The message to write, either a String or a byte array.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/shell.dart' show Command;
  ///
  /// Command(program: 'node').spawn().then(
  ///   (final Child child) async {
  ///     await child.write('message');
  ///     await child.write([0, 1, 2, 3, 4, 5]);
  ///   },
  /// );
  /// ```
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> write<T>(final T data) async {
    dynamic data_ = data;
    if (data is! Uint8List && data is! String) {
      if (data is Iterable<int>) {
        data_ = Uint8List.fromList(data.toList());
      }
      if (data is ByteBuffer) {
        data_ = Uint8List.view(data);
      }

      throw InvalidTypeException<T>(data, 'data');
    }

    return invokeTauriCommand<void>(
      TauriCommand(
        tauriModule: TauriModule.Shell,
        message: TauriCommandMessage(
          cmd: 'stdinWrite',
          pid: pid,
          // correctly serialize Uint8Arrays
          buffer: (data is String) ? data : List<int>.from(data_),
        ),
      ),
    );
  }

  /// Kills the child process.
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> kill() async => invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Shell,
          message: TauriCommandMessage(
            cmd: 'killChild',
            pid: pid,
          ),
        ),
      );
}

enum TauriCommandEvent implements TauriEventInterface {
  close,
  error;

  @override
  String get stringValue => name;
}

class TauriDataCommandEvent implements TauriEventInterface {
  const TauriDataCommandEvent();

  @override
  String get stringValue => 'data';
}

/// The entry point for spawning child processes.
/// It emits the `close` and `error` events.
///
/// ```dart
/// import 'package:tauri_apps/api/shell.dart' show Command;
///
/// final Command command = Command(program: 'node')
///   ..on<TerminatedPayload>(
///       eventName: TauriCommandEvent.close,
///       listener: (final TerminatedPayload data) =>
///         print(
///           'command finished with code ${data.code} '
///           'and signal ${data.signal}',
///         ),
///   )
///   ..on<String>(
///     eventName: TauriCommandEvent.error,
///     listener: (final String error) => print('command error: "$error"'),
///   );
///
/// command.stdout.on<String>(
///   eventName: const TauriCommandDataEvent(),
///   listener: (final String line) => print('command stdout: "$line"'),
/// );
/// command.stderr.on<String>(
///   eventName: const TauriCommandDataEvent(),
///   listener: (final String line) => print('command stderr: "$line"'),
/// );
///
/// command.spawn().then((final Child child) => print('pid: ${child.pid}'));
/// ```
///
/// @since 1.1.0
class Command extends EventEmitter<TauriCommandEvent> {
  /// Creates a new `Command` instance.
  ///
  ///   * [program]: The program name to execute.
  ///     It must be configured on `tauri.conf.json > tauri > allowlist > shell
  ///     > scope`.
  ///   * [args]: Program arguments.
  ///   * [options]: Spawn options.
  Command({
    required final String program,
    final Union<String, List<String>> args =
        const Union<String, List<String>>.right(<String>[]),
    final SpawnOptions options = const SpawnOptions(),
  })  : _program = program,
        stderr = EventEmitter<TauriDataCommandEvent>(),
        stdout = EventEmitter<TauriDataCommandEvent>(),
        _options = options is InternalSpawnOptions
            ? options
            : InternalSpawnOptions(
                cwd: options.cwd,
                env: options.env,
                encoding: options.encoding,
              ),
        _args = args.isLeft ? <String>[args.left] : args.right,
        super();

  /// Creates a command to execute the given sidecar program.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/shell.dart' show Command;
  ///
  /// Command.sidecar(program: 'my-sidecar')
  ///   .execute()
  ///   .then((final ChildProcess output) => print(output));
  /// ```
  ///
  ///   * [program]: The program to execute.
  ///     It must be configured on `tauri.conf.json > tauri > allowlist > shell
  ///     > scope`.
  factory Command.sidecar({
    required final String program,
    required final Union<String, List<String>> args,
    final SpawnOptions options = const SpawnOptions(),
  }) =>
      Command(
        program: program,
        args: args,
        options: InternalSpawnOptions(
          cwd: options.cwd,
          env: options.env,
          encoding: options.encoding,
          sidecar: true,
        ),
      );

  /// @ignore Program to execute.
  final String _program;

  /// @ignore Program arguments
  final List<String> _args;

  /// @ignore Spawn options.
  final InternalSpawnOptions _options;

  /// Event emitter for the `stdout`. Emits the `data` event.
  final EventEmitter<TauriDataCommandEvent> stdout;

  /// Event emitter for the `stderr`. Emits the `data` event.
  final EventEmitter<TauriDataCommandEvent> stderr;

  /// Executes the command as a child process, returning a handle to it.
  ///
  /// @returns A future resolving to the child process handle.
  Future<Child> spawn() async => helpers
      .execute<dynamic>(
        onEvent: (final CommandEvent<dynamic> event) {
          switch (event.event.name) {
            case TauriActionEvent.Error:
              this.emit<dynamic>(TauriCommandEvent.error, event.payload);
              break;
            case TauriActionEvent.Terminated:
              this.emit<dynamic>(TauriCommandEvent.close, event.payload);
              break;
            case TauriActionEvent.Stdout:
              stdout.emit<dynamic>(
                const TauriDataCommandEvent(),
                event.payload,
              );
              break;
            case TauriActionEvent.Stderr:
              stderr.emit<dynamic>(
                const TauriDataCommandEvent(),
                event.payload,
              );
              break;
          }
        },
        program: _program,
        args: Union<String, List<String>>.right(_args),
        options: _options,
      )
      .then(Child.new);

  /// Executes the command as a child process, waiting for it to finish and
  /// collecting all of its output.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/shell.dart' show Command;
  ///
  /// Command(
  ///   program: 'echo',
  ///   args: const Union<String, List<String>>.left('message'),
  /// ).execute().then(
  ///   (final ChildProcess output) {
  ///     assert(output.code == 0);
  ///     assert(output.signal == null);
  ///     assert(output.stdout == 'message');
  ///     assert(output.stderr == '');
  ///   },
  /// );
  /// ```
  ///
  /// @returns A future resolving to the child process output.
  Future<ChildProcess> execute() async => Future<ChildProcess>(
        () async {
          final Completer<ChildProcess> promise = Completer<ChildProcess>();

          on<Object>(
            eventName: TauriCommandEvent.error,
            listener: promise.completeError,
          );

          final List<String> stdout = <String>[];
          final List<String> stderr = <String>[];

          this.stdout.on<String>(
                eventName: const TauriDataCommandEvent(),
                listener: stdout.add,
              );

          this.stderr.on<String>(
                eventName: const TauriDataCommandEvent(),
                listener: stderr.add,
              );

          on<TerminatedPayload>(
            eventName: TauriCommandEvent.close,
            listener: (final TerminatedPayload payload) {
              promise.complete(
                ChildProcess(
                  code: payload.code,
                  signal: payload.signal,
                  stdout: stdout.join('\n'),
                  stderr: stderr.join('\n'),
                ),
              );
            },
          );

          try {
            await spawn();
          } on Exception catch (e, trace) {
            promise.completeError(e, trace);
          }

          return promise.future;
        },
      );
}
