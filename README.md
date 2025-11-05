# SeedUI

This repository contains the source code of the tool, **SeedUI: Understanding Initial Seeds in Fuzzing**, that is accepted at [1st International Workshop on Explainable Automated Software Engineering (Ex-ASE 2025)](https://exase.github.io/2025/).

A demonstration of SeedUI is available in [YouTube](https://youtu.be/qpPjutmIcTs).

- SeedUI requires some [pre-requisites](#pre-requisites) to be installed in your system.
- To skip running fuzzing campaigns and gathering fuzzing corpus, you can directly follow the [quick instructions](#seedui-in-action-short) which uses saved corpus in the `examples` directory.
- If you would like to know the complete sequence of steps that can work on any target that can be fuzzed using AFL++ or Honggfuzz, you can follow the [complete instructions](#seedui-in-action-long).

## Pre-requisites:
  - [Rust](https://www.rust-lang.org/tools/install)
  - [AflPlusPlus](https://github.com/AFLplusplus/AFLplusplus) compiled after applying `external/rename_seeds_afl.patch`.
    
    [or] 
    
    [Honggfuzz](https://github.com/google/honggfuzz.git) compiled after applying `external/rename_seeds_honggfuzz.patch`
  - [LLVM](https://github.com/llvm/llvm-project) >= 14.0.6
  - [npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm)

## SeedUI in action (short)
In general, a corpus should be extracted from fuzzing campaigns (more information in [run-fuzzers](#1-run-fuzzing-campaigns)).
However, to quickly look at SeedUI, we provide a fuzzing corpora that is extracted from two AFL++ fuzzing campaigns of 5 minutes on `readelf`.
We will use this saved corpus in the following steps.

There are three steps to run SeedUI, as follows:
  1. [Extract `drcov` traces for the saved corpus](#1-extract-line-coverage)
  2. [Start SeedUI server](#2-server)
  3. [Start SeedUI web client](#3-client)

### 1. Extract line coverage
  - Compile AFL++ in your local machine. SeedUI uses `afl-qemu-trace` to extract `drcov` trace information. You can use our handy script in the `external` directory for this purpose or use the instructions mentioned in [AFL++ Wiki]((https://github.com/AFLplusplus/AFLplusplus/tree/stable/qemu_mode#12-coverage-information)).
      ```
      cd external
      bash compile_aflpp.sh
      ```
  - Compile `binutils-gdb` with debug information enabled: usually with `-g` flag. We provide a handy script in the `examples/readelf/local` directory:
      ```
      cd examples/readelf/local
      bash compile_binutils_debug.sh
      ```
  - Extract `drcov` trace for saved corpus items using `afl-qemu-trace`. We also provided a python script to extract the line coverage for this corpus: `examples/readelf/extract_binary_traces.py`. Execute the following commands to extract line coverage:
      ```
      cd examples/readelf/
      unzip saved_corpus.zip

      mkdir traces
      python3 extract_binary_traces.py
      ```

### 2. Server
  - A configuration file needs to be specified to the server. We provide the configuration file for the example being discussed (`readelf`) in `examples/readelf/readelf.ron`. The description of the configuration options is explained [here](#3-server). To start the server with the configuration file, use the following commands:
    ```
    cd server
    cargo build --release
    ./target/release/server ../examples/readelf.ron

    [or]

    cd server
    cargo run -r --package server --bin server ../examples/readelf.ron
    ```

### 3. Client
  To start the web client, use the following commands in another terminal: 
  ```
  cd web
  npm install
  npm run build
  npm start
  ```
  
  Once the client is successfully build, you can access the web interface at `https://localhost:3000/`

## SeedUI in action (long)

There are four steps to run SeedUI, as follows:
  1. [Run fuzzing campaign(s) on a target](#1-run-fuzzing-campaigns)
  2. [Extract `drcov` traces for each of the seed saved in the corpus](#2-extract-line-coverage)
  3. [Start SeedUI server](#3-server)
  4. [Start SeedUI web client](#4-client)

### 1. Run fuzzing campaigns

  - Please refer to [Afl++ documentation](./docs/aflplusplus.md) for instructions to run AFL++.
  - Please refer to [Honggfuzz documentation](./docs/honggfuzz.md) for instructions to run Honggfuzz.

### 2. Extract line coverage
  - Compile AFL++ in your local machine. SeedUI uses `afl-qemu-trace` to extract `drcov` trace information. You can use our handy script in the `external` directory for this purpose or use the instructions mentioned in [AFL++ Wiki]((https://github.com/AFLplusplus/AFLplusplus/tree/stable/qemu_mode#12-coverage-information)).
      ```
      cd external
      bash compile_aflpp.sh
      ```
  - Compile the target being fuzzed `binutils-gdb` with debug information enabled: usually with `-g` flag. We provide a handy script for `readelf` in the `examples/readelf/local` directory:
      ```
      cd examples/readelf/local
      bash compile_binutils_debug.sh
      ```
  - Extract `drcov` trace for each of the corpus items using `afl-qemu-trace`. We also provided a handy python script for this purpose: `examples/extract_binary_traces_general.py`. Make sure to update the Binary path, Corpus path, Trace path in the python file before executing it. An example for extracting the traces is explained [here](#1-extract-line-coverage).

### 3. Server
  - A configuration file needs to be specified while starting the server. An example configuration for `readelf` is already provided in `examples/readelf/readelf.ron`. Here we explain different parts of the configuration file:
    ```yaml
    UserConfig(
      target_info: TargetConfig(
          target_path: "", # absolute path to the target binary compiled with DWARF debug information
          target_source_code_path: "", # absolute path to the source code of the target
          target_include_filter: [""], # absolute folder path(s) that contains source files to be included for line coverage
          allowed_extensions: [], # allowed file extension(s) to record line coverage: e.g., "c", "cpp"
      ),
      fuzzer_infos: [
          (
              fuzzer_configuration_id: 1, # incremental integer
              fuzzer_configuration: "", # desired name of the configuration
              traces_directory_path: "", # absolute path to the drcov traces directory
              inputs_directory_path: "", # absolute path to the queue folder of the AFL++ corpus
          ),
          ...
      ]
    )
    ```

  - The configuration file should then be passed to start the server as follows:
    ```
    cd server
    cargo build --release
    ./target/release/server path/to/configuration.ron
    ```

### 4. Client
  To start the web client, use the following commands in another terminal: 
  ```
  cd web
  npm install
  npm run build
  npm start
  ```
  
  Once the client is successfully build, you can access the web interface at `https://localhost:3000/`

  The web client contains five views as described in our paper: "SeedUI: Understanding Initial Seeds in Fuzzing".

## Developer notes
  The tests in the crates: `trace_map` and `inputs_database`, depend on the drcov trace files. Before running the tests, generate the corresponding trace files using the following commands:

  ```shell
  # For trace_map
  cd trace_map/test_data/sources && bash make.sh
  cd ..
  mkdir traces
  python3 extract_binary_traces.py
  ```

  ```shell
  # For inputs_database
  cd test_data/test1/ && bash make.sh
  cd ..
  mkdir traces
  python3 extract_binary_traces.py
  ```