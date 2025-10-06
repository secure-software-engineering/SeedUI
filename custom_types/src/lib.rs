use std::{
    collections::{HashMap, HashSet},
    fmt, usize,
};

use serde::Serialize;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Serialize)]
pub struct InputId(usize);

impl InputId {
    pub fn new(id: usize) -> Self {
        InputId(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Clone, Serialize)]
pub struct InputMeta {
    pub id: InputId,
    pub fuzz_input_id: u32,
    pub total_mutations_required_to_generate: u32,
    pub execution_time: i64,
    pub fuzzer_coverage: u32,
    pub executed_on: i64,
    pub source_line_coverage: HashSet<LineId>,
    pub parents: Vec<u32>,
    pub is_initial_seed: bool,
    pub fuzzer_configuration: u32,
    pub file_name_stem: String,
}

impl Default for InputMeta {
    fn default() -> Self {
        Self::new()
    }
}

impl InputMeta {
    pub fn new() -> Self {
        InputMeta {
            id: InputId(0),
            fuzz_input_id: 0,
            total_mutations_required_to_generate: 0,
            execution_time: 0,
            fuzzer_coverage: 0,
            executed_on: 0,
            source_line_coverage: HashSet::new(),
            parents: Vec::new(),
            is_initial_seed: false,
            fuzzer_configuration: 0,
            file_name_stem: "".to_string(),
        }
    }
}

impl fmt::Debug for InputMeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InputMeta")
            .field(
                "fuzzer_configuration",
                &format_args!("\n\t{:?}", self.fuzzer_configuration),
            )
            .field("id", &format_args!("{:?}", self.id))
            .field(
                "fuzz_input_id",
                &format_args!("\n\t{:?}", self.fuzz_input_id),
            )
            .field("parents", &format_args!("\n\t{:?}", self.parents))
            .field(
                "is_initial_seed",
                &format_args!("\n\t{:?}", self.is_initial_seed),
            )
            .field(
                "file_name_stem",
                &format_args!("\n\t{:?}", self.file_name_stem),
            )
            .finish()
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Serialize)]
pub struct FileId(usize);

impl FileId {
    pub fn new(id: usize) -> Self {
        FileId(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct FileMeta {
    pub name: String,
    pub lines: HashSet<LineId>,
    pub unique_line_hits: HashMap<u32, u32>,
}

impl FileMeta {
    pub fn new(fname: &str) -> Self {
        FileMeta {
            name: String::from(fname),
            lines: HashSet::new(),
            unique_line_hits: HashMap::new(),
        }
    }
}

impl fmt::Debug for FileMeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FileMeta")
            .field("name", &format_args!("{}", self.name))
            // .field("lines", &format_args!("\n\t{:?}", self.lines))
            .field(
                "unique_line_hits",
                &format_args!("\n\t{:?}", self.unique_line_hits),
            )
            .finish()
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Serialize)]
pub struct LineId(FileId, u32);

impl LineId {
    pub fn new(file_id: FileId, num: u32) -> Self {
        LineId(file_id, num)
    }

    pub fn file(&self) -> FileId {
        self.0
    }

    pub fn num(&self) -> u32 {
        self.1
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Serialize)]
pub struct LineMeta {
    pub file_id: FileId,
    pub line_num: u32,
    pub hit_count: u32,
    pub fuzzer_configuration_ids: Vec<u32>,
    pub is_comment: bool,
}

impl fmt::Debug for LineMeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LineMeta")
            .field("file_id", &format_args!("{:?}", self.file_id))
            .field("line_num", &format_args!("{}", self.line_num))
            .field("hit_count", &format_args!("{}", self.hit_count))
            .field(
                "fuzzer_configuration_ids",
                &format_args!("{:?}", self.fuzzer_configuration_ids),
            )
            .field("is_comment", &format_args!("{:?}", self.is_comment))
            .finish()
    }
}
