use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use config::{FuzzerConfig, TargetConfig};
use sut_database::SUT;
use trace_map::{Trace, TraceMap};
use custom_types::*;

#[derive(Clone)]
pub struct InputsDatabase {
    fuzzer_configurations: HashMap<u32, FuzzerConfig>,
    initial_seeds_input_meta_map: HashMap<u32, HashMap<InputId, InputMeta>>,
    fuzzer_id_initial_seeds_map: HashMap<u32, Vec<InputId>>,
    fuzzer_input_id_to_input_id_map: HashMap<(u32, u32), InputId>,
    input_id_to_trace_map: HashMap<InputId, Trace>,
    input_id_to_input_meta_map: HashMap<InputId, InputMeta>,
    fuzzer_id_input_id_map: HashMap<u32, Vec<InputId>>,
    fuzzer_id_initial_seeds_id_to_children_input_id_map: HashMap<(u32, u32), Vec<InputId>>,
    min_max_times: HashMap<u32, (i64, i64)>,
}

impl Default for InputsDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl InputsDatabase {
    pub fn new() -> Self {
        InputsDatabase {
            fuzzer_configurations: HashMap::new(),
            initial_seeds_input_meta_map: HashMap::new(),
            input_id_to_trace_map: HashMap::new(),
            input_id_to_input_meta_map: HashMap::new(),
            fuzzer_id_initial_seeds_map: HashMap::new(),
            fuzzer_id_input_id_map: HashMap::new(),
            fuzzer_input_id_to_input_id_map: HashMap::new(),
            fuzzer_id_initial_seeds_id_to_children_input_id_map: HashMap::new(),
            min_max_times: HashMap::new(),
        }
    }

    pub fn add_fuzzer_configuration(&mut self, fuzzer_config: &FuzzerConfig) {
        self.fuzzer_configurations
            .insert(fuzzer_config.fuzzer_configuration_id, fuzzer_config.clone());
    }

    pub fn add_input(
        &mut self,
        file_name: &str,
        target_config: &TargetConfig,
        sut_db: SUT,
        fuzzer_configuration_id: u32,
    ) -> SUT {
        if file_name.contains("orig") {
            self.add_initial_seed(file_name, target_config, sut_db, fuzzer_configuration_id)
        } else {
            self.add_trace_input(file_name, target_config, sut_db, fuzzer_configuration_id)
        }
    }

    fn record_line_coverage(
        &mut self,
        _input_id: &InputId,
        line_id: LineId,
        _executed_on: i64,
        fuzzer_configuration_id: u32,
        sut_db: &mut SUT,
    ) {
        let hits = sut_db.set_line_covered(line_id, fuzzer_configuration_id);
        // record unique line hits per file
        if hits == 1 {
            sut_db.increment_unique_line_hits(&line_id.file(), fuzzer_configuration_id);
        }
    }

    // id:000005,time:0,executed_on:1754925633575,execs:0,edges_found:8388608,orig:253.txt
    fn add_initial_seed(
        &mut self,
        file_name: &str,
        target_config: &TargetConfig,
        mut sut_db: SUT,
        fuzzer_configuration_id: u32,
    ) -> SUT {
        let trace_map = TraceMap::new(&target_config.target_path);
        let absolute_file_name = PathBuf::from(file_name);
        let file_stem = &absolute_file_name.file_stem().unwrap().to_str().unwrap();
        let file_stem_split: Vec<&str> = file_stem.split("::").collect();

        let input_id = InputId::new(&self.input_id_to_trace_map.len() + 1);
        let mut input_metadata = InputMeta::new();
        input_metadata.id = input_id;
        input_metadata.is_initial_seed = true;
        input_metadata.fuzzer_configuration = fuzzer_configuration_id;
        input_metadata.file_name_stem = file_stem.to_string().to_owned();
        for item in &file_stem_split[0..] {
            if item.contains("id") {
                input_metadata.fuzz_input_id =
                    item.split(':').nth(1).unwrap().parse::<u32>().unwrap();
            } else if item.contains("executed_on") {
                input_metadata.executed_on =
                    item.split(':').nth(1).unwrap().parse::<i64>().unwrap();
            }
        }

        let current_trace = trace_map.parse_with_config(
            fs::canonicalize(absolute_file_name)
                .unwrap()
                .to_str()
                .unwrap(),
            target_config,
            &mut sut_db,
        );

        // parse line coverage data
        for source_trace in &current_trace.unique_lines_set {
            if !sut_db.get_line_meta(*source_trace).unwrap().is_comment {
                self.record_line_coverage(
                    &input_id,
                    *source_trace,
                    input_metadata.executed_on,
                    fuzzer_configuration_id,
                    &mut sut_db,
                );
                input_metadata.source_line_coverage.insert(*source_trace);
            }
        }

        self.input_id_to_trace_map
            .insert(input_id, current_trace.clone());

        self.fuzzer_id_initial_seeds_map
            .entry(fuzzer_configuration_id)
            .or_default()
            .push(input_id);
        self.fuzzer_input_id_to_input_id_map.insert(
            (fuzzer_configuration_id, input_metadata.fuzz_input_id),
            input_id,
        );
        self.initial_seeds_input_meta_map
            .entry(fuzzer_configuration_id)
            .or_default()
            .insert(input_id, input_metadata);

        sut_db
    }

