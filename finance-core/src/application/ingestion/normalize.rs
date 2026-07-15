use crate::domain::{
    normalized_candidate::NormalizedCandidate, parsed_candidate::ParsedCandidate,
};

use super::stage::{PipelineError, PipelineStage};

/// Normalize a ParsedCandidate into canonical form.
///
/// This is the "Normalize" stage of the ETL pipeline.
/// Applies defaults for missing fields, canonicalizes merchant names.
/// Category inference is deferred to the Enrich stage.
pub struct NormalizeStage;

impl PipelineStage for NormalizeStage {
    type Input = ParsedCandidate;
    type Output = NormalizedCandidate;

    fn run(&self, candidate: ParsedCandidate) -> Result<NormalizedCandidate, PipelineError> {
        Ok(NormalizedCandidate::from_parsed(candidate))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        money::Money, parser_info::ParserInfo, transaction_type::TransactionType,
    };
    use rust_decimal::Decimal;

    fn test_parser_info() -> ParserInfo {
        ParserInfo::new("TestParser", 100)
    }

    #[test]
    fn normalizes_full_candidate() {
        let candidate = ParsedCandidate::new(test_parser_info())
            .with_amount(Money::new(Decimal::new(50000, 2)))
            .with_merchant("Swiggy")
            .with_kind(TransactionType::Debit)
            .with_reference("1234567890");

        let normalized = NormalizedCandidate::from_parsed(candidate);

        assert_eq!(normalized.amount.amount(), Decimal::new(50000, 2));
        assert_eq!(normalized.merchant.name, "SWIGGY");
        assert_eq!(normalized.kind, TransactionType::Debit);
        assert_eq!(
            normalized.reference,
            Some("1234567890".to_string())
        );
        assert_eq!(normalized.category, crate::domain::category::Category::Other);
        assert!(!normalized.id.is_nil());
    }

    #[test]
    fn normalizes_candidate_with_defaults() {
        let candidate = ParsedCandidate::new(test_parser_info())
            .with_amount(Money::new(Decimal::new(100, 0)));

        let normalized = NormalizedCandidate::from_parsed(candidate);

        assert_eq!(normalized.amount.amount(), Decimal::new(100, 0));
        assert_eq!(normalized.merchant.name, "UNKNOWN");
        assert_eq!(normalized.kind, TransactionType::Credit); // default
        assert_eq!(normalized.category, crate::domain::category::Category::Other);
        assert!(normalized.reference.is_none());
    }

    #[test]
    fn normalize_stage_returns_ok() {
        use crate::application::ingestion::stage::PipelineStage;

        let candidate = ParsedCandidate::new(test_parser_info())
            .with_amount(Money::new(Decimal::new(200, 0)))
            .with_merchant("Uber");

        let stage = NormalizeStage;
        let normalized = stage.run(candidate).unwrap();
        assert_eq!(normalized.amount.amount(), Decimal::new(200, 0));
        assert_eq!(
            normalized.category,
            crate::domain::category::Category::Other
        );
    }

    #[test]
    fn merchant_name_is_uppercased() {
        let candidate = ParsedCandidate::new(test_parser_info())
            .with_amount(Money::new(Decimal::new(100, 0)))
            .with_merchant("swiggy");

        let normalized = NormalizedCandidate::from_parsed(candidate);
        assert_eq!(normalized.merchant.name, "SWIGGY");
    }

    #[test]
    fn empty_merchant_becomes_unknown() {
        let candidate = ParsedCandidate::new(test_parser_info())
            .with_amount(Money::new(Decimal::new(100, 0)));

        let normalized = NormalizedCandidate::from_parsed(candidate);
        assert_eq!(normalized.merchant.name, "UNKNOWN");
    }
}
