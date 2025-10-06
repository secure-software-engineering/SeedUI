//! Modified version of the one from libAFL (https://github.com/AFLplusplus/LibAFL/blob/main/crates/libafl_targets/src/drcov.rs)
//!     - removed writer, adapted the checks to match the drcov version 2 of current qemuafl outputs and added more utility methods

#![allow(warnings)]

use core::{fmt::Debug, num::ParseIntError, ptr};
use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

use rangemap::RangeMap;

/// A basic block struct
/// This can be used to keep track of new addresses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrCovBasicBlock {
    /// Start of this basic block
    pub start: u64,
    /// End of this basic block
    pub end: u64,
}

impl DrCovBasicBlock {
    /// Create a new [`DrCovBasicBlock`] with the given `start` and `end` addresses.
    #[must_use]
    pub fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }

    /// Create a new [`DrCovBasicBlock`] with a given `start` address and a block size.
    #[must_use]
    pub fn with_size(start: u64, size: u16) -> Self {
        Self::new(start, start + u64::try_from(size).unwrap())
    }
}

/// A (Raw) Basic Block List Entry.
/// This is only relevant in combination with a [`DrCovReader`] or a [`DrCovWriter`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct DrCovBasicBlockEntry {
    /// Start of this basic block
    pub start: u32,
    /// Size of this basic block
    size: u16,
    /// The id of the `DrCov` module this block is in
    mod_id: u16,
}

impl From<&[u8; 8]> for DrCovBasicBlockEntry {
    fn from(value: &[u8; 8]) -> Self {
        // # Safety
        // The value is a valid u8 pointer.
        // There's a chance that the value is not aligned to 32 bit, so we use `read_unaligned`.
        assert_eq!(
            size_of::<DrCovBasicBlockEntry>(),
            size_of::<[u8; 8]>(),
            "`DrCovBasicBlockEntry` size changed!"
        );
        unsafe { ptr::read_unaligned(ptr::from_ref(value) as *const DrCovBasicBlockEntry) }
    }
}

impl From<DrCovBasicBlockEntry> for [u8; 8] {
    fn from(value: DrCovBasicBlockEntry) -> Self {
        // # Safety
        // The value is a c struct.
        // Casting its pointer to bytes should be safe.
        // The resulting pointer needs to be less aligned.
        assert_eq!(
            size_of::<DrCovBasicBlockEntry>(),
            size_of::<[u8; 8]>(),
            "`DrCovBasicBlockEntry` size changed!"
        );
        unsafe { std::slice::from_raw_parts(ptr::from_ref(&value).cast::<u8>(), 8) }
            .try_into()
            .unwrap()
    }
}

impl From<&DrCovBasicBlockEntry> for &[u8] {
    fn from(value: &DrCovBasicBlockEntry) -> Self {
        // # Safety
        // The value is a c struct.
        // Casting its pointer to bytes should be safe.
        unsafe {
            std::slice::from_raw_parts(
                ptr::from_ref(value).cast::<u8>(),
                size_of::<DrCovBasicBlockEntry>(),
            )
        }
    }
}

/// An entry in the `DrCov` module list.
#[derive(Debug, Clone)]
pub struct DrCovModuleEntry {
    /// The index of this module
    pub id: u16,
    /// Base of this module
    pub base: u64,
    /// End address of this module
    pub end: u64,
    /// Entry (can be zero)
    pub entry: usize,
    // /// Checksum (can be zero)
    // pub checksum: usize,
    // /// Timestamp (can be zero)
    // pub timestamp: usize,
    /// The path of this module
    pub path: PathBuf,
}

/// Read `DrCov` (v2) files created with [`DrCovWriter`] or other tools
pub struct DrCovReader {
    /// The modules in this `DrCov` file
    pub module_entries: Vec<DrCovModuleEntry>,
    /// The list of basic blocks as [`DrCovBasicBlockEntry`].
    /// To get the blocks as [`DrCovBasicBlock`], call [`Self::basic_blocks`] instead.
    pub basic_block_entries: Vec<DrCovBasicBlockEntry>,
}

