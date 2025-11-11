#!/usr/bin/python

from pathlib import Path
import os, glob, subprocess

BINARY = Path("./local/binutils-gdb/build/binutils/readelf")
QEMU_BIN = Path("../../external/GitAflplusplus/afl-qemu-trace")
QEMU_PLUGIN_DIR = Path(f"{QEMU_BIN.parent}/qemu_mode/qemuafl/build/contrib/plugins/libdrcov.so")

for item in [("./saved_corpus/afl_run/afl-out/default/queue", "./saved_corpus/afl_traces"), ("./saved_corpus/hfuzz_run/renamed_seeds", "./saved_corpus/hfuzz_traces")]:
    INPUTS_DIR = Path(item[0])
    TRACE_DIR = Path(item[1])
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