#!/bin/bash

git clone --depth 1 -b binutils-2_45 https://github.com/bminor/binutils-gdb.git

set -eux; \
    cd binutils-gdb; \
    mkdir build; \
    cd build && \
    CFLAGS="-Wno-error -DHAVE_SYS_STAT_H -DHAVE_SYS_WAIT_H -DHAVE_LIMITS_H -DHAVE_STDLIB_H -DHAVE_STRING_H -DHAVE_FCNTL_H -g" CC=clang CXX=clang++ ../configure --disable-shared --disable-gdb --disable-gdbserver --disable-gdbsupport --disable-libdecnumber --disable-ld --disable-gold --disable-gprof --disable-gprofng --disable-gas --disable-cpu --disable-intl --disable-libctf --disable-zlib --disable-texinfo --disable-sim --disable-readline --disable-libbacktrace; \
    make clean && make