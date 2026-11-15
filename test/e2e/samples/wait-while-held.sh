#!/bin/sh
echo $$ > pid
while [ -e ../../../../hold ]; do sleep 0.05; done
