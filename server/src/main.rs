use std::{
    env,
    fs::{self, read_dir},
    path::Path,
    process::exit,
};

use actix_cors::Cors;
use actix_web::{
    web::{self},
    App, HttpServer,
};
use glob::glob;

mod app_state;
mod responders;

use inputs_database::InputsDatabase;
use sut_database::SUT;
use config::UserConfig;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: server path/to/config.ron");
        exit(1);
    }

    let config_arg = Path::new(&args[1]);
    if !config_arg.exists() || config_arg.is_dir() {
        println!(
            "Argument {:?} is not a configuration file.\nUsage: server path/to/config.ron",
            &args[1]
        );
        exit(1);
    }

    let addr = "127.0.0.1:8080";
    let config = UserConfig::parse(config_arg.to_str().unwrap());
    let mut input_db: InputsDatabase = InputsDatabase::new();
    let mut sut_db = SUT::new();

    sut_db.parse_config(&config.target_info);
    for fuzzer_info in &config.fuzzer_infos {
        println!("Fuzzer: {:?}", fuzzer_info.fuzzer_configuration);
        input_db.add_fuzzer_configuration(fuzzer_info);

        let mut total_initial_seeds = 0;
        for entry in glob(&format!("{}/id:*", fuzzer_info.traces_directory_path))
            .expect("Failed to read glob pattern")
        {
            sut_db = input_db.add_input(
                entry.unwrap().to_str().unwrap(),
                &config.target_info,
                sut_db,
                fuzzer_info.fuzzer_configuration_id,
            );
            total_initial_seeds += 1;
        }
        println!("\ttotal initial seeds: {:?}", total_initial_seeds);

        let mut total_trace_files = 0;
        for entry in read_dir(&fuzzer_info.traces_directory_path)
            .expect("Failed to read trace files in the directory")
        {
            let path = entry.unwrap().path();
            if path.is_file() {
                if path.file_name().unwrap().to_str().unwrap().contains("orig") {
                    continue;
                }

                if let Ok(absolute_path) = fs::canonicalize(&path) {
                    sut_db = input_db.add_input(
                        absolute_path.to_str().unwrap(),
                        &config.target_info,
                        sut_db,
                        fuzzer_info.fuzzer_configuration_id,
                    );
                    total_trace_files += 1;
                }
            }
        }
        println!("\ttotal trace files: {:?}", total_trace_files);
    }

    input_db.post_process();

    println!("https://{}", addr);

    HttpServer::new(move || {
        // TODO: I know we should never do this but first lets get this app up and running!
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allowed_methods(vec!["GET", "POST"]);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(app_state::AppState::new(
                input_db.clone(),
                sut_db.clone(),
            )))
            .route("/fuzzer_info", web::get().to(responders::get_fuzzer_info))
            .route(
                "/line_coverage",
                web::post().to(responders::get_line_coverage_over_time),
            )
            .route("/sut", web::get().to(responders::get_sut))
            .route(
                "/sut_file_info",
                web::get().to(responders::get_sut_file_id_name_map),
            )
            .route(
                "/input_clusters",
                web::post().to(responders::get_all_input_clusters),
            )
            .route(
                "/compare_inputs",
                web::post().to(responders::get_inputs_comparison),
            )
            .route(
                "/initial_seeds_line_coverage_for_file",
                web::post().to(responders::get_initial_seeds_line_coverage_for_file),
            )
            .route(
                "/line_coverage_for_file",
                web::post().to(responders::get_line_coverage_for_file),
            )
            .route(
                "/initial_seed_timeline",
                web::post().to(responders::get_initial_seed_timeline),
            )
    })
    .bind(addr)?
    .workers(1)
    .run()
    .await
}
