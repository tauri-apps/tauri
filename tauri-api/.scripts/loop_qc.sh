#!/bin/bash

# Loop all quickcheck tests. 
while true
do
    cargo test qc_
    if [[ x$? != x0 ]] ; then
        exit $?
    fi
done