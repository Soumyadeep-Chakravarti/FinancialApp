use thiserror::Error;

#[derive(Debug, Error)]
pub enum FinanceError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Corrupt data in database: {0}")]
    CorruptData(String),
}