    // Assumes that the filename contains all the information from the fuzzer
    fn add_trace_input(
        &mut self,
        file_name: &str,
        target_config: &TargetConfig,
        mut sut_db: SUT,
        fuzzer_configuration_id: u32,
    ) -> SUT {
        let trace_map = TraceMap::new(&target_config.target_path);
        let absolute_file_name = PathBuf::from(file_name);
        let file_stem = absolute_file_name.file_stem().unwrap().to_str().unwrap();
        let file_stem_split: Vec<&str> = file_stem.split("::").collect();
        let executed_on = file_stem_split[2]
            .split(':')
            .nth(1)
            .unwrap()
            .parse::<i64>()
            .unwrap();

        if let std::collections::hash_map::Entry::Vacant(e) =
            self.min_max_times.entry(fuzzer_configuration_id)
        {
            e.insert((executed_on, executed_on));
        } else {
            let previous_times = self
                .min_max_times
                .get_mut(&fuzzer_configuration_id)
                .unwrap();
            if executed_on < previous_times.0 {
                previous_times.0 = executed_on;
            } else if executed_on > previous_times.1 {
                previous_times.1 = executed_on;
            }
        }

        let input_id = InputId::new(&self.input_id_to_trace_map.len() + 1);

        let mut input_metadata = InputMeta::new();
        input_metadata.fuzz_input_id = file_stem_split[1]
            .split(':')
            .nth(1)
            .unwrap()
            .parse::<u32>()
            .unwrap();
        input_metadata.fuzzer_configuration = fuzzer_configuration_id;
        input_metadata.id = input_id;
        input_metadata.file_name_stem = file_stem.to_string().to_owned();
        input_metadata.executed_on = executed_on;
        for item in &file_stem_split[2..] {
            if item.contains("time") {
                input_metadata.execution_time =
                    item.split(':').nth(1).unwrap().parse::<i64>().unwrap();
            } else if item.contains("execs") {
                input_metadata.total_mutations_required_to_generate =
                    item.split(':').nth(1).unwrap().parse::<u32>().unwrap();
            } else if item.contains("edges_found") {
                input_metadata.fuzzer_coverage =
                    item.split(':').nth(1).unwrap().parse::<u32>().unwrap();
            } else if item.contains("src") {
                input_metadata.parents = item
                    .split(':')
                    .nth(1)
                    .unwrap()
                    .split('+')
                    .filter_map(|s| s.trim().parse::<u32>().ok())
                    .collect();
            }
        }

        let current_trace = trace_map.parse_with_config(
            fs::canonicalize(absolute_file_name)
                .unwrap()
                .to_str()
                .unwrap(),
            target_config,
            &mut sut_db,
        );

        // parse line coverage data
        for source_trace in &current_trace.unique_lines_set {
            if !sut_db.get_line_meta(*source_trace).unwrap().is_comment {
                self.record_line_coverage(
                    &input_id,
                    *source_trace,
                    executed_on,
                    fuzzer_configuration_id,
                    &mut sut_db,
                );
                input_metadata.source_line_coverage.insert(*source_trace);
            }
        }

        self.input_id_to_trace_map
            .insert(input_id, current_trace.clone());

        self.fuzzer_id_input_id_map
            .entry(fuzzer_configuration_id)
            .or_default()
            .push(input_id);
        self.fuzzer_input_id_to_input_id_map.insert(
            (fuzzer_configuration_id, input_metadata.fuzz_input_id),
            input_id,
        );
        self.input_id_to_input_meta_map
            .insert(input_id, input_metadata);

        sut_db
    }

    pub fn post_process(&mut self) {
        for (input_id, input_meta) in self.input_id_to_input_meta_map.iter() {
            let parents =
                self.get_initial_seed_parents_for(input_id, &input_meta.fuzzer_configuration);
            for parent in parents.iter() {
                let parent_meta = self
                    .initial_seeds_input_meta_map
                    .get(&input_meta.fuzzer_configuration)
                    .unwrap()
                    .get(parent)
                    .unwrap();
                self.fuzzer_id_initial_seeds_id_to_children_input_id_map
                    .entry((input_meta.fuzzer_configuration, parent_meta.fuzz_input_id))
                    .or_default()
                    .push(*input_id);
            }
        }
    }

