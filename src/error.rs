use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HnmError {
    #[error("target path is not a directory: {0}")]
    NotADirectory(PathBuf),

    #[error("failed to create directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write {path}: {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to create symlink {link} -> {target}: {source}")]
    Symlink {
        link: PathBuf,
        target: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to remove {path}: {source}")]
    Remove {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("template error: {0}")]
    Template(#[from] minijinja::Error),

    #[error("unknown template `{0}`")]
    MissingTemplate(String),
}

pub type Result<T> = std::result::Result<T, HnmError>;
