#!/bin/sh
if [ "$(sh greet.sh --loud)" = HELLO ]; then echo 1; else echo 0; fi > /logs/verifier/reward.txt
