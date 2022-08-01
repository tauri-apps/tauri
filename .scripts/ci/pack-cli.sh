#!/bin/bash
set -euxo pipefail

for o in outputs/*; do
  pushd "$o"

  chmod +x cargo-tauri*
  cp ../../tooling/cli/LICENSE* ../../tooling/cli/README.md .

  target=$(basename "$o" | cut -d. -f1)
  if grep -qE '(apple|windows)' <<< "$target"; then
    zip "../${target}.zip" *
  else
    tar cv * | gzip -9 > "../${target}.tgz"
  fi

  popd
done
