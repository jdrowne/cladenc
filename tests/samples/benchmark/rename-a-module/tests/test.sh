#!/bin/sh
if [ ! -e old.sh ] && [ "$(sh new.sh)" = module ]; then echo 1; else echo 0; fi > /logs/verifier/reward.txt
