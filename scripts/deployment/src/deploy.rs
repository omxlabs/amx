use std::{fs, process::Command, str::FromStr};

use ethers::types::Address;

/// Deploys the artifact at the given path to the given endpoint.
///
/// Returns the address of the deployed program.
pub fn deploy_contract(
    path: impl ToString,
    key_path: impl ToString,
    endpoint: impl ToString,
) -> Address {
    println!("deploying {}", path.to_string());

    let path = fs::canonicalize(&path.to_string()).expect("canonicalize");

    let output = Command::new("cargo")
        .arg("stylus")
        .arg("deploy")
        .arg(format!("--private-key-path={}", key_path.to_string()))
        .arg(format!("-e={}", endpoint.to_string()))
        .arg(format!("--wasm-file-path={}", path.to_str().unwrap()))
        .output()
        .unwrap();

    if !output.status.success() {
        panic!(
            "deploy failed:\nstdout: {}\nstderr: {}",
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        );
    }

    // this will produce output like:
    // ...
    // Deploying program to address 0xE34C0a16B60FB9696823245Fd3Ab6B9Db46aA2C3
    // ...
    //
    // We need to extract the address from this output.

    let output = String::from_utf8(output.stdout).unwrap();
    let address = output
        .lines()
        .find(|line| line.starts_with("Deploying program to address"))
        .expect("address line")
        .split_whitespace()
        .last()
        .expect("address");

    let address = &address[12..54];

    println!("\tdeployed to address: {:?}", address);

    Address::from_str(address).expect("address")
}
