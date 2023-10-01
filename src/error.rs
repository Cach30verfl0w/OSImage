use glob::{GlobError, PatternError};
use thiserror::Error;

pub(crate) const EXIT_INVALID_WORKSPACE: i32 = -1;
pub(crate) const EXIT_BUILD_ERROR: i32 = -2;
pub(crate) const EXIT_QEMU_ERROR: i32 = -3;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No Cargo.toml file found in '{0}'")]
    NoCargoFile(String),
    #[error("The Cargo.toml in '{0}' is invalid")]
    InvalidCargoFile(String),
    #[error("{0}")]
    PatternError(#[from] PatternError),
    #[error("{0}")]
    GlobError(#[from] GlobError),
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    #[error("Executable '{0}' not found in PATH variable")]
    ExecutableNotFound(String),
    #[error("Build of project '{0}' failed with error code {1}")]
    BuildFailed(String, i32),
    #[error("Illegal parameter '{0}'")]
    InvalidParameter(String),
    #[error("{0} failed with error code {1}")]
    ProcessFailed(String, i32),
}