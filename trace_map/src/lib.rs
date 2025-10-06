use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use std::fmt;

use config::TargetConfig;
use custom_types::{FileId, LineId};
use sut_database::SUT;

mod trace_loader;
use trace_loader::TraceLoader;

mod drcov;
use drcov::{DrCovReader, DrCovBasicBlock};

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct SrcCovBasicBlock {
    pub start: LineId,
    pub end: LineId,
}

#[derive(Eq, PartialEq, Clone)]
pub struct Trace {
    binary: Vec<DrCovBasicBlock>,
    pub source: Vec<SrcCovBasicBlock>,
    pub unique_lines_set: HashSet<LineId>,
}

impl fmt::Debug for SrcCovBasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Location")
            .field(
                "start",
                &format_args!("{:?}:{:?}", self.start.file(), self.start.num(),),
            )
            .field(
                "end",
                &format_args!("{:?}:{:?}", self.end.file(), self.end.num(),),
            )
            .finish()
    }
}

impl fmt::Debug for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ret = String::from("");
        for i in 0..self.binary.len() {
            ret.push_str(&format!(
                "{:?} \n\t -- {:?}\n",
                self.binary[i], self.source[i]
            ));
        }
        write!(f, "{}", ret)
    }
}
pub struct TraceMap {
    loader: TraceLoader,
}

fn check_ancestors(filepath: &Path, target_config: &TargetConfig) -> bool {
    let target_source_code_path_buf =
        PathBuf::from(&target_config.target_source_code_path.to_string());
    let filepath_parent = &filepath.parent().unwrap();
    if target_source_code_path_buf.starts_with(filepath_parent) {
        return true;
    }

    for include_filter in &target_config.target_include_filter {
        let current_include_path = PathBuf::from(&include_filter.to_string());
        if filepath_parent.starts_with(current_include_path.parent().unwrap()) {
            return true;
        }
    }

    false
}

impl TraceMap {
    pub fn new(binary: &str) -> TraceMap {
        let binary_path = PathBuf::from(binary);
        TraceMap {
            loader: TraceLoader::new(&binary_path.display().to_string()),
        }
    }

