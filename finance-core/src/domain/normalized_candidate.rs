use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
    category::Category, merchant::Merchant, money::Money, parsed_candidate::ParsedCandidate,
    transaction_type::TransactionType,
};

/// Canonical data after normalization — all fields have defaults applied,
/// but this is not yet a domain entity. Still carries parser lineage.
#[derive(Debug, Clone)]
pub struct NormalizedCandidate {
    pub id: Uuid,
    pub amount: Money,
    pub merchant: Merchant,
    pub category: Category,
    pub kind: TransactionType,
    pub timestamp: DateTime<Utc>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

impl NormalizedCandidate {
    /// Normalize a parsed candidate by applying defaults and canonicalizing data.
    pub fn from_parsed(parsed: ParsedCandidate) -> Self {
        let merchant = parsed.merchant.unwrap_or_default();
        let merchant_name = if merchant.name.is_empty() {
            "Unknown".to_string()
        } else {
            merchant.name.trim().to_uppercase()
        };

        Self {
            id: Uuid::new_v4(),
            amount: parsed.amount.unwrap_or_default(),
            merchant: Merchant {
                name: merchant_name,
                upi_id: merchant.upi_id,
            },
            category: Category::Other,
            kind: parsed.kind.unwrap_or_default(),
            timestamp: parsed.timestamp.unwrap_or_else(Utc::now),
            reference: parsed.reference,
            notes: None,
        }
    }
}
