#!/bin/sh
[ -e ../../../../hold ] || exit 0
sleep 300 &
echo $! > pid
wait
