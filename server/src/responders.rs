use std::{collections::HashMap, fmt, path::PathBuf, sync::OnceLock};

use actix_web::{web, Responder};
use chrono::{Local, NaiveDateTime, TimeZone};
use serde::{Serialize, Deserialize};

use crate::app_state::AppState;
use custom_types::*;

static NORMALIZED_STARTTIME: OnceLock<i64> = OnceLock::new();
fn get_normalized_starttime_millis() -> i64 {
    // NOTE: "2024-01-01 00:00:00.000 UTC+1" => 1704067200000
    // NOTE: "2024-01-01 00:00:00.000 UTC" => 1704063600000
    *NORMALIZED_STARTTIME.get_or_init(|| {
        let local_dt = Local
            .from_local_datetime(
                &NaiveDateTime::parse_from_str("2024-01-01 00:00:00.000", "%Y-%m-%d %H:%M:%S%.3f")
                    .unwrap(),
            )
            .unwrap();
        local_dt.timestamp_millis()
    })
}

#[derive(Clone, Serialize)]
pub struct UIFuzzerInfo {
    pub fuzzer_configuration_id: u32,
    pub fuzzer_configuration_name: String,
    pub total_initial_seeds: usize,
    pub total_inputs: usize,
    pub initial_seeds_children_input_id_map: HashMap<u32, Vec<(usize, u32)>>,
    pub run_time: f32,
}

impl fmt::Debug for UIFuzzerInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UIFuzzerInfo")
            .field(
                "fuzzer_configuration_id",
                &format_args!("{:?}", self.fuzzer_configuration_id),
            )
            .field(
                "fuzzer_configuration_name",
                &format_args!("{:?}", self.fuzzer_configuration_name),
            )
            .field(
                "total_initial_seeds",
                &format_args!("{:?}", self.total_initial_seeds),
            )
            .field("total_inputs", &format_args!("{:?}", self.total_inputs))
            .field(
                "initial_seeds_children_map",
                &format_args!("{:?}", self.initial_seeds_children_input_id_map),
            )
            .finish()
    }
}

pub async fn get_fuzzer_info(data: web::Data<AppState>) -> impl Responder {
    println!("GET /fuzzer_info");
    let mut response: Vec<UIFuzzerInfo> = Vec::new();
    let fuzzer_infos = data.get_inputs_db().get_all_fuzzer_configurations();
    for (fuzzer_conf_id, fuzzer_config) in fuzzer_infos.iter() {
        let fuzzer_min_max_times = data
            .get_inputs_db()
            .get_run_times_for_fuzzer_id(fuzzer_conf_id);
        let min_time: i64 = fuzzer_min_max_times.0;
        let max_time: i64 = fuzzer_min_max_times.1;

        let mut current_response = UIFuzzerInfo {
            fuzzer_configuration_id: *fuzzer_conf_id,
            fuzzer_configuration_name: fuzzer_config.fuzzer_configuration.clone(),
            total_initial_seeds: data
                .get_inputs_db()
                .get_all_initial_seeds_for_fuzzer_id(fuzzer_conf_id)
                .len(),
            total_inputs: data
                .get_inputs_db()
                .get_all_inputs_for_fuzzer_id(fuzzer_conf_id)
                .len(),
            initial_seeds_children_input_id_map: HashMap::new(),
            run_time: (max_time - min_time) as f32 / (1000 * 60 * 60) as f32,
        };

        for initial_seed_id in data
            .get_inputs_db()
            .get_all_initial_seeds_meta_info(fuzzer_conf_id)
            .iter()
        {
            if data
                .get_inputs_db()
                .has_children_for(fuzzer_conf_id, &initial_seed_id.1.fuzz_input_id)
            {
                let children = data.get_inputs_db().get_all_children_input_ids_for(
                    fuzzer_conf_id,
                    &vec![initial_seed_id.1.fuzz_input_id],
                );
                for child in children.iter() {
                    let child_meta = data.get_inputs_db().get_inputs_meta_info_for(child);
                    current_response
                        .initial_seeds_children_input_id_map
                        .entry(initial_seed_id.1.fuzz_input_id)
                        .or_default()
                        .push((child.as_usize(), child_meta.fuzz_input_id));
                }
            }
        }

        response.push(current_response);
    }
    serde_json::to_string(&response)
}

