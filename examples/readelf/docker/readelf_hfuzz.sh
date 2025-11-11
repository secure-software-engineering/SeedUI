#!/bin/bash

cd /home

export SUBJECT="/home/binutils-gdb-hfuzz"

cd "${SUBJECT}"

rm -rf obj-hfuzz
mkdir -p obj-hfuzz

export HFUZZ="/home/GitHonggfuzz"
export DRIVER_DIR="${SUBJECT}/obj-hfuzz"

$HFUZZ/honggfuzz --logfile "$DRIVER_DIR/hfuzz.log" -i "$1" --run_time $2 -o "$DRIVER_DIR/hfuzz-out" -- "${SUBJECT}/build/binutils/readelf" -a ___FILE___