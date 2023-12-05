use std::{fs::DirEntry, path::PathBuf};

use crate::constants::{ARTIFACTS_DIR, EXTENSION, IGNORE_POSTFIXES};

#[derive(Debug, Clone)]
pub struct FileData {
    pub in_file: String,
    pub out_file: String,
    pub path: PathBuf,
}

pub fn filter_func() -> impl FnMut(Result<DirEntry, std::io::Error>) -> Option<FileData> {
    move |f| {
        let path = f.expect("file path").path();

        if !path.is_file() || path.extension().unwrap_or_default() != EXTENSION {
            return None;
        }

        let file_name = path
            .file_name()
            .expect("file_name")
            .to_str()
            .expect("file_name to str")
            .to_string();

        for postfix in IGNORE_POSTFIXES {
            if file_name.ends_with(*postfix) {
                return None;
            }
        }

        let in_file = path.to_str().expect("path to str").to_string();
        let out_file = format!("{}/{}", ARTIFACTS_DIR, file_name);

        Some(FileData {
            in_file,
            out_file,
            path,
        })
    }
}
