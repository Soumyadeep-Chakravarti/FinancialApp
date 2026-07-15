use chrono::{DateTime, Utc};

use crate::domain::{merchant::Merchant, money::Money, parser_info::ParserInfo, transaction_type::TransactionType};

/// A partially parsed transaction from a source.
///
/// Fields are optional because source formats vary widely and may not
/// contain all data. The ETL layer normalizes and enriches this into
/// a canonical `Transaction`.
///
/// ParsedCandidate is source-agnostic — it doesn't know where it came from.
/// The relationship to the original raw import is tracked via SourceLink.
#[derive(Debug, Clone)]
pub struct ParsedCandidate {
    pub amount: Option<Money>,
    pub merchant: Option<Merchant>,
    pub kind: Option<TransactionType>,
    pub reference: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
    pub parser: ParserInfo,
}

impl ParsedCandidate {
    pub fn new(parser: ParserInfo) -> Self {
        Self {
            amount: None,
            merchant: None,
            kind: None,
            reference: None,
            timestamp: None,
            parser,
        }
    }

    pub fn with_amount(mut self, amount: Money) -> Self {
        self.amount = Some(amount);
        self
    }

    pub fn with_merchant(mut self, merchant: impl Into<String>) -> Self {
        self.merchant = Some(Merchant::new(merchant));
        self
    }

    pub fn with_kind(mut self, kind: TransactionType) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn with_reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }

    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
}

impl Default for ParsedCandidate {
    fn default() -> Self {
        Self::new(ParserInfo::new("unknown", 0))
    }
}
