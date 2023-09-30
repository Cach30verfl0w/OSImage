use std::path::Path;
use cargo_toml::Manifest;
use toml::Value;
use crate::error::Error;

pub(crate) fn find_manifest_and_validate<P: AsRef<Path>>(path: P) -> Result<Manifest<Value>, Error> {
    let path = path.as_ref().join("Cargo.toml");

    // Validate the existence of a Cargo.toml
    if !path.exists() {
        return Err(Error::NoCargoFile(path.to_string_lossy().into_owned()));
    }

    // Get manifest and validate
    Manifest::from_path(&path).map_err(|_| Error::InvalidCargoFile(path.to_string_lossy()
        .into_owned()))
}