use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::domain::source::ImportSource;

/// SHA-256 hash size in bytes.
pub const SHA256_SIZE: usize = 32;

/// A raw, unmodified import record.
///
/// Preserves the original data exactly as received. If parsers improve,
/// re-run ETL over raw imports instead of asking users to re-import.
///
/// This is the "Raw Zone" in the data warehouse pattern.
/// Immutable — never modified after creation.
#[derive(Debug, Clone)]
pub struct RawImportRecord {
    pub id: Option<i64>,
    pub source: ImportSource,
    pub mime_type: Option<String>,
    pub payload: Vec<u8>,
    pub checksum: [u8; SHA256_SIZE],
    pub imported_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl RawImportRecord {
    pub fn new(source: ImportSource, payload: Vec<u8>) -> Self {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&payload);
        let checksum: [u8; SHA256_SIZE] = hasher.finalize().into();

        Self {
            id: None,
            source,
            mime_type: None,
            payload,
            checksum,
            imported_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Convenience constructor for text payloads.
    pub fn from_text(source: ImportSource, text: impl Into<String>) -> Self {
        Self::new(source, text.into().into_bytes())
    }

    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Interpret payload as UTF-8 text. Returns None if payload is not valid UTF-8.
    pub fn as_text(&self) -> Option<&str> {
        std::str::from_utf8(&self.payload).ok()
    }

    /// Get checksum as lowercase hex string.
    pub fn checksum_hex(&self) -> String {
        self.checksum.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
