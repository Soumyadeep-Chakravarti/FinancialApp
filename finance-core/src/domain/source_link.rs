use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{parser_info::ParserInfo, relationship_type::RelationshipType};

/// Links a canonical transaction to the raw import(s) it originated from.
///
/// This is a provenance/lineage table, not just a foreign key join.
/// Every link carries semantic meaning (relationship_type) and parser lineage.
#[derive(Debug, Clone)]
pub struct SourceLink {
    pub transaction_id: Uuid,
    pub raw_import_id: i64,
    pub relationship: RelationshipType,
    pub parser: ParserInfo,
    pub created_at: DateTime<Utc>,
}

impl SourceLink {
    pub fn new(
        transaction_id: Uuid,
        raw_import_id: i64,
        relationship: RelationshipType,
        parser: ParserInfo,
    ) -> Self {
        Self {
            transaction_id,
            raw_import_id,
            relationship,
            parser,
            created_at: Utc::now(),
        }
    }
}
