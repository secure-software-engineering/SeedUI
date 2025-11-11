import argparse
from pathlib import Path
import shutil
import os

class RenameSeeds:
    def __init__(self, hfuzz_out, hfuzz_log, initial_seeds, output_folder):
        self.hfuzz_out = Path(hfuzz_out)
        self.hfuzz_log = Path(hfuzz_log)
        self.initial_seeds = Path(initial_seeds)
        self.output_folder = Path(output_folder)
        self.initial_seeds_map = {}
        self.filename_source_map = {}
    
    def parse_initial_seeds(self):
        initial_seed_id = 1
        for root, dirnames, filenames in os.walk(self.hfuzz_out):
            initial_seed_id += len(filenames)

        for f in self.initial_seeds.glob("*"):
            self.initial_seeds_map[f.name] = {'id': initial_seed_id, 'edges_found': 0, 'executed_on': 0}
            initial_seed_id += 1
    
    def get_equivalent_source(self, seed_src):
        seed_src_split = Path(seed_src).name.split(",")
        for filename in self.filename_source_map.keys():
            filename_split = Path(filename).name.split(",")
            similar_elements = set(seed_src_split).intersection(set(filename_split))
            if len(similar_elements) == len(seed_src_split) - 1:
                return filename
        return seed_src
    
    def rename(self):
        self.parse_initial_seeds()
        log_lines = None
        with open(self.hfuzz_log, "r") as hlog:
            log_lines = hlog.readlines()
        total_seeds = 0
        for log_line in log_lines:
            if log_line.startswith("Adding file"):
                seed_file_name = log_line.split(" '")[1].split("' ")[0]
                seed_file_name_split = Path(seed_file_name).stem.split(',')
                source_file_name = log_line.split("from source '")[-1].split("'")[0]
                curr_executed_on = int(seed_file_name_split[1].split(':')[-1])
                curr_edges_found = int(seed_file_name_split[2].split(':')[-1])
                assert(seed_file_name not in self.filename_source_map.keys())
                self.filename_source_map[seed_file_name] = {
                    'src': source_file_name,
                    'id': int(seed_file_name_split[0].split(':')[-1]),
                    'executed_on': curr_executed_on,
                    'edges_found': curr_edges_found,
                }

                total_seeds += 1

                if source_file_name in self.initial_seeds_map.keys():
                    if self.initial_seeds_map[source_file_name]['executed_on'] == 0:
                        self.initial_seeds_map[source_file_name]['executed_on'] = curr_executed_on
                        self.initial_seeds_map[source_file_name]['edges_found'] = curr_edges_found
                    elif self.initial_seeds_map[source_file_name]['executed_on'] > curr_executed_on:
                        self.initial_seeds_map[source_file_name]['executed_on'] = curr_executed_on
                        self.initial_seeds_map[source_file_name]['edges_found'] = curr_edges_found
        
        assert(total_seeds == len(self.filename_source_map.keys()))
        
        if self.output_folder.exists():
            shutil.rmtree(self.output_folder)
        
        self.output_folder.mkdir(parents=True)
        for initial_seed, details in self.initial_seeds_map.items():
            source_file = self.initial_seeds / initial_seed
            destination_file = self.output_folder / f"id:{details['id']},executed_on:{details['executed_on']},edges_found:{details['edges_found']},orig:{details['id']}"
            shutil.copy(source_file, destination_file)
        
        not_found = []
        for descendant, details in self.filename_source_map.items():
            source_path = Path(descendant)
            source_id = 0
            if details['src'] in self.initial_seeds_map.keys():
                source_id = self.initial_seeds_map[details['src']]['id']
            else:
                seed_src = f"{self.hfuzz_out.absolute()}/{details['src']}"
                if seed_src not in self.filename_source_map:
                    seed_src = self.get_equivalent_source(seed_src)
                source_id = self.filename_source_map[seed_src]['id']
            assert(details['id'] != source_id)
            destination_path = self.output_folder / f"cycle:1,id:{details['id']},executed_on:{details['executed_on']},src:{source_id},edges_found:{details['edges_found']}"
            shutil.copy(source_path, destination_path)
        
        with open(f"{self.hfuzz_log.parent}/renaming_errors.log", "w+") as err_file:
            for nf in not_found:
                err_file.write(f"Seed not converted: {nf}\n")
        
        print(f"total_seeds: {total_seeds}, errors: {len(not_found)}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Reduce data")
    parser.add_argument("--hfuzz-out", type=str, required=True, help="Folder where honggfuzz output files are present")
    parser.add_argument("--hfuzz-log", type=str, required=True, help="Log file from honggfuzz")
    parser.add_argument("--initial-seeds", type=str, required=True, help="Folder where initial seeds are present")
    parser.add_argument("--output-folder", type=str, required=True, help="Folder to store the renamed seeds")
    args = parser.parse_args()
    
    experiment = RenameSeeds(
        hfuzz_out=args.hfuzz_out,
        hfuzz_log=args.hfuzz_log,
        initial_seeds=args.initial_seeds,
        output_folder=args.output_folder,
    )
    experiment.rename()
