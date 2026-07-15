use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
    category::Category, enriched_candidate::EnrichedCandidate, merchant::Merchant, money::Money,
    transaction_type::TransactionType,
};

/// A fully validated, enriched, and persisted financial transaction.
///
/// This is the only type that can be stored in the database.
/// It can only be constructed from an `EnrichedCandidate`, ensuring
/// that validation and enrichment are never skipped.
#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: Uuid,
    pub amount: Money,
    pub merchant: Merchant,
    pub category: Category,
    pub kind: TransactionType,
    pub timestamp: DateTime<Utc>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

impl Transaction {
    /// Create a transaction from an enriched candidate.
    ///
    /// This is the canonical constructor — the only way to create a Transaction.
    pub fn new(candidate: EnrichedCandidate) -> Self {
        let category = candidate.category;
        let validated = candidate.into_inner();
        let normalized = validated.into_inner();

        Self {
            id: normalized.id,
            amount: normalized.amount,
            merchant: normalized.merchant,
            category,
            kind: normalized.kind,
            timestamp: normalized.timestamp,
            reference: normalized.reference,
            notes: normalized.notes,
        }
    }
}
