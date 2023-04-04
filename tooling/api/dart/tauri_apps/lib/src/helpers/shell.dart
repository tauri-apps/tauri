import 'dart:async';

import '../event.dart';
import '../shell.dart';
import '../tauri.dart';
import 'tauri.dart';

/// Spawns a process.
///
/// @ignore
///   * [program]: The name of the scoped command.
///   * [onEvent]: Event handler.
///   * [args]: Program arguments.
///   * [options]: Configuration for the process spawn.
/// @returns A future resolving to the process id.
Future<int> execute<E>({
  required final void Function(CommandEvent<E> event) onEvent,
  required final String program,
  final Union<String, List<String>>? args,
  final InternalSpawnOptions? options,
}) async =>
    invokeTauriCommand<int>(
      TauriCommand(
        tauriModule: TauriModule.Shell,
        message: TauriCommandMessage(
          cmd: 'execute',
          program: program,
          args: args?.deref,
          options: options,
          onEventFn: transformCallback<CommandEvent<E>>(callback: onEvent),
        ),
      ),
    );
