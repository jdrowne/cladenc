#!/bin/sh
printf 'one\ntwo' > lines.txt
if [ "$(sh count.sh lines.txt)" -eq 2 ]; then echo 1; else echo 0; fi > /logs/verifier/reward.txt
