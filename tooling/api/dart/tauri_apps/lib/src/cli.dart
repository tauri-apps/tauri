// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// Parse arguments from your Command Line Interface.
///
/// This package is also accessible with `window.__TAURI__.cli` when
/// [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri)
/// in `tauri.conf.json` is set to `true`.
library tauri_cli;


import 'helpers/tauri.dart';

/// @since 1.0.0
class ArgMatch<T> {
  factory ArgMatch({
    required final int occurrences,
    final T? value,
  }) {
    if (value == null ||
        value is bool ||
        value is String ||
        value is List<String>) {
      return ArgMatch<T>._(occurrences: occurrences, value: value);
    }

    throw UnsupportedError(
      'value must be of type: '
      'String || bool || List<String>?.',
    );
  }

  const ArgMatch._({required this.occurrences, this.value});

  /// string if takes value
  /// boolean if flag
  /// string[] or null if takes multiple values
  final T? value;

  /// Number of occurrences
  final int occurrences;
}

/// @since 1.0.0
class SubcommandMatch {
  const SubcommandMatch(this.name, this.matches);

  final String name;
  final CliMatches matches;
}

/// @since 1.0.0
class CliMatches {
  const CliMatches(this.args, [this.subcommand]);
  final Map<String, ArgMatch<dynamic>> args;
  final SubcommandMatch? subcommand;
}

/// Parse the arguments provided to the current process and get the matches
/// using the configuration defined
/// [`tauri.cli`](https://tauri.app/v1/api/config/#tauriconfig.cli) in
/// `tauri.conf.json`
///
/// ```dart
/// import 'package:tauri_apps/api/cli.dart' show getMatches;
///
/// (() async {
///   final CliMatches matches = await getMatches();
///
///   if (matches.subcommand?.name == 'run') {
///     // `./your-app run $ARGS` was executed
///     final Map<String, ArgMatch> args =
///         matches.subcommand?.matches.args ?? <String>[];
///     if (args.keys.contains('debug')) {
///       // `./your-app run --debug` was executed
///     }
///   } else {
///     final Map<String, ArgMatch> args = matches.args;
///     // `./your-app $ARGS` was executed
///   }
/// })();
/// ```
///
/// @since 1.0.0
Future<CliMatches> getMatches() async => invokeTauriCommand<CliMatches>(
      const TauriCommand(
        tauriModule: TauriModule.Cli,
        message: TauriCommandMessage(
          cmd: 'cliMatches',
        ),
      ),
    );
