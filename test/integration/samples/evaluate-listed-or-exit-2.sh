#!/bin/sh
[ -f ../../../../exit-2-at ] && grep -q -x -e "$2" ../../../../exit-2-at && exit 2
[ -f "$1/solved" ] && grep -q -x -e "$2" "$1/solved"
