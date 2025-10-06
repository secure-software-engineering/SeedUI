#!/usr/bin/python

from pathlib import Path
import os, glob, subprocess

BINARY = Path("./path/to/binary") # TODO: provide absoulte binary path
INPUTS_DIR = Path(f"./path/to/corpus") # TODO: provide absoulte corpus path
TRACE_DIR = Path(f"./path/to/store/traces") # TODO: provide absoulte output folder to store the traces

QEMU_BIN = Path("../external/GitAflplusplus/afl-qemu-trace")
QEMU_PLUGIN_DIR = Path(f"{QEMU_BIN.parent}/qemu_mode/qemuafl/build/contrib/plugins/libdrcov.so")

if not TRACE_DIR.exists():
    os.makedirs(TRACE_DIR, exist_ok=False)
    
for input in glob.glob(f"{INPUTS_DIR.absolute()}/*"):
    input_path = Path(input)
    if os.path.isdir(input_path):
        continue
    
    input_stem_str = str(input_path).split('/')[-1]
    input_stem_str = input_stem_str.replace(",", "::")
    
    command = f"{QEMU_BIN.absolute()} -plugin {QEMU_PLUGIN_DIR.absolute()},arg=filename={TRACE_DIR.absolute()}/{input_stem_str}.trace -- {BINARY.absolute()} -a {input_path.absolute()}"
    try:
        subprocess.run(command, check=True, shell=True)
    except Exception as e:
        print(command)
        print(f"{e}\n")