#[derive(Clone, Serialize)]
pub struct UIOverviewInfo {
    pub input_id: u32,
    pub executed_on: i64,
    pub fuzzer_coverage: u32,
}

impl fmt::Debug for UIOverviewInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UIOverviewInfo")
            .field("input_id", &format_args!("{:?}", self.input_id))
            .field("executed_on", &format_args!("{:?}", self.executed_on))
            .field("fuzzer_coverage", &format_args!("{}", self.fuzzer_coverage))
            .finish()
    }
}

pub async fn get_line_coverage_over_time(
    interesting_lines: web::Json<HashMap<usize, Vec<u32>>>,
    data: web::Data<AppState>,
) -> impl Responder {
    println!("POST /line_coverage {:?}", interesting_lines);
    let all_inputs = data.get_inputs_db().get_all_inputs_meta_info();
    let mut ret: HashMap<u32, HashMap<i64, UIOverviewInfo>> = HashMap::new();

    for (fuzzer_configuration_id, _) in data.get_inputs_db().get_all_fuzzer_configurations().iter()
    {
        ret.insert(*fuzzer_configuration_id, HashMap::new());
        let fuzzer_min_max_times = data
            .get_inputs_db()
            .get_run_times_for_fuzzer_id(fuzzer_configuration_id);
        let min_start_time: i64 = fuzzer_min_max_times.0;
        let time_delta_to_substract = min_start_time - get_normalized_starttime_millis();

        for (&_input_id, input_metadata) in all_inputs.iter() {
            if input_metadata.fuzzer_configuration != *fuzzer_configuration_id {
                continue;
            }

            let ex_time = input_metadata.executed_on - time_delta_to_substract;
            ret.get_mut(fuzzer_configuration_id).unwrap().insert(
                ex_time,
                UIOverviewInfo {
                    input_id: input_metadata.fuzz_input_id,
                    executed_on: ex_time,
                    fuzzer_coverage: input_metadata.fuzzer_coverage,
                },
            );
        }
    }

    serde_json::to_string(&ret)
}

#[derive(Serialize)]
struct UIFileInfo {
    name: String,
    id: FileId,
    lines: Vec<LineMeta>,
    content: String,
    unique_lines_covered: HashMap<u32, u32>,
}

impl UIFileInfo {
    pub fn new(file_meta: &FileMeta) -> UIFileInfo {
        UIFileInfo {
            name: String::from(
                PathBuf::from(&file_meta.name)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap(),
            ),
            id: file_meta.lines.iter().nth(0).unwrap().file(),
            lines: Vec::new(),
            content: "".to_string(),
            unique_lines_covered: HashMap::new(),
        }
    }
}

pub async fn get_sut(data: web::Data<AppState>) -> impl Responder {
    println!("GET /sut");

    let mut response: Vec<UIFileInfo> = Vec::new();
    let fuzzer_configurations = data.get_inputs_db().get_all_fuzzer_configurations();

    for file_meta in data.get_sut_db().get_all_file_meta().values() {
        let mut current_ui_file = UIFileInfo::new(file_meta);
        for &line_id in file_meta.lines.iter() {
            current_ui_file
                .lines
                .push(data.get_sut_db().get_line_meta(line_id).unwrap().clone());
        }

        // sorting is really important for the UI - otherwise the file explorer will go bonkers!
        current_ui_file.lines.sort_by_key(|l| l.line_num);

        current_ui_file.content = data.get_sut_db().read_file_content(&file_meta.name);
        // removing the final '\n' which was added extra in the above iteration
        current_ui_file.content.pop();
        current_ui_file.unique_lines_covered = file_meta.unique_line_hits.clone();
        for (fuzzer_configuration_id, _) in fuzzer_configurations.iter() {
            if !current_ui_file
                .unique_lines_covered
                .contains_key(fuzzer_configuration_id)
            {
                current_ui_file
                    .unique_lines_covered
                    .insert(*fuzzer_configuration_id, 0);
            }
        }

        response.push(current_ui_file);
    }

    // sorting is really important for the UI - otherwise the file explorer will go bonkers!
    response.sort_by_key(|i| i.id.as_usize());

    serde_json::to_string(&response)
}

