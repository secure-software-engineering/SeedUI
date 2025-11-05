## SeedUI with Honggfuzz

### Run Honggfuzz inside Docker
In the following we explain the general knowledge of running Honggfuzz on a target.
We use `readelf` from `binutils-gdb` as a target.

We have provided some example scripts for fuzzing `readelf` in the `examples` directory.
We recommend running the fuzzing campaign in a docker container using the following commands:
  - The same docker image as AFL++ can be used as it also builds Honggfuzz. For convinience, we provide the docker build command here again:
    `docker build -t readelf_seed_ui -f examples/readelf/docker/Dockerfile.readelf .`
  - Start a docker container and run fuzzing campaigns in background:
    ```
    cd examples/readelf
    docker-compose up -d
    ```
    We have configured the default timeout of 5 minutes. One can change it to the desired timeout by updating the last command line parameter to the `readelf_hfuzz.sh` script in `examples/readelf/docker-compose.yaml` file.
    We have also configured the command to run `external/post_process_honggfuzz_inputs.py` script after the fuzzing campaign. So, you don't need to run it again.
  - After the fuzzing campaign, the generated corpus for each run can be found in the `examples/readelf/fuzzing_campaigns`. You may need to modify the folder permissions to access the files using the following command,
    ```
    sudo chmod -R 775 fuzzing_campaigns/*
    ```
  - Stop the docker containers:
    ```
    docker-compose down
    ```

### Run Honggfuzz locally

SeedUI expects the file names of the seeds generated during a fuzzing campaign contains a unique identifier, generated time, and its parent information.
As part of this repository, we provided a patch file, in `external/rename_seeds_honggfuzz.patch`, that modifies Honggfuzz source code to provide the identifier and generated time for each seed.
You need to apply this patch to Honggfuzz before running the fuzzing campaign.

The parent information is logged in a file during the fuzzing campaign.
We provided another script, `external/post_process_honggfuzz_inputs.py`, that parses the log and adds the parent information to the seed filenames. 
You need to run this script after the fuzzing campaign is finished as follows:
```python3
python3 post_process_honggfuzz_inputs.py --hfuzz-out=/path/to/hfuzz-out/ --hfuzz-log=/path/to/hfuzz.log --initial-seeds=/path/to/initial_seeds --output-folder=/path/to/store/renamed_seeds
```

After running the fuzzing campaign make sure you persist the `hfuzz-out` directory.