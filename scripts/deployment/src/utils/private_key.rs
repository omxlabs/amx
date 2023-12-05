use std::fs;

pub fn key_from_file(path: &str) -> String {
    println!("Reading key from {}", path);
    println!(
        "Current dir: {}",
        std::env::current_dir().unwrap().display()
    );

    let key = fs::read_to_string(path).unwrap();

    key.chars().filter(|c| !c.is_whitespace()).collect()
}
