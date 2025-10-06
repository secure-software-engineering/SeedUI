#!/usr/bin/python

from pathlib import Path
import os, glob, subprocess
import string

AFL_PATH = "../../external/GitAflplusplus"

BINARY = Path("./sources/test")
INPUTS_DIR = Path("./sources/inputs")
QEMU_BIN = Path(f"{AFL_PATH}/afl-qemu-trace")
QEMU_PLUGIN_DIR = Path(f"{QEMU_BIN.parent}/qemu_mode/qemuafl/build/contrib/plugins/libdrcov.so")
TRACE_DIR = Path("./traces")
for input in glob.glob(f"{INPUTS_DIR.absolute()}/*"):
    input_path = Path(input)
    if os.path.isdir(input_path):
        continue
    
    input_line = ""
    with open(input_path, "r") as inp:
        input_line = inp.readline().split(', ')
    print(f"input: {input}, input lines: {input_line}")

    input_stem_str = str(input_path).split('/')[-1]
    command = f"echo {int(input_line[0])}, {int(input_line[1])} | {QEMU_BIN.absolute()} -plugin {QEMU_PLUGIN_DIR.absolute()},arg=filename={TRACE_DIR.absolute()}/{input_stem_str}.trace -- {BINARY.absolute()} {input_path.absolute()}"
    try:
        subprocess.run(command, check=True, shell=True)
    except Exception as e:
        print(command)
        print(f"{e}\n")
        exit(1)