#!/usr/bin/env bash

set -x
set -euo pipefail

main() {
    local package

    for package in "${@}"; do
        echo "Package: ${package}:${TARGET_ARCH}
Pin: release *
Pin-Priority: -1" > "/etc/apt/preferences.d/${package}"
        echo "${package}"
    done

    rm "${0}"
}

main "${@}"