impl Debug for DrCovReader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DrCovReader")
            .field("modules", &self.module_entries)
            .field("basic_blocks", &self.basic_block_entries.len())
            .finish()
    }
}

fn parse_hex_to_usize(str: &str) -> Result<usize, ParseIntError> {
    // Cut off the first 0x
    usize::from_str_radix(&str[2..], 16)
}

fn parse_hex_to_u64(str: &str) -> Result<u64, ParseIntError> {
    // Cut off the first 0x
    u64::from_str_radix(&str[2..], 16)
}

fn parse_path(s: &str) -> PathBuf {
    let s = s.trim();

    // If first and last character is a quote, let's remove them
    let s = if s.starts_with('\"') && s.ends_with('\"') {
        &s[1..s.len() - 1]
    } else {
        s
    };

    PathBuf::from(s)
}

impl DrCovReader {
    /// Parse a `drcov` file to memory.
    pub fn read<P: AsRef<Path> + ?Sized>(file: &P) -> Result<Self, String> {
        let f = File::open(file).unwrap();
        let mut reader = BufReader::new(f);

        let mut header = String::new();
        reader.read_line(&mut header).unwrap();

        let drcov_version = "DRCOV VERSION: 2";
        if header.to_uppercase().trim() != drcov_version {
            return Err(format!(
                "No valid header. Expected {drcov_version} but got {header}"
            ));
        }

        header.clear();
        reader.read_line(&mut header).unwrap();

        let drcov_flavor = "DRCOV FLAVOR:";
        if header.to_uppercase().starts_with(drcov_flavor) {
            // Ignore flavor line if it's not present.
            log::info!("Got drcov flavor {drcov_flavor}");

            header.clear();
            reader.read_line(&mut header).unwrap();
        }

        let Some(Ok(module_count)) = header
            .split("Module Table: version 2, count ")
            .nth(1)
            .map(|x| x.trim().parse::<usize>())
        else {
            return Err(format!("Expected module table but got: {header}"));
        };

        header.clear();
        reader.read_line(&mut header).unwrap();

        if !header.starts_with("Columns: id, base, end, entry, path") {
            return Err(format!(
                "Module table has unknown or illegal columns: {header}"
            ));
        }

        let mut modules = Vec::with_capacity(module_count);

        for _ in 0..module_count {
            header.clear();
            reader.read_line(&mut header).unwrap();

            let err = |x| format!("Unexpected module entry while parsing {x} in header: {header}");

            let mut split = header.split(", ");

            let Some(Ok(id)) = split.next().map(str::parse) else {
                return Err(err("id"));
            };

            let Some(Ok(base)) = split.next().map(parse_hex_to_u64) else {
                return Err(err("base"));
            };

            let Some(Ok(end)) = split.next().map(parse_hex_to_u64) else {
                return Err(err("end"));
            };

            let Some(Ok(entry)) = split.next().map(parse_hex_to_usize) else {
                return Err(err("entry"));
            };

            let Some(path) = split.next().map(parse_path) else {
                return Err(err("path"));
            };

            modules.push(DrCovModuleEntry {
                id,
                base,
                end,
                entry,
                path,
            });
        }

        header.clear();
        reader.read_line(&mut header).unwrap();

        //"BB Table: {} bbs\n"
        if !header.starts_with("BB Table: ") {
            return Err(format!("Error reading BB Table header. Got: {header}"));
        }
        let mut bb = header.split(' ');
        let Some(Ok(bb_count)) = bb.nth(2).map(str::parse) else {
            return Err(format!(
                "Error parsing BB Table header count. Got: {header}"
            ));
        };

        let mut basic_blocks = Vec::with_capacity(bb_count);

        for _ in 0..bb_count {
            let mut bb_entry = [0_u8; 8];
            reader.read_exact(&mut bb_entry).unwrap();
            basic_blocks.push((&bb_entry).into());
        }

        Ok(DrCovReader {
            module_entries: modules,
            basic_block_entries: basic_blocks,
        })
    }

