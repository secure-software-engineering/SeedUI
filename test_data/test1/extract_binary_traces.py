#!/usr/bin/python

from pathlib import Path
import os, glob, subprocess
import string

AFL_PATH = "../../external/GitAflplusplus"

BINARY = Path("./test_binary")
INPUTS_DIR = Path("./fuzzer_queue")
QEMU_BIN = Path(f"{AFL_PATH}/afl-qemu-trace")
QEMU_PLUGIN_DIR = Path(f"{QEMU_BIN.parent}/qemu_mode/qemuafl/build/contrib/plugins/libdrcov.so")
TRACE_DIR = Path("./traces")
for input in glob.glob(f"{INPUTS_DIR.absolute()}/*"):
    input_path = Path(input)
    if os.path.isdir(input_path):
        continue
    
    input_stem_str = str(input_path).split('/')[-1]
    input_stem_str = input_stem_str.replace(",", "::")
    
    input_content = ""
    with open(input_path.absolute(), "r") as f:
        input_content = f.readline()
    printable = set(string.printable)
    input_content = ''.join(filter(lambda x: x in printable, input_content))
    command = f"echo {input_content} | {QEMU_BIN.absolute()} -plugin {QEMU_PLUGIN_DIR.absolute()},arg=filename={TRACE_DIR.absolute()}/{input_stem_str}.trace -- {BINARY.absolute()}"
    try:
        subprocess.run(command, check=True, shell=True)
    except Exception as e:
        print(command)
        print(f"{e}\n")
        exit(1)