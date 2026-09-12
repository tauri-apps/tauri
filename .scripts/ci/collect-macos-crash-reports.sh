#!/bin/bash

# Copyright 2019-2024 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

# Collects and prints macOS crash reports after a failed `@tauri-apps/api` e2e run.
#
# The CrabNebula Webdriver proxies every command to a server running *inside* the app
# process, so when the app dies the suite only ever reports `connection refused` — the
# actual cause is never in the job log. The app's stdout/stderr is inherited and no panic
# message is printed, which means it goes down on a signal. macOS records the reason in a
# crash report, and that is the only place it survives the run.
#
# Copies every report into $1 (default: $RUNNER_TEMP/crash-reports) for upload.

set -euo pipefail

dest="${1:-${RUNNER_TEMP:-/tmp}/crash-reports}"
mkdir -p "$dest"

# ReportCrash writes asynchronously; give the last crash a moment to land.
sleep 5

found=0
for f in "$HOME/Library/Logs/DiagnosticReports"/*.ips \
         "$HOME/Library/Logs/DiagnosticReports/Retired"/*.ips; do
  [ -e "$f" ] || continue
  found=$((found + 1))
  cp "$f" "$dest/"
  echo "::group::$(basename "$f")"
  # An .ips file is a one-line JSON header followed by the JSON report body.
  head -n 1 "$f"
  tail -n +2 "$f" | jq -r '
    "exception:   \(.exception // {} | tojson)",
    "termination: \(.termination // {} | tojson)",
    "asi:         \(.asi // {} | tojson)",
    "faulting thread \(.faultingThread // 0):",
    (. as $r
      | $r.threads[$r.faultingThread // 0].frames[]?
      | "  \($r.usedImages[.imageIndex].name // "?")  \(.symbol // "?")  +\(.imageOffset)"),
    "last ObjC exception:",
    (. as $r
      | $r.lastExceptionBacktrace[]?
      | "  \($r.usedImages[.imageIndex].name // "?")  \(.symbol // "?")  +\(.imageOffset)")
  ' || cat "$f"
  echo "::endgroup::"
done

[ "$found" -gt 0 ] || echo "no crash reports under $HOME/Library/Logs/DiagnosticReports"