    /// Get a list of traversed [`DrCovBasicBlock`] nodes
    #[must_use]
    pub fn basic_blocks(&self) -> Vec<DrCovBasicBlock> {
        let mut ret = Vec::with_capacity(self.basic_block_entries.len());

        for basic_block in &self.basic_block_entries {
            let bb_id = basic_block.mod_id;
            if let Some(module) = self.module_by_id(bb_id) {
                let start = module.base + u64::from(basic_block.start);
                let end = start + u64::from(basic_block.size);
                ret.push(DrCovBasicBlock::new(start, end));
            } else {
                log::error!("Skipping basic block outside of any modules: {basic_block:?}");
            }
        }
        ret
    }

    pub fn basic_blocks_for_module_id(&self, id: u16) -> Vec<DrCovBasicBlock> {
        let mut ret = Vec::with_capacity(self.basic_block_entries.len());
        if let Some(module) = self.module_by_id(id) {
            for basic_block in &self.basic_block_entries {
                if basic_block.mod_id == id {
                    let start = module.base + u64::from(basic_block.start);
                    // let end = start + u64::from(basic_block.size);
                    ret.push(DrCovBasicBlock::with_size(start, basic_block.size));
                }
            }
        } else {
            log::error!("Skipping basic block outside of any modules");
        }

        ret
    }

    /// Get the module (range) map. This can be used to create a new [`DrCovWriter`].
    #[must_use]
    pub fn module_map(&self) -> RangeMap<u64, (u16, String)> {
        let mut ret = RangeMap::new();
        for module in &self.module_entries {
            ret.insert(
                module.base..module.end,
                (
                    module.id,
                    module.path.clone().into_os_string().into_string().unwrap(),
                ),
            );
        }
        ret
    }

    /// Gets a list of all basic blocks, as absolute addresses, for u64 targets.
    /// Useful for example for [`JmpScare`](https://github.com/fgsect/JMPscare) and other analyses.
    #[must_use]
    pub fn basic_block_addresses_u64(&self) -> Vec<u64> {
        self.basic_blocks().iter().map(|x| x.start).collect()
    }

    /// Gets a list of all basic blocks, as absolute addresses, for u32 targets.
    /// Will return an [`Error`] if addresses are larger than 32 bit.
    pub fn basic_block_addresses_u32(&self) -> Result<Vec<u32>, String> {
        let blocks = self.basic_blocks();
        let mut ret = Vec::with_capacity(blocks.len());
        for block in self.basic_blocks() {
            ret.push(u32::try_from(block.start).unwrap());
        }
        Ok(ret)
    }

    /// Returns the module for a given `id`, or [`None`].
    #[must_use]
    pub fn module_by_id(&self, id: u16) -> Option<&DrCovModuleEntry> {
        self.module_entries.iter().find(|module| module.id == id)
    }

    #[must_use]
    pub fn get_module_entry(&self, module_filename: &str) -> Option<&DrCovModuleEntry> {
        self.module_entries
            .iter()
            .find(|module| module.path.ends_with(module_filename))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_write_read_drcov() {
        let test_file = "test_data/traces/drcov_input_a.trace";
        let reader = DrCovReader::read(&test_file).unwrap();
        let mod_id = reader.get_module_entry("test").unwrap().id;
        assert_eq!(mod_id, 0);
        let module_bbs = reader.basic_blocks_for_module_id(mod_id);
        for mod_bb in &module_bbs {
            println!("\tBlock {:#x}, {:#x}", mod_bb.start, mod_bb.end);
        }
        assert_eq!(module_bbs.len(), 39);
    }
}
