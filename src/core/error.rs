use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArcError {
    #[error("Entity not found: {0:?}")]
    EntityNotFound(crate::core::types::EntityId),

    #[error("Component not found for entity: {0}")]
    ComponentNotFound(String),

    #[error("Invalid action: {0}")]
    InvalidAction(String),

    #[error("LLM error: {0}")]
    LlmError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Navigation error: {0}")]
    NavigationError(String),
}

pub type Result<T> = std::result::Result<T, ArcError>;