pub async fn get_sut_file_id_name_map(data: web::Data<AppState>) -> impl Responder {
    println!("GET /sut_file_info");

    let mut response: HashMap<usize, String> = HashMap::new();
    for (file_id, file_meta) in data.get_sut_db().get_all_file_meta() {
        response.insert(
            file_id.as_usize(),
            String::from(
                PathBuf::from(&file_meta.name)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap(),
            ),
        );
    }

    serde_json::to_string(&response)
}

#[derive(Clone, Serialize)]
pub struct UILineAndBitmapCoverage {
    pub fuzzer_coverage: u32,
}

impl fmt::Debug for UILineAndBitmapCoverage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UILineCoverageInfo")
            .field(
                "fuzzer_coverage",
                &format_args!("{:?}", self.fuzzer_coverage),
            )
            .finish()
    }
}

#[derive(Debug, Deserialize)]
pub struct UIInputClustersRequest {
    pub cluster_threshold_seconds: i64,
}

#[derive(Debug, Serialize)]
struct UIInputClusters {
    initial_seeds: HashMap<u32, f32>,
    inputs: HashMap<InputId, UILineAndBitmapCoverage>,
    total_fuzzer_coverage: u32,
    total_inputs: usize,
}

impl UIInputClusters {
    pub fn new() -> Self {
        UIInputClusters {
            initial_seeds: HashMap::new(),
            inputs: HashMap::new(),
            total_fuzzer_coverage: 0,
            total_inputs: 0,
        }
    }
}

pub async fn get_all_input_clusters(
    request: web::Json<UIInputClustersRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    println!("POST /input_clusters {:?}", request);
    let mut response: HashMap<u32, HashMap<i64, UIInputClusters>> = HashMap::new();
    let all_inputs = data.get_inputs_db().get_all_inputs_meta_info();
    let cluster_threshold = request.cluster_threshold_seconds * 60 * 1000; // minutes * seconds * milliseconds
    println!("cluster threshold in milliseconds: {:?}", cluster_threshold);

    for (fuzzer_configuration_id, _) in data.get_inputs_db().get_all_fuzzer_configurations().iter()
    {
        let current_cluster_map = response.entry(*fuzzer_configuration_id).or_default();
        let initial_seeds_meta = data
            .get_inputs_db()
            .get_all_initial_seeds_meta_info(fuzzer_configuration_id);

        let fuzzer_min_max_times = data
            .get_inputs_db()
            .get_run_times_for_fuzzer_id(fuzzer_configuration_id);
        let min_start_time: i64 = fuzzer_min_max_times.0;
        let max_start_time: i64 = fuzzer_min_max_times.1;
        let time_delta_to_substract = min_start_time - get_normalized_starttime_millis();

        let num_clusters = (max_start_time - min_start_time) / cluster_threshold;
        // NOTE: +1 is needed in order to show the clusters exactly on the selected cluster size in UI
        for cluster_i in 1..=num_clusters + 1 {
            let cluster_key = min_start_time + (cluster_i * cluster_threshold);
            let cluster_key_normalized = cluster_key - time_delta_to_substract;
            current_cluster_map
                .entry(cluster_key_normalized)
                .or_insert(UIInputClusters::new());
        }

        for (input_id, input_metadata) in all_inputs.iter() {
            if input_metadata.fuzzer_configuration != *fuzzer_configuration_id {
                continue;
            }

            let mut cluster_index = ((input_metadata.executed_on - time_delta_to_substract)
                - (min_start_time - time_delta_to_substract))
                / cluster_threshold;
            // NOTE: +1 because of the one before
            cluster_index += 1;
            let cluster_key_normalized =
                get_normalized_starttime_millis() + (cluster_index * cluster_threshold);
            let current_cluster = current_cluster_map
                .get_mut(&cluster_key_normalized)
                .unwrap();
            current_cluster.total_fuzzer_coverage += input_metadata.fuzzer_coverage;
            let parents = data
                .get_inputs_db()
                .get_initial_seed_parents_for(input_id, fuzzer_configuration_id);
            for parent in &parents {
                *current_cluster
                    .initial_seeds
                    .entry(initial_seeds_meta.get(parent).unwrap().fuzz_input_id)
                    .or_insert(0.0) += input_metadata.fuzzer_coverage as f32 / parents.len() as f32;
            }

            // add the corresponding input's line coverage and unique line hits
            current_cluster.inputs.insert(
                *input_id,
                UILineAndBitmapCoverage {
                    fuzzer_coverage: input_metadata.fuzzer_coverage,
                },
            );
        }
    }

    println!("POST /input_clusters response sent");
    serde_json::to_string(&response)
}

