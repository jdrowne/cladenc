#!/bin/sh
[ -f "$1/solved" ] && grep -q -x -e "$2" "$1/solved"
