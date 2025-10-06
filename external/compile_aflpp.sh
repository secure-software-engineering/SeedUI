#!/bin/bash

git clone https://github.com/AFLplusplus/AFLplusplus.git GitAflplusplus
cd GitAflplusplus
git checkout b89727bea903aec80d003b6764fb53c232d33d95
cp ../rename_seeds.patch .
git apply rename_seeds.patch

CC=clang CXX=clang++ make
cd qemu_mode
CC=clang CXX=clang++ bash build_qemu_support.sh
cd qemuafl
CC=clang CXX=clang++ make plugins