#[derive(Debug, Deserialize)]
pub struct CompareSeedsRequest {
    pub fuzzer_configuration_id: u32,
    pub initial_seed_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitialSeedComparison {
    pub initial_seed_id: u32,
    pub byte_modification_counts: HashMap<usize, u32>,
}

pub async fn get_inputs_comparison(
    request: web::Json<CompareSeedsRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    println!("POST /compare_inputs {:?}", request);
    let mut ret: InitialSeedComparison = InitialSeedComparison {
        initial_seed_id: request.initial_seed_id,
        byte_modification_counts: HashMap::new(),
    };
    let comparison = data
        .get_inputs_db()
        .compare_inputs(&request.fuzzer_configuration_id, &request.initial_seed_id);

    let mut compressed_comparison: HashMap<usize, u32> = HashMap::new();
    let mut comparison_keys: Vec<usize> = comparison.keys().copied().collect();
    comparison_keys.sort();
    let mut previous_byte_count: &u32 = comparison.get(&0).unwrap();
    for byte in comparison_keys.iter() {
        let current_count = comparison.get(byte).unwrap();
        if current_count != previous_byte_count {
            compressed_comparison.insert(*byte, *current_count);
            previous_byte_count = current_count;
        }
    }

    ret.byte_modification_counts = compressed_comparison;

    serde_json::to_string(&ret)
}

#[derive(Debug, Deserialize)]
pub struct InitialSeedsLineCoverageRequest {
    pub file_id: usize,
}

pub async fn get_initial_seeds_line_coverage_for_file(
    request: web::Json<InitialSeedsLineCoverageRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    println!("POST /initial_seeds_line_coverage_for_file {:?}", request);
    let mut response: HashMap<u32, HashMap<u32, Vec<LineMeta>>> = HashMap::new();
    for (fuzzer_configuration_id, _) in data.get_inputs_db().get_all_fuzzer_configurations().iter()
    {
        response.entry(*fuzzer_configuration_id).or_default();

        for (_input_id, initial_seeds_meta) in data
            .get_inputs_db()
            .get_all_initial_seeds_meta_info(fuzzer_configuration_id)
            .iter()
        {
            let current_initial_seed_coverage = data
                .get_inputs_db()
                .get_initial_seed_line_coverage_for_file_id(
                    fuzzer_configuration_id,
                    &initial_seeds_meta.fuzz_input_id,
                    &FileId::new(request.file_id),
                    data.get_sut_db(),
                );
            response.get_mut(fuzzer_configuration_id).unwrap().insert(
                initial_seeds_meta.fuzz_input_id,
                current_initial_seed_coverage,
            );
        }
    }

    serde_json::to_string(&response)
}

#[derive(Debug, Deserialize)]
pub struct LineCoverageRequest {
    pub fuzzer_configuration_id: u32,
    pub file_id: usize,
    pub initial_seed_id: u32,
    pub child_id: usize,
}

pub async fn get_line_coverage_for_file(
    request: web::Json<LineCoverageRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    println!("POST /line_coverage_for_file {:?}", request);
    let response = data
        .get_inputs_db()
        .get_all_children_line_coverage_for_file_id(
            &request.fuzzer_configuration_id,
            &request.initial_seed_id,
            &FileId::new(request.file_id),
            data.get_sut_db(),
        )
        .get(&InputId::new(request.child_id))
        .unwrap()
        .clone();
    serde_json::to_string(&response)
}

#[derive(Debug, Deserialize)]
pub struct TimelineRequest {
    pub fuzzer_configuration_id: u32,
    pub initial_seed_ids: Vec<u32>,
}

#[derive(Debug, Serialize)]
pub struct TimelineNode {
    pub id: String,
    pub name: String,
    pub x_executed_on: i64,
    pub y_fuzzer_coverage: u32,
    pub meta_data: String,
    pub multiple: bool,
}

#[derive(Debug, Serialize)]
pub struct TimelineEdge {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Serialize)]
pub struct TimelineResponse {
    pub nodes: Vec<TimelineNode>,
    pub edges: Vec<TimelineEdge>,
}

