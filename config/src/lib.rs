use std::{fs, path::PathBuf};
use serde::{Deserialize, Serialize};
use ron::from_str;

#[derive(Debug, Deserialize, Serialize)]
pub struct TargetConfig {
    pub target_path: String,
    pub target_source_code_path: String,
    pub target_include_filter: Vec<String>,
    pub allowed_extensions: Vec<String>,
}

impl Default for TargetConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetConfig {
    pub fn new() -> TargetConfig {
        TargetConfig {
            target_path: "".to_string(),
            target_source_code_path: "".to_string(),
            target_include_filter: Vec::new(),
            allowed_extensions: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FuzzerConfig {
    pub fuzzer_configuration: String,
    pub traces_directory_path: String,
    pub inputs_directory_path: String,
    pub fuzzer_configuration_id: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserConfig {
    pub target_info: TargetConfig,
    pub fuzzer_infos: Vec<FuzzerConfig>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl UserConfig {
    pub fn new() -> UserConfig {
        UserConfig {
            target_info: TargetConfig::new(),
            fuzzer_infos: Vec::new(),
        }
    }
}

fn canonicalize(config_old: UserConfig) -> UserConfig {
    let mut config_new = UserConfig::new();

    config_new.target_info.target_path =
        fs::canonicalize(PathBuf::from(&config_old.target_info.target_path))
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
    config_new.target_info.target_source_code_path = fs::canonicalize(PathBuf::from(
        &config_old.target_info.target_source_code_path,
    ))
    .unwrap()
    .to_str()
    .unwrap()
    .to_string();

    for item in config_old.target_info.target_include_filter.iter() {
        config_new.target_info.target_include_filter.push(
            fs::canonicalize(PathBuf::from(item.to_string()))
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        );
    }

    for item in config_old.target_info.allowed_extensions.iter() {
        config_new
            .target_info
            .allowed_extensions
            .push(item.to_string());
    }

    for fuzz_item in config_old.fuzzer_infos.iter() {
        config_new.fuzzer_infos.push(FuzzerConfig {
            fuzzer_configuration_id: fuzz_item.fuzzer_configuration_id,
            fuzzer_configuration: fuzz_item.fuzzer_configuration.to_string(),
            traces_directory_path: fs::canonicalize(PathBuf::from(
                fuzz_item.traces_directory_path.to_string(),
            ))
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
            inputs_directory_path: fs::canonicalize(PathBuf::from(
                fuzz_item.inputs_directory_path.to_string(),
            ))
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        });
    }

    config_new
}

impl UserConfig {
    pub fn parse(config_path: &str) -> UserConfig {
        let p = fs::canonicalize(PathBuf::from(&config_path))
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let f = fs::read_to_string(&p).expect("Failed opening configuration file");
        canonicalize(from_str(&f).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_config_simple() {
        let config = "test_data/test_config_1.ron";
        let parsed_config = UserConfig::parse(config);
        // println!("{:?}", parsed_config);
        assert_eq!(parsed_config.fuzzer_infos.len(), 2);
        assert!(parsed_config
            .target_info
            .target_path
            .contains("test_config_1.ron"));
        assert!(parsed_config
            .target_info
            .target_source_code_path
            .contains("test_data"));
    }

    #[test]
    #[should_panic(expected = "MissingStructField")]
    fn test_user_config_incomplete() {
        let config = "test_data/test_config_2.ron";
        let _ = UserConfig::parse(config);
    }

    #[test]
    fn test_user_config_optionals() {
        let config = "test_data/test_config_3.ron";
        let parsed_config = UserConfig::parse(config);
        // println!("{:?}", parsed_config);
        assert_eq!(parsed_config.fuzzer_infos.len(), 2);
        assert_eq!(parsed_config.target_info.target_include_filter.len(), 0);
    }
}
