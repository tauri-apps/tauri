#!/bin/bash
# Copyright 2019-2021 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT


# Loop all quickcheck tests for tauri-api. 
while true
do
    cargo test qc_
    if [[ x$? != x0 ]] ; then
        exit $?
    fi
done
