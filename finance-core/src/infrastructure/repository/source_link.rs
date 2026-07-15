use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::domain::{
    parser_info::ParserInfo, relationship_type::RelationshipType, source_link::SourceLink,
};
use crate::errors::FinanceError;

pub struct SourceLinkRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SourceLinkRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn insert(&self, link: &SourceLink) -> Result<(), FinanceError> {
        self.conn.execute(
            "INSERT INTO source_links (transaction_id, raw_import_id, relationship_type, parser_name, parser_version)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                link.transaction_id.to_string(),
                link.raw_import_id,
                link.relationship.as_str(),
                link.parser.name,
                link.parser.version,
            ],
        )?;
        Ok(())
    }

    pub fn find_by_transaction(&self, transaction_id: Uuid) -> Result<Vec<SourceLink>, FinanceError> {
        let mut stmt = self.conn.prepare(
            "SELECT transaction_id, raw_import_id, relationship_type, parser_name, parser_version
             FROM source_links WHERE transaction_id = ?1",
        )?;

        let mut rows = stmt.query(params![transaction_id.to_string()])?;
        let mut links = Vec::new();
        while let Some(row) = rows.next()? {
            links.push(row_to_source_link(row)?);
        }
        Ok(links)
    }

    pub fn find_by_raw_import(&self, raw_import_id: i64) -> Result<Vec<SourceLink>, FinanceError> {
        let mut stmt = self.conn.prepare(
            "SELECT transaction_id, raw_import_id, relationship_type, parser_name, parser_version
             FROM source_links WHERE raw_import_id = ?1",
        )?;

        let mut rows = stmt.query(params![raw_import_id])?;
        let mut links = Vec::new();
        while let Some(row) = rows.next()? {
            links.push(row_to_source_link(row)?);
        }
        Ok(links)
    }

    pub fn exists(&self, transaction_id: Uuid, raw_import_id: i64) -> Result<bool, FinanceError> {
        let mut stmt = self.conn.prepare(
            "SELECT 1 FROM source_links
             WHERE transaction_id = ?1 AND raw_import_id = ?2 LIMIT 1",
        )?;
        let mut rows = stmt.query(params![transaction_id.to_string(), raw_import_id])?;
        Ok(rows.next()?.is_some())
    }

    pub fn delete_by_transaction(&self, transaction_id: Uuid) -> Result<bool, FinanceError> {
        let rows = self.conn.execute(
            "DELETE FROM source_links WHERE transaction_id = ?1",
            params![transaction_id.to_string()],
        )?;
        Ok(rows > 0)
    }

    /// Find all transactions parsed by a specific parser version.
    pub fn find_by_parser(
        &self,
        parser_name: &str,
        parser_version: u16,
    ) -> Result<Vec<SourceLink>, FinanceError> {
        let mut stmt = self.conn.prepare(
            "SELECT transaction_id, raw_import_id, relationship_type, parser_name, parser_version
             FROM source_links WHERE parser_name = ?1 AND parser_version = ?2",
        )?;

        let mut rows = stmt.query(params![parser_name, parser_version])?;
        let mut links = Vec::new();
        while let Some(row) = rows.next()? {
            links.push(row_to_source_link(row)?);
        }
        Ok(links)
    }

    /// Find all links with a specific relationship type.
    pub fn find_by_relationship(
        &self,
        relationship: RelationshipType,
    ) -> Result<Vec<SourceLink>, FinanceError> {
        let mut stmt = self.conn.prepare(
            "SELECT transaction_id, raw_import_id, relationship_type, parser_name, parser_version
             FROM source_links WHERE relationship_type = ?1",
        )?;

        let mut rows = stmt.query(params![relationship.as_str()])?;
        let mut links = Vec::new();
        while let Some(row) = rows.next()? {
            links.push(row_to_source_link(row)?);
        }
        Ok(links)
    }
}

fn row_to_source_link(row: &rusqlite::Row<'_>) -> Result<SourceLink, FinanceError> {
    let transaction_id_str: String = row
        .get(0)
        .map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let raw_import_id: i64 = row
        .get(1)
        .map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let relationship_str: String = row
        .get(2)
        .map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let parser_name: String = row
        .get(3)
        .map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let parser_version: u16 = row
        .get(4)
        .map_err(|e| FinanceError::CorruptData(e.to_string()))?;

    let transaction_id = Uuid::parse_str(&transaction_id_str)
        .map_err(|e| FinanceError::CorruptData(format!("invalid UUID: {e}")))?;
    let relationship = relationship_str
        .parse()
        .map_err(|e: String| FinanceError::CorruptData(e))?;

    Ok(SourceLink::new(
        transaction_id,
        raw_import_id,
        relationship,
        ParserInfo::new_static(&parser_name, parser_version),
    ))
}
