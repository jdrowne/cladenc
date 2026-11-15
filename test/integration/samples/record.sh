#!/bin/sh
printf '%s\n' "$@" > arguments
env > environment
cat > input
echo out
echo err >&2
