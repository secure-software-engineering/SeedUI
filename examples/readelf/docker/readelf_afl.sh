#!/bin/bash

cd /home

export SUBJECT="/home/binutils-gdb"

cd "${SUBJECT}"

rm -rf obj-afl
mkdir -p obj-afl

export AFL="/home/GitAflplusplus"
export AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 AFL_NO_UI=1 AFL_QUIET=1 # AFL_DEBUG_CHILD=1 
export LLVM_COMPILER_PATH=/usr/local/llvm-14/bin
export LLVM_CONFIG=/usr/local/llvm-14/bin/llvm-config
export CC=$AFL/afl-clang-fast CXX=$AFL/afl-clang-fast++
export AFL_CC=/usr/local/llvm-14/bin/clang AFL_CXX=/usr/local/llvm-14/bin/clang++
export AFL_LLVM_INSTRUMENT=CLASSIC

export DRIVER_DIR="${SUBJECT}/obj-afl"

$AFL/afl-fuzz -i "$1" -o "$DRIVER_DIR/afl-out" -V $2 -- "${SUBJECT}/build/binutils/readelf" -a @@