impl TimelineResponse {
    pub fn new() -> Self {
        TimelineResponse {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

pub async fn get_initial_seed_timeline(
    request: web::Json<TimelineRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    println!("POST /initial_seed_timeline {:?}", request);
    let mut response = TimelineResponse::new();
    let mut time_delta_to_substract = 0;

    for initial_seed_id in request.initial_seed_ids.iter() {
        let initial_seed_info = data
            .get_inputs_db()
            .get_all_initial_seeds_meta_info(&request.fuzzer_configuration_id)
            .iter()
            .find(|&(_input_id, input_meta)| input_meta.fuzz_input_id == *initial_seed_id)
            .unwrap();
        time_delta_to_substract =
            initial_seed_info.1.executed_on - get_normalized_starttime_millis();

        response.nodes.push(TimelineNode {
            id: format!("seed-{:?}", initial_seed_info.1.fuzz_input_id),
            name: format!("seed-{:?}", initial_seed_info.1.fuzz_input_id),
            x_executed_on: initial_seed_info.1.executed_on - time_delta_to_substract,
            y_fuzzer_coverage: 0,
            meta_data: format!("initial seed-{:?}", initial_seed_info.1.fuzz_input_id),
            multiple: false,
        });
    }

    for child_input_id in data
        .get_inputs_db()
        .get_all_children_input_ids_for(&request.fuzzer_configuration_id, &request.initial_seed_ids)
        .iter()
    {
        let current_meta = data
            .get_inputs_db()
            .get_inputs_meta_info_for(child_input_id);
        assert!(current_meta.fuzzer_configuration == request.fuzzer_configuration_id);
        let mut current_response_meta_data = String::new();
        let mut curr_node = TimelineNode {
            id: format!("seed-{:?}", child_input_id.as_usize()),
            name: format!("seed-{:?}", current_meta.fuzz_input_id),
            x_executed_on: current_meta.executed_on - time_delta_to_substract,
            y_fuzzer_coverage: current_meta.fuzzer_coverage,
            meta_data: "".to_string(),
            multiple: current_meta.parents.len() > 1,
        };

        for parent_id in current_meta.parents.iter() {
            if data
                .get_inputs_db()
                .has_children_for(&request.fuzzer_configuration_id, parent_id)
            {
                response.edges.push(TimelineEdge {
                    source: format!("seed-{:?}", *parent_id),
                    target: format!("seed-{:?}", child_input_id.as_usize()),
                });
                current_response_meta_data.push_str(&format!("initial seed-{:?}, ", *parent_id));
            } else {
                let curr_parent = data
                    .get_inputs_db()
                    .get_input_id_for(&request.fuzzer_configuration_id, parent_id);
                let curr_parent_meta = data.get_inputs_db().get_inputs_meta_info_for(curr_parent);
                response.edges.push(TimelineEdge {
                    source: format!("seed-{:?}", curr_parent.as_usize()),
                    target: format!("seed-{:?}", child_input_id.as_usize()),
                });
                // NOTE: To display the parent information we cannot use the incremental input_id because they will not match with the id field in the seed filename
                current_response_meta_data
                    .push_str(&format!("seed-{:?}, ", curr_parent_meta.fuzz_input_id));
            }
        }

        current_response_meta_data.pop();
        current_response_meta_data.pop();
        curr_node.meta_data = current_response_meta_data;
        response.nodes.push(curr_node);
    }

    serde_json::to_string(&response)
}
