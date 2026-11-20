#!/bin/sh
for f in $*; do
    dir=$(dirname $f)
    mkdir -p $dir/backup
    cp $f $dir/backup/$(basename $f).bak
done
