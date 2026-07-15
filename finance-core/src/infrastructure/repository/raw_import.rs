use std::str::FromStr;

use rusqlite::{params, Connection};

use crate::domain::raw_import::RawImportRecord;
use crate::domain::source::ImportSource;
use crate::errors::FinanceError;

pub struct RawImportRepository<'a> {
    conn: &'a Connection,
}

impl<'a> RawImportRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn insert(&self, record: &mut RawImportRecord) -> Result<(), FinanceError> {
        let metadata_json = serde_json::to_string(&record.metadata)
            .map_err(|e| FinanceError::CorruptData(format!("metadata serialization: {e}")))?;

        self.conn.execute(
            "INSERT INTO raw_imports (source, mime_type, payload, checksum, imported_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                record.source.to_string(),
                record.mime_type,
                record.payload,
                &record.checksum[..],
                record.imported_at.to_rfc3339(),
                metadata_json,
            ],
        )?;

        record.id = Some(self.conn.last_insert_rowid());
        Ok(())
    }

    pub fn find_by_id(&self, id: i64) -> Result<Option<RawImportRecord>, FinanceError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source, mime_type, payload, checksum, imported_at, metadata
             FROM raw_imports WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_raw_import(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn find_by_checksum(&self, checksum: &[u8]) -> Result<Option<RawImportRecord>, FinanceError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source, mime_type, payload, checksum, imported_at, metadata
             FROM raw_imports WHERE checksum = ?1 LIMIT 1",
        )?;

        let mut rows = stmt.query(params![checksum])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_raw_import(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn exists_checksum(&self, checksum: &[u8]) -> Result<bool, FinanceError> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM raw_imports WHERE checksum = ?1 LIMIT 1")?;
        let mut rows = stmt.query(params![checksum])?;
        Ok(rows.next()?.is_some())
    }

    /// Insert a raw import, but only if its checksum doesn't already exist.
    ///
    /// Returns:
    /// - `Ok(true)` if inserted successfully
    /// - `Ok(false)` if a record with this checksum already exists (skipped)
    ///
    /// This is the primary dedup mechanism: same payload = same checksum = skip.
    pub fn insert_if_new(&self, record: &mut RawImportRecord) -> Result<bool, FinanceError> {
        if self.exists_checksum(&record.checksum)? {
            return Ok(false);
        }

        self.insert(record)?;
        Ok(true)
    }

    pub fn delete(&self, id: i64) -> Result<bool, FinanceError> {
        let rows = self
            .conn
            .execute("DELETE FROM raw_imports WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }
}

fn row_to_raw_import(row: &rusqlite::Row<'_>) -> Result<RawImportRecord, FinanceError> {
    let id: i64 = row.get(0).map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let source_str: String = row.get(1).map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let mime_type: Option<String> = row.get(2).map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let payload: Vec<u8> = row.get(3).map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let checksum_blob: Vec<u8> = row.get(4).map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let imported_at_str: String = row.get(5).map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let metadata_json: String = row.get(6).map_err(|e| FinanceError::CorruptData(e.to_string()))?;

    let source = ImportSource::from_str(&source_str)
        .map_err(|e| FinanceError::CorruptData(format!("invalid source: {e}")))?;

    let imported_at = chrono::DateTime::parse_from_rfc3339(&imported_at_str)
        .map_err(|e| FinanceError::CorruptData(format!("invalid timestamp: {e}")))?
        .with_timezone(&chrono::Utc);

    let metadata: std::collections::HashMap<String, String> =
        serde_json::from_str(&metadata_json)
            .map_err(|e| FinanceError::CorruptData(format!("invalid metadata: {e}")))?;

    let checksum: [u8; crate::domain::raw_import::SHA256_SIZE] = checksum_blob
        .try_into()
        .map_err(|_| FinanceError::CorruptData("invalid checksum length".to_string()))?;

    Ok(RawImportRecord {
        id: Some(id),
        source,
        mime_type,
        payload,
        checksum,
        imported_at,
        metadata,
    })
}
