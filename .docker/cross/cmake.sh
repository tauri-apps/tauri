#!/usr/bin/env bash

set -x
set -euo pipefail

# shellcheck disable=SC1091
. lib.sh

main() {
    local version=3.23.1

    install_packages curl

    local td
    td="$(mktemp -d)"

    pushd "${td}"

    curl --retry 3 -sSfL "https://github.com/Kitware/CMake/releases/download/v${version}/cmake-${version}-linux-x86_64.sh" -o cmake.sh
    sh cmake.sh --skip-license --prefix=/usr/local

    popd

    purge_packages

    rm -rf "${td}"
    rm -rf /var/lib/apt/lists/*
    rm "${0}"
}

main "${@}"
