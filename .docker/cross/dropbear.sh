#!/usr/bin/env bash

set -x
set -euo pipefail

# shellcheck disable=SC1091
. lib.sh

main() {
    local version=2022.82

    install_packages \
        autoconf \
        automake \
        bzip2 \
        curl \
        make

    if_centos install_packages zlib-devel
    if_ubuntu install_packages zlib1g-dev

    local td
    td="$(mktemp -d)"

    pushd "${td}"

    curl --retry 3 -sSfL "https://matt.ucc.asn.au/dropbear/releases/dropbear-${version}.tar.bz2" -O
    tar --strip-components=1 -xjf "dropbear-${version}.tar.bz2"

    # Remove some unwanted message
    sed -i '/skipping hostkey/d' cli-kex.c
    sed -i '/failed to identify current user/d' cli-runopts.c

    ./configure \
       --disable-syslog \
       --disable-shadow \
       --disable-lastlog \
       --disable-utmp \
       --disable-utmpx \
       --disable-wtmp \
       --disable-wtmpx \
       --disable-pututline \
       --disable-pututxline

    make "-j$(nproc)" PROGRAMS=dbclient
    cp dbclient /usr/local/bin/

    purge_packages

    popd

    rm -rf "${td}"
    rm "${0}"
}

main "${@}"
