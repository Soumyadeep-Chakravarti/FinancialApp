use crate::domain::{
    normalized_candidate::NormalizedCandidate, validated_candidate::ValidatedCandidate,
};

use super::stage::{PipelineError, PipelineStage};

/// Validates that a NormalizedCandidate meets all invariants.
///
/// Rejects candidates with:
/// - Zero or negative amount
/// - Empty merchant name
/// - Nil UUID (should never happen, but safety check)
///
/// On success, wraps the candidate in a `ValidatedCandidate`.
pub struct ValidateStage;

impl PipelineStage for ValidateStage {
    type Input = NormalizedCandidate;
    type Output = ValidatedCandidate;

    fn run(&self, candidate: NormalizedCandidate) -> Result<ValidatedCandidate, PipelineError> {
        if candidate.amount.amount() <= rust_decimal::Decimal::ZERO {
            return Err(PipelineError::Rejected {
                stage: "validate".into(),
                reason: format!("amount must be positive, got {}", candidate.amount.amount()),
            });
        }

        if candidate.merchant.name.is_empty() {
            return Err(PipelineError::Rejected {
                stage: "validate".into(),
                reason: "merchant name is empty".into(),
            });
        }

        if candidate.id.is_nil() {
            return Err(PipelineError::Rejected {
                stage: "validate".into(),
                reason: "transaction ID is nil".into(),
            });
        }

        Ok(ValidatedCandidate::new(candidate))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        category::Category, merchant::Merchant, money::Money,
        normalized_candidate::NormalizedCandidate, transaction_type::TransactionType,
    };
    use rust_decimal::Decimal;

    fn make_valid_candidate() -> NormalizedCandidate {
        NormalizedCandidate {
            id: uuid::Uuid::new_v4(),
            amount: Money::new(Decimal::new(50000, 2)),
            merchant: Merchant::new("SWIGGY"),
            category: Category::Other,
            kind: TransactionType::Debit,
            timestamp: chrono::Utc::now(),
            reference: None,
            notes: None,
        }
    }

    #[test]
    fn accepts_valid_candidate() {
        let stage = ValidateStage;
        let candidate = make_valid_candidate();
        let result = stage.run(candidate).unwrap();
        assert_eq!(result.inner().amount.amount(), Decimal::new(50000, 2));
    }

    #[test]
    fn rejects_zero_amount() {
        let stage = ValidateStage;
        let mut candidate = make_valid_candidate();
        candidate.amount = Money::new(Decimal::ZERO);
        assert!(matches!(
            stage.run(candidate),
            Err(PipelineError::Rejected { .. })
        ));
    }

    #[test]
    fn rejects_negative_amount() {
        let stage = ValidateStage;
        let mut candidate = make_valid_candidate();
        candidate.amount = Money::new(Decimal::new(-100, 0));
        assert!(matches!(
            stage.run(candidate),
            Err(PipelineError::Rejected { .. })
        ));
    }

    #[test]
    fn rejects_empty_merchant() {
        let stage = ValidateStage;
        let mut candidate = make_valid_candidate();
        candidate.merchant = Merchant::new("");
        assert!(matches!(
            stage.run(candidate),
            Err(PipelineError::Rejected { .. })
        ));
    }

    #[test]
    fn rejects_nil_id() {
        let stage = ValidateStage;
        let mut candidate = make_valid_candidate();
        candidate.id = uuid::Uuid::nil();
        assert!(matches!(
            stage.run(candidate),
            Err(PipelineError::Rejected { .. })
        ));
    }
}
