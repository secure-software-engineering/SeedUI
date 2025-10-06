use std::{collections::HashMap, fmt, fs::read_to_string, path::PathBuf, str::FromStr};

use config::TargetConfig;
use custom_types::*;

#[derive(Clone)]
pub struct SUT {
    file_id_to_file_meta_map: HashMap<FileId, FileMeta>,
    filename_to_file_id_map: HashMap<String, FileId>,
    file_id_line_num_line_meta_map: HashMap<LineId, LineMeta>,
    allowed_folders: Vec<String>,
    allowed_extensions: Vec<String>,
}

impl fmt::Debug for SUT {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret = String::from("");
        for (file_id, file_meta) in self.file_id_to_file_meta_map.iter() {
            ret.push_str(&format!("{:?} -> {:?}\n", file_id.as_usize(), file_meta));
            for line_id in file_meta.lines.iter() {
                ret.push_str(&format!(
                    "\t{:?} -> {:?}\n",
                    line_id,
                    self.file_id_line_num_line_meta_map.get(line_id)
                ));
            }
        }
        write!(f, "{}", ret)
    }
}

impl SUT {
    pub fn new() -> Self {
        SUT {
            filename_to_file_id_map: HashMap::new(),
            file_id_to_file_meta_map: HashMap::new(),
            file_id_line_num_line_meta_map: HashMap::new(),
            allowed_folders: Vec::new(),
            allowed_extensions: Vec::new(),
        }
    }

    pub fn parse_config(&mut self, config: &TargetConfig) {
        let mut paths_to_check: Vec<&str> = vec![&config.target_source_code_path];
        for p in &config.target_include_filter {
            paths_to_check.push(p.as_str());
        }

        self.allowed_folders = paths_to_check
            .iter()
            .map(|i| String::from_str(i).unwrap())
            .collect();
        self.allowed_extensions = config
            .allowed_extensions
            .iter()
            .map(|i| String::from_str(i).unwrap())
            .collect();
    }

    pub fn parse_file(&mut self, filename: &str) -> Option<FileId> {
        if self.filename_to_file_id_map.contains_key(filename) {
            return self.filename_to_file_id_map.get(filename).copied();
        }

        let filepath = PathBuf::from_str(filename).unwrap();
        if filepath.is_dir() {
            return None;
        }

        if !self.allowed_extensions.is_empty() {
            let extension = filepath.extension().unwrap_or_default().to_str().unwrap();
            if !self.allowed_extensions.contains(&extension.to_string()) {
                return None;
            }
        }

        let mut allowed = false;
        for folder in self.allowed_folders.iter() {
            if filepath.starts_with(folder) {
                allowed = true;
            }
        }

        if allowed {
            let curr_map_len = self.filename_to_file_id_map.len() + 1;
            let file_id = self
                .filename_to_file_id_map
                .entry(filepath.display().to_string())
                .or_insert(FileId::new(curr_map_len));
            let file_meta = self
                .file_id_to_file_meta_map
                .entry(*file_id)
                .or_insert(FileMeta::new(&filepath.display().to_string()));

            // The current file is inserted into the map and so we need to create line ids for this file
            match read_to_string(&filepath) {
                Ok(f) => {
                    for (cur_line, cur_line_str) in f.lines().enumerate() {
                        let first_word = cur_line_str.split_whitespace().next().unwrap_or_default();
                        let line_num: u32 = cur_line as u32 + 1;
                        self.file_id_line_num_line_meta_map.insert(
                            LineId::new(*file_id, line_num),
                            LineMeta {
                                file_id: *file_id,
                                line_num: line_num,
                                hit_count: 0,
                                fuzzer_configuration_ids: Vec::new(),
                                is_comment: cur_line_str.is_empty()
                                    || first_word.starts_with("/*")
                                    || first_word.starts_with("*/")
                                    || first_word.starts_with("//")
                                    || (first_word.len() == 1 && first_word.starts_with("*")),
                            },
                        );
                        file_meta.lines.insert(LineId::new(*file_id, line_num));
                    }
                }
                Err(_) => {
                    println!("Unable to read file: {:?}", filepath);
                }
            };
        }

        return self.filename_to_file_id_map.get(filename).copied();
    }

    pub fn read_file_content(&self, filepath: &str) -> String {
        match read_to_string(&filepath) {
            Ok(f) => f,
            Err(_) => "File content unavailable".to_string(),
        }
    }

    pub fn get_file_id(&self, filename: &str) -> Option<FileId> {
        self.filename_to_file_id_map.get(filename).copied()
    }

    pub fn get_all_files(&self) -> Vec<String> {
        self.filename_to_file_id_map
            .keys()
            .map(|x| x.clone())
            .collect()
    }

    pub fn get_all_lines(&self, file_id: FileId) -> Vec<&LineMeta> {
        let mut ret: Vec<&LineMeta> = Vec::new();
        for (line_id, &ref line_meta) in self.file_id_line_num_line_meta_map.iter() {
            if line_id.file() == file_id {
                ret.push(&line_meta);
            }
        }
        ret
    }

    pub fn get_line_meta(&self, line_id: LineId) -> Option<&LineMeta> {
        self.file_id_line_num_line_meta_map.get(&line_id)
    }

    pub fn get_mut_line_meta(&mut self, line_id: LineId) -> Option<&mut LineMeta> {
        self.file_id_line_num_line_meta_map.get_mut(&line_id)
    }

