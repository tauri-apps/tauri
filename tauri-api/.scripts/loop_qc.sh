#!/bin/bash

# Loop all quickcheck tests for tauri-api. 
while true
do
    cargo test qc_
    if [[ x$? != x0 ]] ; then
        exit $?
    fi
done