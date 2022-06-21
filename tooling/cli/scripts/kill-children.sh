#!/usr/bin/env sh

getcpid() {
    cpids=$(pgrep -P $1|xargs)
    for cpid in $cpids;
    do
        echo "$cpid"
        getcpid $cpid
    done
}

kill $(getcpid $1)