    pub fn set_line_covered(&mut self, line_id: LineId, fuzzer_configuration_id: u32) -> u32 {
        match self.file_id_line_num_line_meta_map.get_mut(&line_id) {
            Some(line_meta) => {
                line_meta.hit_count += 1;
                if !line_meta
                    .fuzzer_configuration_ids
                    .contains(&fuzzer_configuration_id)
                {
                    line_meta
                        .fuzzer_configuration_ids
                        .push(fuzzer_configuration_id);
                    return 1;
                }
                return line_meta.hit_count;
            }
            None => return 0,
        };
    }

    pub fn get_file_meta(&self, file_id: &FileId) -> Option<&FileMeta> {
        self.file_id_to_file_meta_map.get(file_id)
    }

    pub fn increment_unique_line_hits(&mut self, file_id: &FileId, fuzzer_configuration_id: u32) {
        *self
            .file_id_to_file_meta_map
            .get_mut(file_id)
            .unwrap()
            .unique_line_hits
            .entry(fuzzer_configuration_id)
            .or_insert(0) += 1;
    }

    pub fn get_all_file_meta(&self) -> &HashMap<FileId, FileMeta> {
        &self.file_id_to_file_meta_map
    }

    pub fn get_file_id_line_num_line_meta_map(&self) -> &HashMap<LineId, LineMeta> {
        &self.file_id_line_num_line_meta_map
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;

    #[test]
    fn test1() {
        let config = TargetConfig {
            target_path: "".to_string(),
            target_source_code_path: String::from(
                fs::canonicalize(PathBuf::from("./test_data/test1"))
                    .unwrap()
                    .to_str()
                    .unwrap(),
            ),
            target_include_filter: vec![],
            allowed_extensions: vec![],
        };
        print!("{:?}\n", config);
        let mut sut_db = SUT::new();
        sut_db.parse_config(&config);
        sut_db.parse_file(&format!("{}/{}", config.target_source_code_path, "main.c"));
        assert_eq!(sut_db.get_all_files().len(), 1);
        sut_db.parse_file(&format!("{}/{}", config.target_source_code_path, "foo.c"));
        assert_eq!(sut_db.get_all_files().len(), 2);
        sut_db.parse_file(&format!("{}/{}", config.target_source_code_path, "foo.h"));
        assert_eq!(sut_db.get_all_files().len(), 3);
        let mut line_gt: HashMap<&str, usize> = HashMap::new();
        line_gt.insert("main.c", 17);
        line_gt.insert("foo.c", 20);
        line_gt.insert("foo.h", 4);

        let mut actual: HashMap<&str, usize> = HashMap::new();
        for file in &sut_db.get_all_files() {
            if file.ends_with("main.c") {
                actual.insert(
                    "main.c",
                    sut_db
                        .get_all_lines(sut_db.get_file_id(file).unwrap())
                        .len(),
                );
            } else if file.ends_with("foo.c") {
                actual.insert(
                    "foo.c",
                    sut_db
                        .get_all_lines(sut_db.get_file_id(file).unwrap())
                        .len(),
                );
            } else if file.ends_with("foo.h") {
                actual.insert(
                    "foo.h",
                    sut_db
                        .get_all_lines(sut_db.get_file_id(file).unwrap())
                        .len(),
                );
            }
        }
        assert_eq!(actual, line_gt);
        assert_eq!(sut_db.get_all_files().len(), 3);
    }

    #[test]
    fn test2() {
        let config = TargetConfig {
            target_path: "".to_string(),
            target_source_code_path: String::from(
                fs::canonicalize(PathBuf::from("./test_data/test2"))
                    .unwrap()
                    .to_str()
                    .unwrap(),
            ),
            target_include_filter: vec![],
            allowed_extensions: vec!["c".to_string(), "h".to_string()],
        };
        print!("{:?}\n", config);
        let mut sut_db = SUT::new();
        sut_db.parse_config(&config);
        sut_db.parse_file(&format!("{}/{}", config.target_source_code_path, "main.c"));
        assert_eq!(sut_db.get_all_files().len(), 1);
        sut_db.parse_file(&format!(
            "{}/{}",
            config.target_source_code_path, "inner/foo.c"
        ));
        assert_eq!(sut_db.get_all_files().len(), 2);
        sut_db.parse_file(&format!(
            "{}/{}",
            config.target_source_code_path, "inner/foo.h"
        ));
        assert_eq!(sut_db.get_all_files().len(), 3);
        let mut line_gt: HashMap<&str, usize> = HashMap::new();
        line_gt.insert("main.c", 17);
        line_gt.insert("inner/foo.c", 20);
        line_gt.insert("inner/foo.h", 4);

        let mut actual: HashMap<&str, usize> = HashMap::new();
        for file in &sut_db.get_all_files() {
            if file.ends_with("main.c") {
                actual.insert(
                    "main.c",
                    sut_db
                        .get_all_lines(sut_db.get_file_id(file).unwrap())
                        .len(),
                );
            } else if file.ends_with("inner/foo.c") {
                actual.insert(
                    "inner/foo.c",
                    sut_db
                        .get_all_lines(sut_db.get_file_id(file).unwrap())
                        .len(),
                );
            } else if file.ends_with("foo.h") {
                actual.insert(
                    "inner/foo.h",
                    sut_db
                        .get_all_lines(sut_db.get_file_id(file).unwrap())
                        .len(),
                );
            }
        }
        assert_eq!(actual, line_gt);
    }
}
