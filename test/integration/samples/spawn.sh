#!/bin/sh
sleep 300 &
echo $! > pid
wait