    pub fn parse_with_config(
        self,
        trace_file: &str,
        target_config: &TargetConfig,
        sut_db: &mut SUT,
    ) -> Trace {
        let mut current_filtered_trace = Trace {
            binary: Vec::new(),
            source: Vec::new(),
            unique_lines_set: HashSet::new(),
        };

        // When afl-qemu-trace crashes in between, there may be some empty drcov trace files
        let file_size = std::fs::metadata(trace_file)
            .expect("file metadata not found")
            .len();
        if file_size == 0 {
            return current_filtered_trace;
        }

        let reader = DrCovReader::read(&trace_file).unwrap();
        let (base, mod_id) = match reader.get_module_entry(&target_config.target_path) {
            Some(some_entry) => (some_entry.base, some_entry.id),
            None => {
                panic!("Module {} not found!", trace_file);
            }
        };

        let binary_traces = reader.basic_blocks_for_module_id(mod_id);
        let mut source_trace_set: HashSet<SrcCovBasicBlock> = HashSet::new();

        for bb in binary_traces {
            // the substraction with base is necessary as the start and base are the virtual addresses
            let soure_loc_find = self.loader.get_location(bb.start - base);
            let (source_loc, source_found) = match &soure_loc_find {
                Some(e) => match sut_db.parse_file(
                    fs::canonicalize(PathBuf::from(e.file.unwrap()))
                        .unwrap()
                        .to_str()
                        .unwrap(),
                ) {
                    Some(e_f) => {
                        let mut ret = (LineId::new(FileId::new(usize::MAX), 0), false);
                        if e.line.is_some() {
                            ret = (LineId::new(e_f, e.line.unwrap()), true);
                        }
                        ret
                    }
                    None => {
                        (LineId::new(FileId::new(usize::MAX), 0), false)
                    }
                },
                None => {
                    (LineId::new(FileId::new(usize::MAX), 0), false)
                }
            };
            let end_loc_find = self.loader.get_location(bb.end - base);
            let (end_loc, end_found) = match &end_loc_find {
                Some(e) => match sut_db.parse_file(
                    fs::canonicalize(PathBuf::from(e.file.unwrap()))
                        .unwrap()
                        .to_str()
                        .unwrap(),
                ) {
                    Some(e_f) => {
                        let mut ret = (LineId::new(FileId::new(usize::MAX), 0), false);
                        if e.line.is_some() {
                            ret = (LineId::new(e_f, e.line.unwrap()), true);
                        }
                        ret
                    }
                    None => {
                        (LineId::new(FileId::new(usize::MAX), 0), false)
                    }
                },
                None => {
                    (LineId::new(FileId::new(usize::MAX), 0), false)
                }
            };

            if source_found && end_found {
                let source_file_path =
                    PathBuf::from(&soure_loc_find.unwrap().file.unwrap().to_string());
                let end_file_path = PathBuf::from(&end_loc_find.unwrap().file.unwrap().to_string());
                let src_to_insert = SrcCovBasicBlock {
                    start: source_loc,
                    end: end_loc,
                };

                if source_loc != end_loc
                    && check_ancestors(&source_file_path, target_config)
                    && check_ancestors(&end_file_path, target_config)
                    && source_trace_set.insert(src_to_insert.clone())
                {
                    for line in src_to_insert.start.num()..=src_to_insert.end.num() {
                        current_filtered_trace
                            .unique_lines_set
                            .insert(LineId::new(src_to_insert.start.file(), line));
                    }

                    current_filtered_trace.binary.push(bb);
                    current_filtered_trace.source.push(src_to_insert);
                }
            }
        }

        current_filtered_trace
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_trace_map_input_b() {
        let config = TargetConfig {
            target_path: "test_data/sources/test".to_string(),
            target_source_code_path: String::from(
                fs::canonicalize(PathBuf::from("./test_data/sources/"))
                    .unwrap()
                    .to_str()
                    .unwrap(),
            ),
            target_include_filter: vec![],
            allowed_extensions: vec![],
        };
        let trace_map = TraceMap::new(&config.target_path);
        let mut sut_db = SUT::new();
        sut_db.parse_config(&config);
        let parsed_trace = trace_map.parse_with_config(
            "test_data/traces/drcov_input_b.trace",
            &config,
            &mut sut_db,
        );
        assert_eq!(parsed_trace.binary.len(), parsed_trace.source.len());
        println!("{:?}", parsed_trace);
        assert_eq!(parsed_trace.binary.len(), 17);
    }

    #[test]
    fn test_trace_map_input_a() {
        let config = TargetConfig {
            target_path: "test_data/sources/test".to_string(),
            target_source_code_path: String::from(
                fs::canonicalize(PathBuf::from("./test_data/sources"))
                    .unwrap()
                    .to_str()
                    .unwrap(),
            ),
            target_include_filter: vec![],
            allowed_extensions: vec![],
        };
        let trace_map = TraceMap::new(&config.target_path);
        let mut sut_db = SUT::new();
        sut_db.parse_config(&config);
        let parsed_trace = trace_map.parse_with_config(
            "test_data/traces/drcov_input_a.trace",
            &config,
            &mut sut_db,
        );
        assert_eq!(parsed_trace.binary.len(), parsed_trace.source.len());
        println!("{:?}", parsed_trace);
        assert_eq!(parsed_trace.binary.len(), 17);
        assert_eq!(parsed_trace.source.len(), 17);
    }

    #[test]
    fn test_trace_map_input_c() {
        let config = TargetConfig {
            target_path: "test_data/sources/test".to_string(),
            target_source_code_path: String::from(
                fs::canonicalize(PathBuf::from("./test_data/sources"))
                    .unwrap()
                    .to_str()
                    .unwrap(),
            ),
            target_include_filter: vec![],
            allowed_extensions: vec![],
        };
        let trace_map = TraceMap::new(&config.target_path);
        let mut sut_db = SUT::new();
        sut_db.parse_config(&config);
        let parsed_trace = trace_map.parse_with_config(
            "test_data/traces/drcov_input_c.trace",
            &config,
            &mut sut_db,
        );
        assert_eq!(parsed_trace.binary.len(), parsed_trace.source.len());
        println!("{:?}", parsed_trace);
        assert_eq!(parsed_trace.binary.len(), 5);
        assert_eq!(parsed_trace.source.len(), 5);
    }
}
