use addr2line::{Loader, Location};
use std::path::PathBuf;

pub struct TraceLoader {
    target_loader: Loader,
}

impl TraceLoader {
    pub fn new(binary: &str) -> TraceLoader {
        TraceLoader {
            target_loader: Loader::new(PathBuf::from(binary)).unwrap(),
        }
    }

    pub fn get_location(&self, hex: u64) -> Option<Location<'_>> {
        // Caching possible here!
        match self.target_loader.find_location(hex) {
            Ok(e) => e,
            Err(e) => {
                println!("Location {:#x} with error: {:?}", hex, e);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn it_works() {
        let trace_info = TraceLoader::new("test_data/sources/test");
        assert_eq!(
            trace_info
                // virtual address - base
                .get_location(0x4000001986 - 0x4000000000)
                .unwrap()
                .line,
            Some(22)
        );
    }
}
