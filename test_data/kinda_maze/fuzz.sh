#!/bin/bash

export SUBJECT="./"

cd "${SUBJECT}"

rm -rf obj-afl
mkdir obj-afl

export AFL="../../external/GitAflplusplus"
export AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 AFL_NO_UI=1 AFL_QUIET=1 # AFL_DEBUG_CHILD=1 
export LLVM_COMPILER_PATH=/usr/local/llvm-14/bin
export LLVM_CONFIG=/usr/local/llvm-14/bin/llvm-config
export CC=$AFL/afl-clang-fast CXX=$AFL/afl-clang-fast++
export AFL_CC=/usr/local/llvm-14/bin/clang AFL_CXX=/usr/local/llvm-14/bin/clang++
export AFL_LLVM_INSTRUMENT=CLASSIC

$CC -I sources/ -o test_maze sources/test_maze.c

export DRIVER_DIR="${SUBJECT}/obj-afl"

# $AFL/afl-fuzz -i "$DRIVER_DIR/in/" -o "$DRIVER_DIR/afl-out" "${DRIVER_DIR}/test_1"
$AFL/afl-fuzz -i "$SUBJECT/inputs/" -o "$DRIVER_DIR/afl-out" -z -V 5 -- "${SUBJECT}/test_maze" @@