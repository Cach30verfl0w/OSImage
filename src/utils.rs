use std::env;
use std::path::{Path, PathBuf};

// https://stackoverflow.com/questions/37498864/finding-executable-in-path-with-rust/37499032#37499032
pub(crate) fn find_in_path<P>(name: P) -> Option<PathBuf> where P: AsRef<Path> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(&name);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    })
}