    fn get_raw_bytes_for_input(&self, fuzzer_config: &FuzzerConfig, file_stem: &str) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        let file_path =
            Path::new(&fuzzer_config.inputs_directory_path).join(file_stem.replace("::", ","));
        let _ = File::open(file_path).unwrap().read_to_end(&mut buffer);
        buffer
    }

    pub fn compare_inputs(
        &self,
        configuration_id: &u32,
        initial_seed_id: &u32,
    ) -> HashMap<usize, u32> {
        let mut previous_byte_values: HashMap<usize, u8> = HashMap::new();
        let mut ret: HashMap<usize, u32> = HashMap::new();
        let fuzzer_config = self.fuzzer_configurations.get(configuration_id).unwrap();
        let initial_seed_input_id = self
            .fuzzer_input_id_to_input_id_map
            .get(&(*configuration_id, *initial_seed_id))
            .unwrap();
        let current_initial_seed_meta = self
            .initial_seeds_input_meta_map
            .get(configuration_id)
            .unwrap()
            .get(initial_seed_input_id)
            .unwrap();
        let initial_seed_raw_bytes =
            self.get_raw_bytes_for_input(fuzzer_config, &current_initial_seed_meta.file_name_stem);
        for (index, raw_value) in initial_seed_raw_bytes.iter().enumerate() {
            previous_byte_values.insert(index, *raw_value);
            ret.insert(index, 0);
        }

        let corresponding_input_ids = self
            .fuzzer_id_initial_seeds_id_to_children_input_id_map
            .get(&(*configuration_id, *initial_seed_id))
            .unwrap();
        for input_id in corresponding_input_ids {
            let current_input_meta = self.input_id_to_input_meta_map.get(input_id).unwrap();
            let current_seed_raw_bytes =
                self.get_raw_bytes_for_input(fuzzer_config, &current_input_meta.file_name_stem);

            for (index, raw_value) in current_seed_raw_bytes.iter().enumerate() {
                if previous_byte_values.contains_key(&index) {
                    if previous_byte_values.get(&index).unwrap() != raw_value {
                        *ret.get_mut(&index).unwrap() += 1;
                    }
                } else {
                    ret.insert(index, 1);
                }
                previous_byte_values.insert(index, *raw_value);
            }
        }

        ret
    }

    pub fn get_fuzzer_configuration(&self, configuration_id: &u32) -> Option<&FuzzerConfig> {
        self.fuzzer_configurations.get(configuration_id)
    }

    pub fn get_all_fuzzer_configurations(&self) -> &HashMap<u32, FuzzerConfig> {
        &self.fuzzer_configurations
    }

    pub fn get_all_initial_seeds_for_fuzzer_id(&self, fuzzer_id: &u32) -> Vec<&InputId> {
        self.initial_seeds_input_meta_map
            .get(fuzzer_id)
            .unwrap()
            .keys()
            .collect()
    }

    pub fn get_all_inputs_for_fuzzer_id(&self, fuzzer_id: &u32) -> &Vec<InputId> {
        self.fuzzer_id_input_id_map.get(fuzzer_id).unwrap()
    }

    pub fn get_all_inputs_meta_info(&self) -> &HashMap<InputId, InputMeta> {
        &self.input_id_to_input_meta_map
    }

    pub fn get_inputs_meta_info_for(&self, input_id: &InputId) -> &InputMeta {
        self.input_id_to_input_meta_map.get(input_id).unwrap()
    }

    pub fn get_input_id_for(&self, fuzzer_configuration_id: &u32, fuzz_input_id: &u32) -> &InputId {
        self.fuzzer_input_id_to_input_id_map
            .get(&(*fuzzer_configuration_id, *fuzz_input_id))
            .unwrap()
    }

    pub fn get_all_initial_seeds_meta_info(
        &self,
        fuzzer_configuration_id: &u32,
    ) -> &HashMap<InputId, InputMeta> {
        self.initial_seeds_input_meta_map
            .get(fuzzer_configuration_id)
            .unwrap()
    }

    pub fn get_trace_for(&self, input_id: &InputId) -> &Trace {
        self.input_id_to_trace_map.get(input_id).unwrap()
    }

    fn get_parents_for(
        &self,
        input_id: &InputId,
        fuzzer_configuration_id: &u32,
        ret_val: &mut HashSet<InputId>,
    ) {
        if self
            .get_all_initial_seeds_for_fuzzer_id(fuzzer_configuration_id)
            .contains(&input_id)
        {
            return;
        }

        let current_input_meta = self.input_id_to_input_meta_map.get(input_id).unwrap();
        if !current_input_meta.parents.is_empty() {
            for parent in current_input_meta.parents.iter() {
                let parent_input_id = self
                    .fuzzer_input_id_to_input_id_map
                    .get(&(*fuzzer_configuration_id, *parent))
                    .unwrap();
                if self
                    .input_id_to_input_meta_map
                    .contains_key(parent_input_id)
                {
                    self.get_parents_for(parent_input_id, fuzzer_configuration_id, ret_val);
                } else if self
                    .get_all_initial_seeds_for_fuzzer_id(fuzzer_configuration_id)
                    .contains(&parent_input_id)
                {
                    ret_val.insert(*parent_input_id);
                }
            }
        }
    }

    pub fn get_initial_seed_parents_for(
        &self,
        input_id: &InputId,
        fuzzer_configuration_id: &u32,
    ) -> HashSet<InputId> {
        let mut ret: HashSet<InputId> = HashSet::new();
        self.get_parents_for(input_id, fuzzer_configuration_id, &mut ret);
        ret
    }

    pub fn has_children_for(&self, fuzzer_id: &u32, initial_seed_id: &u32) -> bool {
        self.fuzzer_id_initial_seeds_id_to_children_input_id_map
            .contains_key(&(*fuzzer_id, *initial_seed_id))
    }

    pub fn get_all_children_input_ids_for(
        &self,
        fuzzer_id: &u32,
        initial_seed_ids: &Vec<u32>,
    ) -> Vec<InputId> {
        let mut ret_val: HashSet<InputId> = HashSet::new();
        for &seed_id in initial_seed_ids.iter() {
            if ret_val.is_empty() {
                ret_val.extend(
                    self.fuzzer_id_initial_seeds_id_to_children_input_id_map
                        .get(&(*fuzzer_id, seed_id))
                        .unwrap(),
                );
            } else {
                let curr_children: HashSet<InputId> = self
                    .fuzzer_id_initial_seeds_id_to_children_input_id_map
                    .get(&(*fuzzer_id, seed_id))
                    .unwrap()
                    .iter()
                    .cloned()
                    .collect();
                ret_val = ret_val.intersection(&curr_children).cloned().collect();
            }
        }

        ret_val.into_iter().collect()
    }

    pub fn get_initial_seed_line_coverage_for_file_id(
        &self,
        fuzzer_configuration_id: &u32,
        initial_seed_id: &u32,
        file_id: &FileId,
        sut_db: &SUT,
    ) -> Vec<LineMeta> {
        let mut line_coverage: Vec<LineMeta> = Vec::new();

        let initial_seed_meta = self
            .initial_seeds_input_meta_map
            .get(fuzzer_configuration_id)
            .unwrap()
            .iter()
            .find(|(&_x, y)| y.fuzz_input_id == *initial_seed_id)
            .unwrap()
            .1;

        for line in initial_seed_meta.source_line_coverage.iter() {
            if line.file() == *file_id {
                line_coverage.push(sut_db.get_line_meta(*line).unwrap().clone());
            }
        }

        line_coverage
    }

    pub fn get_all_children_line_coverage_for_file_id(
        &self,
        fuzzer_configuration_id: &u32,
        initial_seed_id: &u32,
        file_id: &FileId,
        sut_db: &SUT,
    ) -> HashMap<InputId, Vec<LineMeta>> {
        let mut line_coverage: HashMap<InputId, Vec<LineMeta>> = HashMap::new();

        if !self.has_children_for(fuzzer_configuration_id, initial_seed_id) {
            return line_coverage;
        }

        for child in self
            .get_all_children_input_ids_for(fuzzer_configuration_id, &vec![*initial_seed_id])
            .iter()
        {
            let mut current_line_coverage: HashSet<LineMeta> = HashSet::new();
            for line in self
                .input_id_to_input_meta_map
                .get(child)
                .unwrap()
                .source_line_coverage
                .iter()
            {
                if line.file() == *file_id {
                    current_line_coverage.insert(sut_db.get_line_meta(*line).unwrap().clone());
                }
            }

            if !current_line_coverage.is_empty() {
                line_coverage.insert(*child, current_line_coverage.iter().cloned().collect());
            }
        }

        line_coverage
    }

    pub fn get_run_times_for_fuzzer_id(&self, fuzzer_configuration_id: &u32) -> &(i64, i64) {
        self.min_max_times.get(fuzzer_configuration_id).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_fuzzer_config_1() {
        let config = FuzzerConfig {
            fuzzer_configuration_id: 42,
            traces_directory_path: "Hello".to_string(),
            inputs_directory_path: "Hello".to_string(),
            fuzzer_configuration: "World".to_string(),
        };
        let mut input_db = InputsDatabase::new();
        input_db.add_fuzzer_configuration(&config);

        assert_eq!(
            input_db
                .get_fuzzer_configuration(&42)
                .unwrap()
                .fuzzer_configuration,
            config.fuzzer_configuration
        );
    }

    #[test]
    fn test_input_1() {
        let path = env::current_dir().unwrap();
        let test_data_dir = fs::canonicalize(PathBuf::from(
            path.join("../test_data/test1/")
                .to_string_lossy()
                .into_owned(),
        ))
        .unwrap();
        let config = TargetConfig {
            target_path: test_data_dir
                .join("test_binary")
                .to_string_lossy()
                .into_owned(),
            target_source_code_path: String::from(
                fs::canonicalize(PathBuf::from(
                    test_data_dir.join("sources").to_string_lossy().into_owned(),
                ))
                .unwrap()
                .to_str()
                .unwrap(),
            ),
            target_include_filter: vec![],
            allowed_extensions: vec![],
        };

        let mut input_db = InputsDatabase::new();
        let mut sut_db = SUT::new();
        sut_db.parse_config(&config);
        let _sut_db = input_db.add_input(test_data_dir.join("traces/cycle:1::id:000002::executed_on:1753701941117::src:000001::time:191::execs:378::op:havoc::rep:2::+cov::gain:2::score:100::depth:1::bitmap_cvg:0.02::edges_found:123.trace").to_string_lossy().into_owned().as_str(),  &config, sut_db, 0);

        assert_eq!(input_db.input_id_to_input_meta_map.len(), 1);
        assert_eq!(
            input_db
                .input_id_to_input_meta_map
                .get(&InputId::new(1))
                .unwrap()
                .fuzzer_coverage,
            123
        );
    }

    #[test]
    fn test_input_2() {
        let path = env::current_dir().unwrap();
        let test_data_dir = fs::canonicalize(PathBuf::from(
            path.join("../test_data/test1/")
                .to_string_lossy()
                .into_owned(),
        ))
        .unwrap();
        let config = TargetConfig {
            target_path: test_data_dir
                .join("test_binary")
                .to_string_lossy()
                .into_owned(),
            target_source_code_path: String::from(
                fs::canonicalize(PathBuf::from(
                    test_data_dir.join("sources").to_string_lossy().into_owned(),
                ))
                .unwrap()
                .to_str()
                .unwrap(),
            ),
            target_include_filter: vec![],
            allowed_extensions: vec![],
        };

        let mut input_db = InputsDatabase::new();
        let mut sut_db = SUT::new();
        sut_db.parse_config(&config);
        let sut_db = input_db.add_input(test_data_dir.join("traces/cycle:2::id:000005::executed_on:1753701941381::src:000002::time:455::execs:880::op:havoc::rep:1::gain:1::score:200::depth:2::bitmap_cvg:0.02::edges_found:123.trace").to_string_lossy().into_owned().as_str(), &config, sut_db, 0);
        println!("{:?}", sut_db);

        let mut file_meta_gt = FileMeta::new("test_data/test1/sources/main.c");
        let file_id = FileId::new(0);
        file_meta_gt.lines = HashSet::from([
            LineId::new(file_id, 7),
            LineId::new(file_id, 17),
            LineId::new(file_id, 10),
            LineId::new(file_id, 11),
            LineId::new(file_id, 12),
            LineId::new(file_id, 13),
            LineId::new(file_id, 14),
            LineId::new(file_id, 8),
            LineId::new(file_id, 9),
            LineId::new(file_id, 4),
        ]);

        assert_eq!(
            input_db
                .input_id_to_input_meta_map
                .get(&InputId::new(1))
                .unwrap()
                .fuzzer_coverage,
            123
        );
    }

    #[test]
    fn test_input_multiple_1() {
        let path = env::current_dir().unwrap();
        let test_data_dir = fs::canonicalize(PathBuf::from(
            path.join("../test_data/test1/")
                .to_string_lossy()
                .into_owned(),
        ))
        .unwrap();
        let config = TargetConfig {
            target_path: test_data_dir
                .join("test_binary")
                .to_string_lossy()
                .into_owned(),
            target_source_code_path: String::from(
                fs::canonicalize(PathBuf::from(
                    test_data_dir.join("sources").to_string_lossy().into_owned(),
                ))
                .unwrap()
                .to_str()
                .unwrap(),
            ),
            target_include_filter: vec![],
            allowed_extensions: vec![],
        };

        let mut input_db = InputsDatabase::new();
        let mut sut_db = SUT::new();
        sut_db.parse_config(&config);
        let sut_db = input_db.add_input(test_data_dir.join("traces/cycle:2::id:000003::executed_on:1753701941262::src:000002::time:336::execs:656::op:havoc::rep:1::+cov::gain:2::score:200::depth:2::bitmap_cvg:0.02::edges_found:123.trace").to_string_lossy().into_owned().as_str(),  &config, sut_db, 0);
        let _sut_db = input_db.add_input(test_data_dir.join("traces/cycle:2::id:000007::executed_on:1753701941458::src:000002::time:532::execs:1020::op:havoc::rep:4::gain:1::score:200::depth:2::bitmap_cvg:0.02::edges_found:123.trace").to_string_lossy().into_owned().as_str(),  &config, sut_db, 0);

        let input_meta = input_db
            .get_all_inputs_meta_info()
            .get(&InputId::new(1))
            .unwrap();
        assert_eq!(input_meta.fuzzer_coverage, 123);
    }

    #[test]
    fn test_initial_seed_1() {
        let path = env::current_dir().unwrap();
        let test_data_dir = fs::canonicalize(PathBuf::from(
            path.join("../test_data/test1/")
                .to_string_lossy()
                .into_owned(),
        ))
        .unwrap();
        let config = TargetConfig {
            target_path: test_data_dir
                .join("test_binary")
                .to_string_lossy()
                .into_owned(),
            target_source_code_path: String::from(
                fs::canonicalize(PathBuf::from(
                    test_data_dir.join("sources").to_string_lossy().into_owned(),
                ))
                .unwrap()
                .to_str()
                .unwrap(),
            ),
            target_include_filter: vec![],
            allowed_extensions: vec![],
        };

        let mut input_db = InputsDatabase::new();
        let mut sut_db = SUT::new();
        sut_db.parse_config(&config);
        let _sut_db = input_db.add_initial_seed(
            test_data_dir
                .join("traces/id:000000::time:0::executed_on:1753701940885::execs:0::orig:a.trace")
                .to_string_lossy()
                .into_owned()
                .as_str(),
            &config,
            sut_db,
            0,
        );

        let input_meta = input_db.get_all_initial_seeds_meta_info(&0);

        let initial_seed_trace = input_db.get_trace_for(input_meta.keys().next().unwrap());

        assert_eq!(input_db.get_all_initial_seeds_meta_info(&0).len(), 1);
        assert_eq!(
            input_meta.values().next().unwrap().executed_on,
            1753701940885
        );
        assert!(input_meta.values().next().unwrap().is_initial_seed);
        assert_eq!(
            input_meta.values().next().unwrap().file_name_stem,
            "id:000000::time:0::executed_on:1753701940885::execs:0::orig:a"
        );
        // unique_lines can be greater than source.len() because source is a combination of start and end!
        assert_eq!(initial_seed_trace.unique_lines_set.len(), 13);
        assert_eq!(initial_seed_trace.source.len(), 8);
    }

    #[test]
    fn test_initial_seed_multiple() {
        let path = env::current_dir().unwrap();
        let test_data_dir = fs::canonicalize(PathBuf::from(
            path.join("../test_data/test1/")
                .to_string_lossy()
                .into_owned(),
        ))
        .unwrap();
        let config = TargetConfig {
            target_path: test_data_dir
                .join("test_binary")
                .to_string_lossy()
                .into_owned(),
            target_source_code_path: String::from(
                fs::canonicalize(PathBuf::from(
                    test_data_dir.join("sources").to_string_lossy().into_owned(),
                ))
                .unwrap()
                .to_str()
                .unwrap(),
            ),
            target_include_filter: vec![],
            allowed_extensions: vec![],
        };

        let mut input_db = InputsDatabase::new();
        let mut sut_db = SUT::new();
        sut_db.parse_config(&config);
        let sut_db = input_db.add_initial_seed(
            test_data_dir
                .join("traces/id:000000::time:0::executed_on:1753701940885::execs:0::orig:a.trace")
                .to_string_lossy()
                .into_owned()
                .as_str(),
            &config,
            sut_db,
            0,
        );
        let _sut_db = input_db.add_initial_seed(
            test_data_dir
                .join("traces/id:000001::time:0::executed_on:1753701940885::execs:0::orig:b.trace")
                .to_string_lossy()
                .into_owned()
                .as_str(),
            &config,
            sut_db,
            0,
        );

        let input_meta = input_db.get_all_initial_seeds_meta_info(&0);

        let initial_seed_trace = input_db.get_trace_for(input_meta.keys().next().unwrap());

        assert_eq!(input_db.get_all_initial_seeds_meta_info(&0).len(), 2);
        assert_eq!(
            input_meta.values().next().unwrap().executed_on,
            1753701940885
        );
        assert!(input_meta.values().next().unwrap().is_initial_seed);
        // unique_lines can be greater than source.len() because source is a combination of start and end!
        assert_eq!(initial_seed_trace.unique_lines_set.len(), 13);
        assert_eq!(initial_seed_trace.source.len(), 8);
    }

    #[test]
    fn test_parent() {
        let path = env::current_dir().unwrap();
        let test_data_dir = fs::canonicalize(PathBuf::from(
            path.join("../test_data/test1/")
                .to_string_lossy()
                .into_owned(),
        ))
        .unwrap();
        let config = TargetConfig {
            target_path: test_data_dir
                .join("test_binary")
                .to_string_lossy()
                .into_owned(),
            target_source_code_path: String::from(
                fs::canonicalize(PathBuf::from(
                    test_data_dir.join("sources").to_string_lossy().into_owned(),
                ))
                .unwrap()
                .to_str()
                .unwrap(),
            ),
            target_include_filter: vec![],
            allowed_extensions: vec![],
        };

        let mut input_db = InputsDatabase::new();
        let mut sut_db = SUT::new();
        sut_db.parse_config(&config);
        let sut_db = input_db.add_initial_seed(
            test_data_dir
                .join("traces/id:000000::time:0::executed_on:1753701940885::execs:0::orig:a.trace")
                .to_string_lossy()
                .into_owned()
                .as_str(),
            &config,
            sut_db,
            0,
        );
        let sut_db = input_db.add_initial_seed(
            test_data_dir
                .join("traces/id:000001::time:0::executed_on:1753701940885::execs:0::orig:b.trace")
                .to_string_lossy()
                .into_owned()
                .as_str(),
            &config,
            sut_db,
            0,
        );
        let sut_db = input_db.add_input(test_data_dir.join("traces/cycle:1::id:000002::executed_on:1753701941117::src:000001::time:191::execs:378::op:havoc::rep:2::+cov::gain:2::score:100::depth:1::bitmap_cvg:0.02::edges_found:123.trace").to_string_lossy().into_owned().as_str(),  &config, sut_db, 0);
        let _sut_db = input_db.add_input(test_data_dir.join("traces/cycle:2::id:000007::executed_on:1753701941458::src:000002::time:532::execs:1020::op:havoc::rep:4::gain:1::score:200::depth:2::bitmap_cvg:0.02::edges_found:123.trace").to_string_lossy().into_owned().as_str(),  &config, sut_db, 0);

        assert_eq!(input_db.get_all_initial_seeds_meta_info(&0).len(), 2);
        let mut check_parent = input_db.get_initial_seed_parents_for(&InputId::new(4), &0);
        assert_eq!(check_parent.len(), 1);
        assert_eq!(*check_parent.iter().next().unwrap(), InputId::new(2));

        check_parent = input_db.get_initial_seed_parents_for(&InputId::new(3), &0);
        assert_eq!(check_parent.len(), 1);
        assert_eq!(*check_parent.iter().next().unwrap(), InputId::new(2));
    }

    #[test]
    fn test_all_children() {
        let path = env::current_dir().unwrap();
        let test_data_dir = fs::canonicalize(PathBuf::from(
            path.join("../test_data/test1/")
                .to_string_lossy()
                .into_owned(),
        ))
        .unwrap();
        let config = TargetConfig {
            target_path: test_data_dir
                .join("test_binary")
                .to_string_lossy()
                .into_owned(),
            target_source_code_path: String::from(
                fs::canonicalize(PathBuf::from(
                    test_data_dir.join("sources").to_string_lossy().into_owned(),
                ))
                .unwrap()
                .to_str()
                .unwrap(),
            ),
            target_include_filter: vec![],
            allowed_extensions: vec![],
        };

        let mut input_db = InputsDatabase::new();
        let mut sut_db = SUT::new();
        sut_db.parse_config(&config);
        let sut_db = input_db.add_initial_seed(
            test_data_dir
                .join("traces/id:000000::time:0::executed_on:1753701940885::execs:0::orig:a.trace")
                .to_string_lossy()
                .into_owned()
                .as_str(),
            &config,
            sut_db,
            0,
        );
        let sut_db = input_db.add_initial_seed(
            test_data_dir
                .join("traces/id:000001::time:0::executed_on:1753701940885::execs:0::orig:b.trace")
                .to_string_lossy()
                .into_owned()
                .as_str(),
            &config,
            sut_db,
            0,
        );
        let sut_db = input_db.add_input(test_data_dir.join("traces/cycle:1::id:000002::executed_on:1753701941117::src:000001::time:191::execs:378::op:havoc::rep:2::+cov::gain:2::score:100::depth:1::bitmap_cvg:0.02::edges_found:123.trace").to_string_lossy().into_owned().as_str(),  &config, sut_db, 0);
        let _sut_db = input_db.add_input(test_data_dir.join("traces/cycle:2::id:000007::executed_on:1753701941458::src:000002::time:532::execs:1020::op:havoc::rep:4::gain:1::score:200::depth:2::bitmap_cvg:0.02::edges_found:123.trace").to_string_lossy().into_owned().as_str(),  &config, sut_db, 0);
        input_db.post_process();

        println!(
            "{:?}",
            input_db.fuzzer_id_initial_seeds_id_to_children_input_id_map
        );
        let children = input_db.get_all_children_input_ids_for(&0, &vec![1]);
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_initial_seed_parent() {
        let path = env::current_dir().unwrap();
        let test_data_dir = fs::canonicalize(PathBuf::from(
            path.join("../test_data/test1/")
                .to_string_lossy()
                .into_owned(),
        ))
        .unwrap();
        let config = TargetConfig {
            target_path: test_data_dir
                .join("test_binary")
                .to_string_lossy()
                .into_owned(),
            target_source_code_path: String::from(
                fs::canonicalize(PathBuf::from(
                    test_data_dir.join("sources").to_string_lossy().into_owned(),
                ))
                .unwrap()
                .to_str()
                .unwrap(),
            ),
            target_include_filter: vec![],
            allowed_extensions: vec![],
        };

        let mut input_db = InputsDatabase::new();
        let mut sut_db = SUT::new();
        sut_db.parse_config(&config);
        let sut_db = input_db.add_initial_seed(
            test_data_dir
                .join("traces/id:000000::time:0::executed_on:1753701940885::execs:0::orig:a.trace")
                .to_string_lossy()
                .into_owned()
                .as_str(),
            &config,
            sut_db,
            0,
        );
        let sut_db = input_db.add_initial_seed(
            test_data_dir
                .join("traces/id:000001::time:0::executed_on:1753701940885::execs:0::orig:b.trace")
                .to_string_lossy()
                .into_owned()
                .as_str(),
            &config,
            sut_db,
            0,
        );
        let sut_db = input_db.add_input(test_data_dir.join("traces/cycle:1::id:000002::executed_on:1753701941117::src:000001::time:191::execs:378::op:havoc::rep:2::+cov::gain:2::score:100::depth:1::bitmap_cvg:0.02::edges_found:123.trace").to_string_lossy().into_owned().as_str(), &config, sut_db, 0);
        let _sut_db = input_db.add_input(test_data_dir.join("traces/cycle:2::id:000007::executed_on:1753701941458::src:000002::time:532::execs:1020::op:havoc::rep:4::gain:1::score:200::depth:2::bitmap_cvg:0.02::edges_found:123.trace").to_string_lossy().into_owned().as_str(), &config, sut_db, 0);

        assert_eq!(input_db.get_all_initial_seeds_meta_info(&0).len(), 2);
        let check_parent = input_db.get_initial_seed_parents_for(&InputId::new(1), &0);
        assert_eq!(check_parent.len(), 0);
    }

    #[test]
    fn test_compare_inputs() {
        let path = env::current_dir().unwrap();
        let test_data_dir = fs::canonicalize(PathBuf::from(
            path.join("../test_data/test1/")
                .to_string_lossy()
                .into_owned(),
        ))
        .unwrap();

        let fuzzer_config = FuzzerConfig {
            fuzzer_configuration: "test".to_string(),
            traces_directory_path: String::from(
                fs::canonicalize(PathBuf::from(
                    test_data_dir.join("traces").to_string_lossy().into_owned(),
                ))
                .unwrap()
                .to_str()
                .unwrap(),
            ),
            inputs_directory_path: String::from(
                fs::canonicalize(PathBuf::from(
                    test_data_dir
                        .join("fuzzer_queue")
                        .to_string_lossy()
                        .into_owned(),
                ))
                .unwrap()
                .to_str()
                .unwrap(),
            ),
            fuzzer_configuration_id: 0,
        };

        let config = TargetConfig {
            target_path: test_data_dir
                .join("test_binary")
                .to_string_lossy()
                .into_owned(),
            target_source_code_path: String::from(
                fs::canonicalize(PathBuf::from(
                    test_data_dir.join("sources").to_string_lossy().into_owned(),
                ))
                .unwrap()
                .to_str()
                .unwrap(),
            ),
            target_include_filter: vec![],
            allowed_extensions: vec![],
        };

        let mut input_db = InputsDatabase::new();
        input_db.add_fuzzer_configuration(&fuzzer_config);

        let mut sut_db = SUT::new();
        sut_db.parse_config(&config);
        let sut_db = input_db.add_initial_seed(
            test_data_dir
                .join("traces/id:000000::time:0::executed_on:1753701940885::execs:0::orig:a.trace")
                .to_string_lossy()
                .into_owned()
                .as_str(),
            &config,
            sut_db,
            0,
        );
        let sut_db = input_db.add_initial_seed(
            test_data_dir
                .join("traces/id:000001::time:0::executed_on:1753701940885::execs:0::orig:b.trace")
                .to_string_lossy()
                .into_owned()
                .as_str(),
            &config,
            sut_db,
            0,
        );
        let sut_db = input_db.add_input(test_data_dir.join("traces/cycle:1::id:000002::executed_on:1753701941117::src:000001::time:191::execs:378::op:havoc::rep:2::+cov::gain:2::score:100::depth:1::bitmap_cvg:0.02::edges_found:123.trace").to_string_lossy().into_owned().as_str(), &config, sut_db, 0);
        let _sut_db = input_db.add_input(test_data_dir.join("traces/cycle:2::id:000007::executed_on:1753701941458::src:000002::time:532::execs:1020::op:havoc::rep:4::gain:1::score:200::depth:2::bitmap_cvg:0.02::edges_found:123.trace").to_string_lossy().into_owned().as_str(),  &config, sut_db, 0);
        input_db.post_process();

        println!(
            "{:?}",
            input_db.fuzzer_id_initial_seeds_id_to_children_input_id_map
        );
        let mut ground_truth: HashMap<usize, u32> = HashMap::new();
        ground_truth.insert(0, 2);
        ground_truth.insert(1, 2);
        ground_truth.insert(2, 2);
        ground_truth.insert(3, 2);
        ground_truth.insert(4, 1);
        ground_truth.insert(5, 1);
        ground_truth.insert(6, 1);
        ground_truth.insert(7, 1);
        let byte_changes = input_db.compare_inputs(&0, &1);
        println!("byte changes: {:?}", byte_changes);
        assert_eq!(byte_changes, ground_truth);
    }
}
