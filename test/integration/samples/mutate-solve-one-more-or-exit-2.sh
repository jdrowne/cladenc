#!/bin/sh
job=$(basename "$(dirname "$PWD")")
[ -f ../../../../exit-2-at ] && grep -q -x -e "$job" ../../../../exit-2-at && exit 2
touch "$2/solved"
echo "t$(($(wc -l < "$2/solved") + 1))" >> "$2/solved"
