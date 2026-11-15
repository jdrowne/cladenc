#!/bin/sh
touch "$2/solved"
echo "t$(($(wc -l < "$2/solved") + 1))" >> "$2/solved"
