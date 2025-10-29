#!/bin/bash

cd /home

export SUBJECT="/home/binutils-gdb"

cd "${SUBJECT}"

rm -rf obj-hfuzz
mkdir -p obj-hfuzz

export HFUZZ="/home/GitHonggfuzz"
export DRIVER_DIR="${SUBJECT}/obj-hfuzz"

$HFUZZ/honggfuzz -i "$1" --run_time $2 -o "$DRIVER_DIR/hfuzz-out" -- "${SUBJECT}/build/binutils/readelf" -a ___FILE___