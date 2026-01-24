use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Environment error: {0}")]
    Environment(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::de::Error),

    #[error("WASM Runtime error: {0}")]
    Wasm(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
