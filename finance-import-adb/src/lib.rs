pub mod adb;

use finance_core::domain::raw_import::RawImportRecord;

/// Errors that can occur during import.
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("ADB not found: {0}")]
    AdbNotFound(String),

    #[error("no device connected")]
    NoDevice,

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("query failed: {0}")]
    QueryFailed(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Generic importer interface.
///
/// Every importer produces `RawImportRecord`s. The importer does not parse,
/// categorize, or normalize — it only converts source data into raw records.
/// The parser pipeline handles everything else.
pub trait Importer {
    /// Import records from the source.
    fn import(&mut self) -> Result<Vec<RawImportRecord>, ImportError>;
}
