use log::LevelFilter;

pub fn configure_logs(level: LevelFilter) {
    let res = simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .with_module_level("omx_tests", level)
        .without_timestamps()
        .init();

    if res.is_err() {
        println!("Logger already initialized");
    }
}
