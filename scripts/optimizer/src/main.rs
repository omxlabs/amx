use omx_optimizer::{
    constants::{ARTIFACTS_DIR, EXECUTION_PATH, TARGET_DIR},
    files_filter::{filter_func, FileData},
};
use std::{env, fs};
use wasm_opt::OptimizationOptions;

fn main() {
    let execution_path = fs::canonicalize(EXECUTION_PATH).expect("canonicalize");
    env::set_current_dir(&execution_path).expect("set_current_dir");

    if fs::metadata(ARTIFACTS_DIR).is_ok() {
        fs::remove_dir_all(ARTIFACTS_DIR).expect("remove artifacts dir");
    }

    fs::create_dir(ARTIFACTS_DIR).expect("create artifacts dir");

    let dir = fs::read_dir(TARGET_DIR).unwrap();
    let mut files = dir
        .into_iter()
        .filter_map(filter_func())
        .collect::<Vec<_>>();

    files.sort_by(|a, b| a.in_file.cmp(&b.in_file).reverse());

    for FileData {
        in_file,
        out_file,
        path,
    } in files
    {
        print!("Optimizing {in_file} -> {out_file}");

        OptimizationOptions::new_optimize_for_size_aggressively()
            .run(&in_file, &out_file)
            .unwrap();

        let old_size = fs::metadata(path.clone()).unwrap().len();
        let new_size = fs::metadata(&out_file).unwrap().len();

        println!("\rOptimized {in_file} ({old_size} bytes) -> {out_file} ({new_size} bytes)");
    }
}
